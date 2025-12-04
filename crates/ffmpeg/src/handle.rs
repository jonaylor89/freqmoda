//! RAII wrappers for FFmpeg resources.

use crate::error::{check, FfmpegError};
use ffmpeg_sys::*;
use std::ffi::CString;
use std::ptr;

/// RAII wrapper for AVFrame.
pub struct Frame(*mut AVFrame);

impl Frame {
    pub fn new() -> Result<Self, FfmpegError> {
        let ptr = unsafe { av_frame_alloc() };
        if ptr.is_null() {
            Err(FfmpegError::Allocation("AVFrame"))
        } else {
            Ok(Self(ptr))
        }
    }

    pub fn as_ptr(&self) -> *const AVFrame {
        self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVFrame {
        self.0
    }

    /// Unreference frame data, keeping the structure allocated.
    pub fn unref(&mut self) {
        unsafe { av_frame_unref(self.0) }
    }

    /// Get number of samples in frame.
    pub fn nb_samples(&self) -> i32 {
        unsafe { (*self.0).nb_samples }
    }

    /// Get sample format.
    pub fn format(&self) -> i32 {
        unsafe { (*self.0).format }
    }

    /// Get sample rate.
    pub fn sample_rate(&self) -> i32 {
        unsafe { (*self.0).sample_rate }
    }

    /// Get channel layout.
    pub fn ch_layout(&self) -> &AVChannelLayout {
        unsafe { &(*self.0).ch_layout }
    }

    /// Get pts (presentation timestamp).
    pub fn pts(&self) -> i64 {
        unsafe { (*self.0).pts }
    }

    /// Set pts.
    pub fn set_pts(&mut self, pts: i64) {
        unsafe { (*self.0).pts = pts }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe { av_frame_free(&mut self.0) }
    }
}

unsafe impl Send for Frame {}

/// RAII wrapper for AVPacket.
pub struct Packet(*mut AVPacket);

impl Packet {
    pub fn new() -> Result<Self, FfmpegError> {
        let ptr = unsafe { av_packet_alloc() };
        if ptr.is_null() {
            Err(FfmpegError::Allocation("AVPacket"))
        } else {
            Ok(Self(ptr))
        }
    }

    pub fn as_ptr(&self) -> *const AVPacket {
        self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVPacket {
        self.0
    }

    /// Unreference packet data.
    pub fn unref(&mut self) {
        unsafe { av_packet_unref(self.0) }
    }

    /// Get stream index.
    pub fn stream_index(&self) -> i32 {
        unsafe { (*self.0).stream_index }
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        unsafe { av_packet_free(&mut self.0) }
    }
}

unsafe impl Send for Packet {}

/// RAII wrapper for AVCodecContext.
pub struct CodecContext(*mut AVCodecContext);

impl CodecContext {
    pub fn new(codec: *const AVCodec) -> Result<Self, FfmpegError> {
        let ptr = unsafe { avcodec_alloc_context3(codec) };
        if ptr.is_null() {
            Err(FfmpegError::Allocation("AVCodecContext"))
        } else {
            Ok(Self(ptr))
        }
    }

