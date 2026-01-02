//! Audio processing pipeline.

use crate::error::{check, check_again, FfmpegError};
use crate::handle::{
    find_decoder, find_encoder_by_name, get_filter, CodecContext, FilterGraph, Frame, Packet,
    Resampler,
};
use crate::io::{InputContext, OutputContext};
use ffmpeg_sys::*;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use tracing::{debug, instrument};

/// Output format specification.
#[derive(Debug, Clone)]
pub struct OutputFormat {
    /// Container format name (e.g., "mp3", "ogg", "flac", "wav", "ipod" for m4a)
    pub format: String,
    /// Codec name (e.g., "libmp3lame", "libvorbis", "flac", "pcm_s16le", "aac")
    pub codec: Option<String>,
    /// Sample rate in Hz
    pub sample_rate: Option<i32>,
    /// Number of channels
    pub channels: Option<i32>,
    /// Bit rate in bits per second
    pub bit_rate: Option<i64>,
    /// Quality for VBR encoding (codec-specific)
    pub quality: Option<f32>,
    /// Compression level (codec-specific)
    pub compression_level: Option<i32>,
}

impl OutputFormat {
    /// Create output format from extension.
    pub fn from_extension(ext: &str) -> Self {
        let (format, codec) = match ext.to_lowercase().as_str() {
            "mp3" => ("mp3", Some("libmp3lame")),
            "wav" => ("wav", Some("pcm_s16le")),
            "flac" => ("flac", Some("flac")),
            "ogg" => ("ogg", Some("libvorbis")),
            "m4a" => ("ipod", Some("aac")),
            "opus" => ("ogg", Some("libopus")),
            _ => ("mp3", Some("libmp3lame")),
        };
        Self {
            format: format.to_string(),
            codec: codec.map(String::from),
            sample_rate: None,
            channels: None,
            bit_rate: None,
            quality: None,
            compression_level: None,
        }
    }
}

/// Options for audio processing.
#[derive(Debug)]
pub struct ProcessOptions<'a> {
    /// Input audio data
    pub input: &'a [u8],
    /// Output format specification
    pub output_format: OutputFormat,
    /// Filter graph string (e.g., "volume=0.5,atempo=1.2")
    pub filters: Option<String>,
    /// Metadata tags to set
    pub metadata: &'a HashMap<String, String>,
    /// Start time in seconds (for trimming)
    pub start_time: Option<f64>,
    /// Duration in seconds (for trimming)
    pub duration: Option<f64>,
}

/// Audio processor using FFmpeg.
pub struct AudioProcessor {
    // No state needed for now, but this allows future extension
    // (e.g., thread pool, reusable contexts)
}

impl AudioProcessor {
    /// Create a new audio processor.
    pub fn new() -> Result<Self, FfmpegError> {
        crate::init();
        Ok(Self {})
    }

