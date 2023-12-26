extern crate ffmpeg_sys_next as ffmpeg;
use std::ffi::CString;
use std::ptr;

fn main() {
    unsafe {
        // Initialize FFmpeg
        ffmpeg::av_register_all();

        // Allocate the output context
        let mut format_context = ptr::null_mut();
        let output_format = ffmpeg::av_guess_format(
            ptr::null(),
            CString::new("output.mp4").unwrap().as_ptr(),
            ptr::null(),
        );
        ffmpeg::avformat_alloc_output_context2(
            &mut format_context,
            output_format,
            ptr::null(),
            CString::new("output.mp4").unwrap().as_ptr(),
        );

        // Set up the codec and parameters
        let codec = ffmpeg::avcodec_find_encoder(ffmpeg::AVCodecID::AV_CODEC_ID_H264);
        let mut codec_context = ffmpeg::avcodec_alloc_context3(codec);
        (*codec_context).width = 1920;
        (*codec_context).height = 1080;
        (*codec_context).time_base = ffmpeg::AVRational { num: 1, den: 25 };
        (*codec_context).framerate = ffmpeg::AVRational { num: 25, den: 1 };
        (*codec_context).gop_size = 10;
        (*codec_context).max_b_frames = 1;
        (*codec_context).pix_fmt = ffmpeg::AVPixelFormat::AV_PIX_FMT_YUV420P;

        // Open the codec
        ffmpeg::avcodec_open2(codec_context, codec, ptr::null_mut());

        let stream = ffmpeg::avformat_new_stream(format_context, ptr::null_mut());
        // Synchronize stream parameters with the codec context
        ffmpeg::avcodec_parameters_from_context((*stream).codecpar, codec_context);

        // Create a new stream
        // (*(*stream).codecpar).codec_type = ffmpeg::AVMediaType::AVMEDIA_TYPE_VIDEO;
        // (*(*stream).codecpar).codec_id = ffmpeg::AVCodecID::AV_CODEC_ID_H264;
        // (*(*stream).codecpar).width = 1920;
        // (*(*stream).codecpar).height = 1080;
        // (*(*stream).codecpar).format = ffmpeg::AVPixelFormat::AV_PIX_FMT_YUV420P as i32;

        // Allocate a frame and set its fields
        let frame = ffmpeg::av_frame_alloc();
        (*frame).format = ffmpeg::AVPixelFormat::AV_PIX_FMT_YUV420P as i32;
        (*frame).width = 1920;
        (*frame).height = 1080;
        ffmpeg::av_frame_get_buffer(frame, 32);

        // Write the header
        ffmpeg::avformat_write_header(format_context, ptr::null_mut());

        // Write frames (for a short duration, e.g., 5 seconds)
        (*frame).pts = 0;
        for i in 0..(25 * 5) {
            // 25 fps * 5 seconds
            (*frame).pts = i;
            ffmpeg::av_write_frame(format_context, ptr::null_mut());
        }

        // Write the trailer and clean up
        ffmpeg::av_write_trailer(format_context);
        ffmpeg::avcodec_close(codec_context);
        ffmpeg::av_free(frame as *mut libc::c_void);
        ffmpeg::avio_closep(&mut (*format_context).pb);
        ffmpeg::avformat_free_context(format_context);
    }
}
