//! Custom I/O for in-memory audio processing.

use crate::error::{check, FfmpegError};
use ffmpeg_sys::*;
use std::ffi::CString;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem;
use std::os::raw::{c_int, c_void};
use std::ptr;
use std::slice;

const AVIO_BUFFER_SIZE: usize = 32 * 1024; // 32KB buffer

/// Read buffer for in-memory input.
pub struct ReadBuffer {
    data: Vec<u8>,
    pos: usize,
}

impl ReadBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }
}

impl Read for ReadBuffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining = self.data.len() - self.pos;
        let to_read = buf.len().min(remaining);
        buf[..to_read].copy_from_slice(&self.data[self.pos..self.pos + to_read]);
        self.pos += to_read;
        Ok(to_read)
    }
}

impl Seek for ReadBuffer {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p as i64,
            SeekFrom::End(p) => self.data.len() as i64 + p,
            SeekFrom::Current(p) => self.pos as i64 + p,
        };
        if new_pos < 0 || new_pos > self.data.len() as i64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek out of bounds",
            ));
        }
        self.pos = new_pos as usize;
        Ok(self.pos as u64)
    }
}

/// Write buffer for in-memory output.
pub struct WriteBuffer {
    data: Vec<u8>,
}

impl WriteBuffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

impl Write for WriteBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Seek for WriteBuffer {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p as i64,
            SeekFrom::End(p) => self.data.len() as i64 + p,
            SeekFrom::Current(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "SeekFrom::Current not supported for write buffer",
                ));
            }
        };
        if new_pos < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek before start",
            ));
        }
        let new_pos = new_pos as usize;
        if new_pos > self.data.len() {
            self.data.resize(new_pos, 0);
        }
        Ok(new_pos as u64)
    }
}

/// RAII wrapper for AVIOContext used for reading.
pub struct InputContext {
    avio: *mut AVIOContext,
    format: *mut AVFormatContext,
    // We need to hold this to keep the opaque pointer valid
    _buffer: Box<ReadBuffer>,
}

impl InputContext {
    pub fn open(data: Vec<u8>) -> Result<Self, FfmpegError> {
        let mut read_buf = Box::new(ReadBuffer::new(data));

        // Allocate AVIO buffer
        let avio_buffer = unsafe { av_malloc(AVIO_BUFFER_SIZE) } as *mut u8;
        if avio_buffer.is_null() {
            return Err(FfmpegError::Allocation("AVIO buffer"));
        }

        // Create custom AVIO context
        let opaque = &mut *read_buf as *mut ReadBuffer as *mut c_void;
        let avio = unsafe {
            avio_alloc_context(
                avio_buffer,
                AVIO_BUFFER_SIZE as c_int,
                0, // read-only
                opaque,
                Some(read_callback),
                None, // no write
                Some(seek_callback),
            )
        };
        if avio.is_null() {
            unsafe { av_free(avio_buffer as *mut c_void) };
            return Err(FfmpegError::Allocation("AVIOContext"));
        }

        // Allocate format context
        let format = unsafe { avformat_alloc_context() };
        if format.is_null() {
            unsafe { avio_context_free(&mut (avio as *mut _)) };
            return Err(FfmpegError::Allocation("AVFormatContext"));
        }

        // Attach AVIO to format context
        unsafe { (*format).pb = avio };

        // Open input (probe format from content)
        let ret = unsafe {
            avformat_open_input(
                &mut (format as *mut _),
                ptr::null(),     // no filename
                ptr::null_mut(), // probe format
                ptr::null_mut(), // no options
            )
        };
        if ret < 0 {
            // format context is freed by avformat_open_input on error
            unsafe { avio_context_free(&mut (avio as *mut _)) };
            return Err(FfmpegError::Ffmpeg {
                code: ret,
                operation: "avformat_open_input",
                detail: crate::error::error_string(ret),
            });
        }

        // Find stream info
        check(
            unsafe { avformat_find_stream_info(format, ptr::null_mut()) },
            "avformat_find_stream_info",
        )?;

        Ok(Self {
            avio,
            format,
            _buffer: read_buf,
        })
    }