    /// Process audio data.
    #[instrument(skip(self, opts), fields(input_size = opts.input.len()))]
    pub fn process(&self, opts: ProcessOptions<'_>) -> Result<Vec<u8>, FfmpegError> {
        // Open input
        let input = InputContext::open(opts.input.to_vec())?;
        let audio_stream_idx = input.find_audio_stream()?;
        let stream = input.stream(audio_stream_idx);

        debug!(stream_idx = audio_stream_idx, "Found audio stream");

        // Set up decoder
        let codecpar = unsafe { (*stream).codecpar };
        let codec_id = unsafe { (*codecpar).codec_id };
        let decoder_codec = find_decoder(codec_id)?;

        let mut decoder = CodecContext::new(decoder_codec)?;
        decoder.set_parameters(codecpar)?;
        decoder.open(decoder_codec)?;

        debug!(sample_rate = decoder.sample_rate(), "Decoder opened");

        // Set up encoder
        let encoder_codec = if let Some(ref codec_name) = opts.output_format.codec {
            find_encoder_by_name(codec_name)?
        } else {
            // Default to MP3
            find_encoder_by_name("libmp3lame")?
        };

        let mut encoder = CodecContext::new(encoder_codec)?;

        // Configure encoder
        let out_sample_rate = opts
            .output_format
            .sample_rate
            .unwrap_or(decoder.sample_rate());
        encoder.set_sample_rate(out_sample_rate);
        encoder.set_time_base(AVRational {
            num: 1,
            den: out_sample_rate,
        });

        // Set channel layout
        let out_channels = opts
            .output_format
            .channels
            .unwrap_or_else(|| unsafe { (*decoder.as_ptr()).ch_layout.nb_channels });

        // Create stereo or mono layout based on channel count
        let mut out_layout: AVChannelLayout = unsafe { std::mem::zeroed() };
        if out_channels == 1 {
            unsafe { av_channel_layout_default(&mut out_layout, 1) };
        } else {
            unsafe { av_channel_layout_default(&mut out_layout, 2) };
        }
        encoder.set_ch_layout(&out_layout)?;

        // Set sample format (use first supported format from codec)
        let sample_fmts = unsafe { (*encoder_codec).sample_fmts };
        let out_sample_fmt = if !sample_fmts.is_null() {
            unsafe { *sample_fmts }
        } else {
            AVSampleFormat::AV_SAMPLE_FMT_FLTP
        };
        encoder.set_sample_fmt(out_sample_fmt);

        if let Some(bit_rate) = opts.output_format.bit_rate {
            encoder.set_bit_rate(bit_rate);
        } else {
            encoder.set_bit_rate(192_000); // Default 192kbps
        }

        if let Some(quality) = opts.output_format.quality {
            // Convert quality to FFmpeg's scale (codec-specific)
            encoder.set_global_quality((quality * 100.0) as i32);
        }

        if let Some(level) = opts.output_format.compression_level {
            encoder.set_compression_level(level);
        }

        encoder.open(encoder_codec)?;

        debug!(
            sample_rate = encoder.sample_rate(),
            frame_size = encoder.frame_size(),
            "Encoder opened"
        );

        // Set up filter graph if filters are specified
        let (filter_graph, buffersrc_ctx, buffersink_ctx) = if opts.filters.is_some()
            || decoder.sample_fmt() != out_sample_fmt
            || decoder.sample_rate() != out_sample_rate
            || unsafe { av_channel_layout_compare(decoder.ch_layout(), &out_layout) } != 0
        {
            Some(self.setup_filters(
                &decoder,
                out_sample_rate,
                out_sample_fmt,
                &out_layout,
                opts.filters.as_deref(),
            )?)
        } else {
            None
        }
        .map_or(
            (None, ptr::null_mut(), ptr::null_mut()),
            |(g, src, sink)| (Some(g), src, sink),
        );

        // Set up resampler if needed (for when we don't have a filter graph)
        let mut resampler = if filter_graph.is_none()
            && (decoder.sample_fmt() != out_sample_fmt
                || decoder.sample_rate() != out_sample_rate
                || unsafe { av_channel_layout_compare(decoder.ch_layout(), &out_layout) } != 0)
        {
            let mut r = Resampler::new()?;
            r.configure(
                decoder.ch_layout(),
                decoder.sample_fmt(),
                decoder.sample_rate(),
                &out_layout,
                out_sample_fmt,
                out_sample_rate,
            )?;
            Some(r)
        } else {
            None
        };

        // Set up output
        let mut output = OutputContext::open(&opts.output_format.format)?;

        // Add audio stream
        let out_stream = output.add_audio_stream(encoder_codec)?;
        check(
            unsafe { avcodec_parameters_from_context((*out_stream).codecpar, encoder.as_ptr()) },
            "avcodec_parameters_from_context",
        )?;
        unsafe { (*out_stream).time_base = encoder.time_base() };

        // Set metadata
        for (key, value) in opts.metadata {
            output.set_metadata(key, value)?;
        }

        output.write_header()?;

        // Processing loop
        let mut pkt = Packet::new()?;
        let mut frame = Frame::new()?;
        let mut filt_frame = Frame::new()?;
        let mut enc_pkt = Packet::new()?;
        let mut samples_processed: i64 = 0;

        // Calculate start/end samples for trimming
        let start_samples = opts
            .start_time
            .map(|t| (t * decoder.sample_rate() as f64) as i64);
        let end_samples = opts.duration.map(|d| {
            let start = start_samples.unwrap_or(0);
            start + (d * decoder.sample_rate() as f64) as i64
        });

        loop {
            // Read packet
            let ret = unsafe { av_read_frame(input.format_ctx(), pkt.as_mut_ptr()) };
            if ret < 0 {
                if is_eof(ret) {
                    break;
                }
                check(ret, "av_read_frame")?;
            }

            if pkt.stream_index() as usize != audio_stream_idx {
                pkt.unref();
                continue;
            }

            // Decode
            check(
                unsafe { avcodec_send_packet(decoder.as_mut_ptr(), pkt.as_ptr()) },
                "avcodec_send_packet",
            )?;

            loop {
                let ret =
                    unsafe { avcodec_receive_frame(decoder.as_mut_ptr(), frame.as_mut_ptr()) };
                if !check_again(ret, "avcodec_receive_frame")? {
                    break;
                }

                // Check trimming
                let frame_start = samples_processed;
                let frame_end = frame_start + frame.nb_samples() as i64;
                samples_processed = frame_end;

                if let Some(start) = start_samples {
                    if frame_end <= start {
                        frame.unref();
                        continue;
                    }
                }
                if let Some(end) = end_samples {
                    if frame_start >= end {
                        frame.unref();
                        break;
                    }
                }

                // Apply filters or resample
                let processed_frame = if filter_graph.is_some() {
                    // Push frame through filter graph
                    check(
                        unsafe {
                            av_buffersrc_add_frame_flags(buffersrc_ctx, frame.as_mut_ptr(), 0)
                        },
                        "av_buffersrc_add_frame",
                    )?;

                    loop {
                        let ret = unsafe {
                            av_buffersink_get_frame(buffersink_ctx, filt_frame.as_mut_ptr())
                        };
                        if !check_again(ret, "av_buffersink_get_frame")? {
                            break;
                        }

                        self.encode_frame(
                            &mut encoder,
                            &mut filt_frame,
                            &mut enc_pkt,
                            &mut output,
                            out_stream,
                        )?;
                        filt_frame.unref();
                    }
                    continue;
                } else if let Some(ref mut r) = resampler {
                    r.convert_frame(&mut filt_frame, &frame)?;
                    &mut filt_frame
                } else {
                    &mut frame
                };

                self.encode_frame(
                    &mut encoder,
                    processed_frame,
                    &mut enc_pkt,
                    &mut output,
                    out_stream,
                )?;

                frame.unref();
                filt_frame.unref();
            }

            pkt.unref();
        }

        // Flush decoder
        check(
            unsafe { avcodec_send_packet(decoder.as_mut_ptr(), ptr::null()) },
            "avcodec_send_packet (flush)",
        )?;

        loop {
            let ret = unsafe { avcodec_receive_frame(decoder.as_mut_ptr(), frame.as_mut_ptr()) };
            if !check_again(ret, "avcodec_receive_frame (flush)")? {
                break;
            }

            if filter_graph.is_some() {
                check(
                    unsafe { av_buffersrc_add_frame_flags(buffersrc_ctx, frame.as_mut_ptr(), 0) },
                    "av_buffersrc_add_frame (flush)",
                )?;

                loop {
                    let ret =
                        unsafe { av_buffersink_get_frame(buffersink_ctx, filt_frame.as_mut_ptr()) };
                    if !check_again(ret, "av_buffersink_get_frame (flush)")? {
                        break;
                    }

                    self.encode_frame(
                        &mut encoder,
                        &mut filt_frame,
                        &mut enc_pkt,
                        &mut output,
                        out_stream,
                    )?;
                    filt_frame.unref();
                }
            } else if let Some(ref mut r) = resampler {
                r.convert_frame(&mut filt_frame, &frame)?;
                self.encode_frame(
                    &mut encoder,
                    &mut filt_frame,
                    &mut enc_pkt,
                    &mut output,
                    out_stream,
                )?;
            } else {
                self.encode_frame(
                    &mut encoder,
                    &mut frame,
                    &mut enc_pkt,
                    &mut output,
                    out_stream,
                )?;
            }

            frame.unref();
            filt_frame.unref();
        }

        // Flush filter graph
        if filter_graph.is_some() {
            check(
                unsafe { av_buffersrc_add_frame_flags(buffersrc_ctx, ptr::null_mut(), 0) },
                "av_buffersrc_add_frame (eof)",
            )?;

            loop {
                let ret =
                    unsafe { av_buffersink_get_frame(buffersink_ctx, filt_frame.as_mut_ptr()) };
                if !check_again(ret, "av_buffersink_get_frame (eof)")? {
                    break;
                }

                self.encode_frame(
                    &mut encoder,
                    &mut filt_frame,
                    &mut enc_pkt,
                    &mut output,
                    out_stream,
                )?;
                filt_frame.unref();
            }
        }

        // Flush resampler
        if let Some(ref mut r) = resampler {
            r.flush(&mut filt_frame)?;
            if filt_frame.nb_samples() > 0 {
                self.encode_frame(
                    &mut encoder,
                    &mut filt_frame,
                    &mut enc_pkt,
                    &mut output,
                    out_stream,
                )?;
            }
        }

        // Flush encoder
        check(
            unsafe { avcodec_send_frame(encoder.as_mut_ptr(), ptr::null()) },
            "avcodec_send_frame (flush)",
        )?;

        loop {
            let ret = unsafe { avcodec_receive_packet(encoder.as_mut_ptr(), enc_pkt.as_mut_ptr()) };
            if !check_again(ret, "avcodec_receive_packet (flush)")? {
                break;
            }

            unsafe {
                av_packet_rescale_ts(
                    enc_pkt.as_mut_ptr(),
                    encoder.time_base(),
                    (*out_stream).time_base,
                );
                (*enc_pkt.as_mut_ptr()).stream_index = 0;
            }

            output.write_packet(enc_pkt.as_mut_ptr())?;
            enc_pkt.unref();
        }

        output.write_trailer()?;

        let result = output.take_output();
        debug!(output_size = result.len(), "Processing complete");

        Ok(result)
    }