    pub fn as_ptr(&self) -> *const AVCodecContext {
        self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVCodecContext {
        self.0
    }

    /// Copy parameters from stream to context.
    pub fn set_parameters(&mut self, par: *const AVCodecParameters) -> Result<(), FfmpegError> {
        check(
            unsafe { avcodec_parameters_to_context(self.0, par) },
            "avcodec_parameters_to_context",
        )?;
        Ok(())
    }

    /// Open codec.
    pub fn open(&mut self, codec: *const AVCodec) -> Result<(), FfmpegError> {
        check(
            unsafe { avcodec_open2(self.0, codec, ptr::null_mut()) },
            "avcodec_open2",
        )?;
        Ok(())
    }

    /// Get sample format.
    pub fn sample_fmt(&self) -> AVSampleFormat {
        unsafe { (*self.0).sample_fmt }
    }

    /// Get sample rate.
    pub fn sample_rate(&self) -> i32 {
        unsafe { (*self.0).sample_rate }
    }

    /// Get channel layout.
    pub fn ch_layout(&self) -> &AVChannelLayout {
        unsafe { &(*self.0).ch_layout }
    }

    /// Get time base.
    pub fn time_base(&self) -> AVRational {
        unsafe { (*self.0).time_base }
    }

    /// Get frame size (for encoders).
    pub fn frame_size(&self) -> i32 {
        unsafe { (*self.0).frame_size }
    }

    /// Set sample format.
    pub fn set_sample_fmt(&mut self, fmt: AVSampleFormat) {
        unsafe { (*self.0).sample_fmt = fmt }
    }

    /// Set sample rate.
    pub fn set_sample_rate(&mut self, rate: i32) {
        unsafe { (*self.0).sample_rate = rate }
    }

    /// Set bit rate.
    pub fn set_bit_rate(&mut self, rate: i64) {
        unsafe { (*self.0).bit_rate = rate }
    }

    /// Set time base.
    pub fn set_time_base(&mut self, tb: AVRational) {
        unsafe { (*self.0).time_base = tb }
    }

    /// Set channel layout (copies the layout).
    pub fn set_ch_layout(&mut self, layout: &AVChannelLayout) -> Result<(), FfmpegError> {
        check(
            unsafe { av_channel_layout_copy(&mut (*self.0).ch_layout, layout) },
            "av_channel_layout_copy",
        )?;
        Ok(())
    }

    /// Set global quality for VBR encoding.
    pub fn set_global_quality(&mut self, quality: i32) {
        unsafe {
            (*self.0).global_quality = quality;
            (*self.0).flags |= AV_CODEC_FLAG_QSCALE as i32;
        }
    }

    /// Set compression level.
    pub fn set_compression_level(&mut self, level: i32) {
        unsafe { (*self.0).compression_level = level }
    }
}

impl Drop for CodecContext {
    fn drop(&mut self) {
        unsafe { avcodec_free_context(&mut self.0) }
    }
}

unsafe impl Send for CodecContext {}

/// RAII wrapper for AVFilterGraph.
pub struct FilterGraph(*mut AVFilterGraph);

impl FilterGraph {
    pub fn new() -> Result<Self, FfmpegError> {
        let ptr = unsafe { avfilter_graph_alloc() };
        if ptr.is_null() {
            Err(FfmpegError::Allocation("AVFilterGraph"))
        } else {
            Ok(Self(ptr))
        }
    }

    pub fn as_ptr(&self) -> *const AVFilterGraph {
        self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVFilterGraph {
        self.0
    }

    /// Create a filter and add it to the graph.
    pub fn create_filter(
        &mut self,
        filter: *const AVFilter,
        name: &str,
        args: Option<&str>,
    ) -> Result<*mut AVFilterContext, FfmpegError> {
        let name_c = CString::new(name).unwrap();
        let args_c = args.map(|s| CString::new(s).unwrap());

        let mut ctx: *mut AVFilterContext = ptr::null_mut();
        check(
            unsafe {
                avfilter_graph_create_filter(
                    &mut ctx,
                    filter,
                    name_c.as_ptr(),
                    args_c.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
                    ptr::null_mut(),
                    self.0,
                )
            },
            "avfilter_graph_create_filter",
        )?;
        Ok(ctx)
    }

    /// Configure the filter graph.
    pub fn config(&mut self) -> Result<(), FfmpegError> {
        check(
            unsafe { avfilter_graph_config(self.0, ptr::null_mut()) },
            "avfilter_graph_config",
        )?;
        Ok(())
    }

    /// Parse a filter string and add to graph.
    pub fn parse(
        &mut self,
        filters: &str,
        inputs: *mut AVFilterInOut,
        outputs: *mut AVFilterInOut,
    ) -> Result<(), FfmpegError> {
        let filters_c = CString::new(filters).unwrap();
        let mut inputs = inputs;
        let mut outputs = outputs;
        check(
            unsafe {
                avfilter_graph_parse_ptr(
                    self.0,
                    filters_c.as_ptr(),
                    &mut inputs,
                    &mut outputs,
                    ptr::null_mut(),
                )
            },
            "avfilter_graph_parse_ptr",
        )?;
        // Free the in/out structures after parsing
        unsafe {
            avfilter_inout_free(&mut inputs);
            avfilter_inout_free(&mut outputs);
        }
        Ok(())
    }
}

impl Drop for FilterGraph {
    fn drop(&mut self) {
        unsafe { avfilter_graph_free(&mut self.0) }
    }
}

unsafe impl Send for FilterGraph {}

/// RAII wrapper for SwrContext.
pub struct Resampler(*mut SwrContext);

impl Resampler {
    pub fn new() -> Result<Self, FfmpegError> {
        let ptr = unsafe { swr_alloc() };
        if ptr.is_null() {
            Err(FfmpegError::Allocation("SwrContext"))
        } else {
            Ok(Self(ptr))
        }
    }