    pub fn format_ctx(&self) -> *mut AVFormatContext {
        self.format
    }

    /// Find the best audio stream and return its index.
    pub fn find_audio_stream(&self) -> Result<usize, FfmpegError> {
        let idx = unsafe {
            av_find_best_stream(
                self.format,
                AVMediaType::AVMEDIA_TYPE_AUDIO,
                -1,
                -1,
                ptr::null_mut(),
                0,
            )
        };
        if idx < 0 {
            Err(FfmpegError::NoAudioStream)
        } else {
            Ok(idx as usize)
        }
    }

    /// Get stream by index.
    pub fn stream(&self, idx: usize) -> *mut AVStream {
        unsafe { *(*self.format).streams.add(idx) }
    }
}

impl Drop for InputContext {
    fn drop(&mut self) {
        unsafe {
            avformat_close_input(&mut self.format);
            // Note: avio buffer is freed by avio_context_free
            avio_context_free(&mut self.avio);
        }
    }
}

unsafe impl Send for InputContext {}

/// RAII wrapper for AVIOContext used for writing.
pub struct OutputContext {
    avio: *mut AVIOContext,
    format: *mut AVFormatContext,
    write_buf: *mut WriteBuffer,
    header_written: bool,
}

impl OutputContext {
    pub fn open(format_name: &str) -> Result<Self, FfmpegError> {
        let format_c = CString::new(format_name).unwrap();

        // Allocate output format context
        let mut format: *mut AVFormatContext = ptr::null_mut();
        check(
            unsafe {
                avformat_alloc_output_context2(
                    &mut format,
                    ptr::null_mut(),
                    format_c.as_ptr(),
                    ptr::null(),
                )
            },
            "avformat_alloc_output_context2",
        )?;

        if format.is_null() {
            return Err(FfmpegError::Unsupported(format!(
                "output format '{}'",
                format_name
            )));
        }

        // Allocate write buffer
        let write_buf = Box::into_raw(Box::new(WriteBuffer::new()));

        // Allocate AVIO buffer
        let avio_buffer = unsafe { av_malloc(AVIO_BUFFER_SIZE) } as *mut u8;
        if avio_buffer.is_null() {
            unsafe {
                avformat_free_context(format);
                drop(Box::from_raw(write_buf));
            }
            return Err(FfmpegError::Allocation("AVIO buffer"));
        }

        // Create custom AVIO context for writing
        let avio = unsafe {
            avio_alloc_context(
                avio_buffer,
                AVIO_BUFFER_SIZE as c_int,
                1, // write mode
                write_buf as *mut c_void,
                None, // no read
                Some(mem::transmute(
                    write_callback as unsafe extern "C" fn(*mut c_void, *const u8, c_int) -> c_int,
                )),
                Some(write_seek_callback),
            )
        };
        if avio.is_null() {
            unsafe {
                av_free(avio_buffer as *mut c_void);
                avformat_free_context(format);
                drop(Box::from_raw(write_buf));
            }
            return Err(FfmpegError::Allocation("AVIOContext"));
        }

        // Attach AVIO to format context
        unsafe { (*format).pb = avio };

        Ok(Self {
            avio,
            format,
            write_buf,
            header_written: false,
        })
    }

    pub fn format_ctx(&mut self) -> *mut AVFormatContext {
        self.format
    }

    /// Add an audio stream to the output.
    pub fn add_audio_stream(
        &mut self,
        codec: *const AVCodec,
    ) -> Result<*mut AVStream, FfmpegError> {
        let stream = unsafe { avformat_new_stream(self.format, codec) };
        if stream.is_null() {
            Err(FfmpegError::Allocation("AVStream"))
        } else {
            Ok(stream)
        }
    }

    /// Set metadata on the output.
    pub fn set_metadata(&mut self, key: &str, value: &str) -> Result<(), FfmpegError> {
        let key_c = CString::new(key).unwrap();
        let value_c = CString::new(value).unwrap();
        check(
            unsafe {
                av_dict_set(
                    &mut (*self.format).metadata,
                    key_c.as_ptr(),
                    value_c.as_ptr(),
                    0,
                )
            },
            "av_dict_set",
        )?;
        Ok(())
    }