    fn setup_filters(
        &self,
        decoder: &CodecContext,
        out_sample_rate: i32,
        out_sample_fmt: AVSampleFormat,
        out_layout: &AVChannelLayout,
        filters: Option<&str>,
    ) -> Result<(FilterGraph, *mut AVFilterContext, *mut AVFilterContext), FfmpegError> {
        let mut graph = FilterGraph::new()?;

        // Get buffer source and sink filters
        let buffersrc = get_filter("abuffer")?;
        let buffersink = get_filter("abuffersink")?;

        // Build abuffer args
        let mut ch_layout_str = [0i8; 64];
        unsafe {
            av_channel_layout_describe(
                decoder.ch_layout(),
                ch_layout_str.as_mut_ptr(),
                ch_layout_str.len(),
            );
        }
        let ch_layout = unsafe {
            std::ffi::CStr::from_ptr(ch_layout_str.as_ptr())
                .to_string_lossy()
                .into_owned()
        };

        let sample_fmt_name = unsafe {
            let name = av_get_sample_fmt_name(decoder.sample_fmt());
            std::ffi::CStr::from_ptr(name)
                .to_string_lossy()
                .into_owned()
        };

        let abuffer_args = format!(
            "time_base=1/{}:sample_rate={}:sample_fmt={}:channel_layout={}",
            decoder.sample_rate(),
            decoder.sample_rate(),
            sample_fmt_name,
            ch_layout
        );

        let buffersrc_ctx = graph.create_filter(buffersrc, "in", Some(&abuffer_args))?;
        let buffersink_ctx = graph.create_filter(buffersink, "out", None)?;

        // Set output format on buffersink
        let sample_fmts = [out_sample_fmt, AVSampleFormat::AV_SAMPLE_FMT_NONE];
        check(
            unsafe {
                let opt_name = CString::new("sample_fmts").unwrap();
                av_opt_set_bin(
                    buffersink_ctx as *mut _,
                    opt_name.as_ptr(),
                    sample_fmts.as_ptr() as *const u8,
                    std::mem::size_of::<AVSampleFormat>() as i32,
                    AV_OPT_SEARCH_CHILDREN as i32,
                )
            },
            "av_opt_set_bin (sample_fmts)",
        )?;

        let sample_rates = [out_sample_rate, 0i32];
        check(
            unsafe {
                let opt_name = CString::new("sample_rates").unwrap();
                av_opt_set_bin(
                    buffersink_ctx as *mut _,
                    opt_name.as_ptr(),
                    sample_rates.as_ptr() as *const u8,
                    std::mem::size_of::<i32>() as i32,
                    AV_OPT_SEARCH_CHILDREN as i32,
                )
            },
            "av_opt_set_bin (sample_rates)",
        )?;

        check(
            unsafe {
                let opt_name = CString::new("ch_layouts").unwrap();
                let mut layout_str = [0i8; 64];
                av_channel_layout_describe(out_layout, layout_str.as_mut_ptr(), layout_str.len());
                av_opt_set(
                    buffersink_ctx as *mut _,
                    opt_name.as_ptr(),
                    layout_str.as_ptr(),
                    AV_OPT_SEARCH_CHILDREN as i32,
                )
            },
            "av_opt_set (ch_layouts)",
        )?;

        // Build filter chain
        let filter_str = if let Some(f) = filters {
            if f.is_empty() {
                "anull".to_string()
            } else {
                f.to_string()
            }
        } else {
            "anull".to_string()
        };

        // Create in/out endpoints
        let mut outputs = unsafe { avfilter_inout_alloc() };
        let mut inputs = unsafe { avfilter_inout_alloc() };

        if outputs.is_null() || inputs.is_null() {
            unsafe {
                avfilter_inout_free(&mut outputs);
                avfilter_inout_free(&mut inputs);
            }
            return Err(FfmpegError::Allocation("AVFilterInOut"));
        }

        unsafe {
            let out_name = CString::new("in").unwrap();
            (*outputs).name = av_strdup(out_name.as_ptr());
            (*outputs).filter_ctx = buffersrc_ctx;
            (*outputs).pad_idx = 0;
            (*outputs).next = ptr::null_mut();

            let in_name = CString::new("out").unwrap();
            (*inputs).name = av_strdup(in_name.as_ptr());
            (*inputs).filter_ctx = buffersink_ctx;
            (*inputs).pad_idx = 0;
            (*inputs).next = ptr::null_mut();
        }

        graph.parse(&filter_str, inputs, outputs)?;
        graph.config()?;

        debug!(filters = filter_str, "Filter graph configured");

        Ok((graph, buffersrc_ctx, buffersink_ctx))
    }