    pub fn as_ptr(&self) -> *const SwrContext {
        self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut SwrContext {
        self.0
    }

    /// Configure resampler with input and output parameters.
    pub fn configure(
        &mut self,
        in_layout: &AVChannelLayout,
        in_fmt: AVSampleFormat,
        in_rate: i32,
        out_layout: &AVChannelLayout,
        out_fmt: AVSampleFormat,
        out_rate: i32,
    ) -> Result<(), FfmpegError> {
        check(
            unsafe {
                swr_alloc_set_opts2(
                    &mut self.0,
                    out_layout,
                    out_fmt,
                    out_rate,
                    in_layout,
                    in_fmt,
                    in_rate,
                    0,
                    ptr::null_mut(),
                )
            },
            "swr_alloc_set_opts2",
        )?;
        check(unsafe { swr_init(self.0) }, "swr_init")?;
        Ok(())
    }

    /// Convert audio data.
    pub fn convert_frame(&mut self, output: &mut Frame, input: &Frame) -> Result<(), FfmpegError> {
        check(
            unsafe { swr_convert_frame(self.0, output.as_mut_ptr(), input.as_ptr()) },
            "swr_convert_frame",
        )?;
        Ok(())
    }

    /// Flush remaining samples.
    pub fn flush(&mut self, output: &mut Frame) -> Result<(), FfmpegError> {
        check(
            unsafe { swr_convert_frame(self.0, output.as_mut_ptr(), ptr::null()) },
            "swr_convert_frame (flush)",
        )?;
        Ok(())
    }
}

impl Drop for Resampler {
    fn drop(&mut self) {
        unsafe { swr_free(&mut self.0) }
    }
}

unsafe impl Send for Resampler {}

/// RAII wrapper for AVDictionary.
pub struct Dictionary(*mut AVDictionary);

impl Dictionary {
    pub fn new() -> Self {
        Self(ptr::null_mut())
    }

    pub fn as_mut_ptr(&mut self) -> *mut *mut AVDictionary {
        &mut self.0
    }

    /// Set a key-value pair.
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), FfmpegError> {
        let key_c = CString::new(key).unwrap();
        let value_c = CString::new(value).unwrap();
        check(
            unsafe { av_dict_set(&mut self.0, key_c.as_ptr(), value_c.as_ptr(), 0) },
            "av_dict_set",
        )?;
        Ok(())
    }
}

impl Drop for Dictionary {
    fn drop(&mut self) {
        unsafe { av_dict_free(&mut self.0) }
    }
}

/// Helper to find a decoder by codec ID.
pub fn find_decoder(codec_id: AVCodecID) -> Result<*const AVCodec, FfmpegError> {
    let codec = unsafe { avcodec_find_decoder(codec_id) };
    if codec.is_null() {
        Err(FfmpegError::CodecNotFound(format!(
            "decoder for {:?}",
            codec_id
        )))
    } else {
        Ok(codec)
    }
}

/// Helper to find an encoder by name.
pub fn find_encoder_by_name(name: &str) -> Result<*const AVCodec, FfmpegError> {
    let name_c = CString::new(name).unwrap();
    let codec = unsafe { avcodec_find_encoder_by_name(name_c.as_ptr()) };
    if codec.is_null() {
        Err(FfmpegError::CodecNotFound(name.to_string()))
    } else {
        Ok(codec)
    }
}

/// Helper to find an encoder by codec ID.
pub fn find_encoder(codec_id: AVCodecID) -> Result<*const AVCodec, FfmpegError> {
    let codec = unsafe { avcodec_find_encoder(codec_id) };
    if codec.is_null() {
        Err(FfmpegError::CodecNotFound(format!(
            "encoder for {:?}",
            codec_id
        )))
    } else {
        Ok(codec)
    }
}

/// Helper to get filter by name.
pub fn get_filter(name: &str) -> Result<*const AVFilter, FfmpegError> {
    let name_c = CString::new(name).unwrap();
    let filter = unsafe { avfilter_get_by_name(name_c.as_ptr()) };
    if filter.is_null() {
        Err(FfmpegError::FilterConfig(format!(
            "filter '{}' not found",
            name
        )))
    } else {
        Ok(filter)
    }
}