    /// Write header.
    pub fn write_header(&mut self) -> Result<(), FfmpegError> {
        check(
            unsafe { avformat_write_header(self.format, ptr::null_mut()) },
            "avformat_write_header",
        )?;
        self.header_written = true;
        Ok(())
    }

    /// Write a packet.
    pub fn write_packet(&mut self, pkt: *mut AVPacket) -> Result<(), FfmpegError> {
        check(
            unsafe { av_interleaved_write_frame(self.format, pkt) },
            "av_interleaved_write_frame",
        )?;
        Ok(())
    }

    /// Write trailer and finalize.
    pub fn write_trailer(&mut self) -> Result<(), FfmpegError> {
        if self.header_written {
            check(unsafe { av_write_trailer(self.format) }, "av_write_trailer")?;
        }
        Ok(())
    }

    /// Take the output buffer.
    pub fn take_output(mut self) -> Vec<u8> {
        // Flush any remaining data
        unsafe { avio_flush(self.avio) };

        // Take ownership of the write buffer
        let buf = unsafe { Box::from_raw(self.write_buf) };
        self.write_buf = ptr::null_mut(); // Prevent double-free in Drop
        buf.into_inner()
    }
}

impl Drop for OutputContext {
    fn drop(&mut self) {
        unsafe {
            // Free format context
            avformat_free_context(self.format);
            // Free AVIO (and its internal buffer)
            avio_context_free(&mut self.avio);
            // Free write buffer if not taken
            if !self.write_buf.is_null() {
                drop(Box::from_raw(self.write_buf));
            }
        }
    }
}

unsafe impl Send for OutputContext {}

// FFI callbacks for reading
extern "C" fn read_callback(opaque: *mut c_void, buf: *mut u8, buf_size: c_int) -> c_int {
    let reader = unsafe { &mut *(opaque as *mut ReadBuffer) };
    let slice = unsafe { slice::from_raw_parts_mut(buf, buf_size as usize) };
    match reader.read(slice) {
        Ok(0) => AVERROR_EOF,
        Ok(n) => n as c_int,
        Err(_) => -1,
    }
}

extern "C" fn seek_callback(opaque: *mut c_void, offset: i64, whence: c_int) -> i64 {
    let reader = unsafe { &mut *(opaque as *mut ReadBuffer) };

    // Handle AVSEEK_SIZE
    if whence & AVSEEK_SIZE as c_int != 0 {
        return reader.data.len() as i64;
    }

    let seek_from = match whence {
        0 => SeekFrom::Start(offset as u64), // SEEK_SET
        1 => SeekFrom::Current(offset),      // SEEK_CUR
        2 => SeekFrom::End(offset),          // SEEK_END
        _ => return -1,
    };

    match reader.seek(seek_from) {
        Ok(pos) => pos as i64,
        Err(_) => -1,
    }
}

// FFI callbacks for writing
unsafe extern "C" fn write_callback(opaque: *mut c_void, buf: *const u8, buf_size: c_int) -> c_int {
    let writer = unsafe { &mut *(opaque as *mut WriteBuffer) };
    let slice = unsafe { slice::from_raw_parts(buf as *const u8, buf_size as usize) };
    match writer.write(slice) {
        Ok(n) => n as c_int,
        Err(_) => -1,
    }
}

extern "C" fn write_seek_callback(opaque: *mut c_void, offset: i64, whence: c_int) -> i64 {
    let writer = unsafe { &mut *(opaque as *mut WriteBuffer) };

    // Handle AVSEEK_SIZE
    if whence & AVSEEK_SIZE as c_int != 0 {
        return writer.data.len() as i64;
    }

    let seek_from = match whence {
        0 => SeekFrom::Start(offset as u64), // SEEK_SET
        2 => SeekFrom::End(offset),          // SEEK_END
        _ => return -1,                      // SEEK_CUR not supported for write
    };

    match writer.seek(seek_from) {
        Ok(pos) => pos as i64,
        Err(_) => -1,
    }
}