    fn encode_frame(
        &self,
        encoder: &mut CodecContext,
        frame: &mut Frame,
        pkt: &mut Packet,
        output: &mut OutputContext,
        out_stream: *mut AVStream,
    ) -> Result<(), FfmpegError> {
        check(
            unsafe { avcodec_send_frame(encoder.as_mut_ptr(), frame.as_ptr()) },
            "avcodec_send_frame",
        )?;

        loop {
            let ret = unsafe { avcodec_receive_packet(encoder.as_mut_ptr(), pkt.as_mut_ptr()) };
            if !check_again(ret, "avcodec_receive_packet")? {
                break;
            }

            unsafe {
                av_packet_rescale_ts(
                    pkt.as_mut_ptr(),
                    encoder.time_base(),
                    (*out_stream).time_base,
                );
                (*pkt.as_mut_ptr()).stream_index = 0;
            }

            output.write_packet(pkt.as_mut_ptr())?;
            pkt.unref();
        }

        Ok(())
    }
}

impl Default for AudioProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to initialize audio processor")
    }
}

fn is_eof(code: i32) -> bool {
    ffmpeg_sys::is_eof(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_extension() {
        let mp3 = OutputFormat::from_extension("mp3");
        assert_eq!(mp3.format, "mp3");
        assert_eq!(mp3.codec, Some("libmp3lame".to_string()));

        let wav = OutputFormat::from_extension("wav");
        assert_eq!(wav.format, "wav");
        assert_eq!(wav.codec, Some("pcm_s16le".to_string()));

        let flac = OutputFormat::from_extension("flac");
        assert_eq!(flac.format, "flac");
        assert_eq!(flac.codec, Some("flac".to_string()));
    }
}
