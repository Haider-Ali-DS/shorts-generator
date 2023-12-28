extern crate ffmpeg_sys_next as ffmpeg;
use std::ffi::CString;
use std::ptr;

use ffmpeg::AVIO_FLAG_WRITE;

fn main() {
    unsafe {
        ffmpeg::av_register_all();
        ffmpeg::av_log_set_level(ffmpeg::AV_LOG_DEBUG);
        let output_file = CString::new("output.mp4").unwrap();
        let mut format_context = ptr::null_mut();

        let output_format =
            ffmpeg::av_guess_format(ptr::null_mut(), output_file.as_ptr(), ptr::null_mut());
        ffmpeg::avformat_alloc_output_context2(
            &mut format_context,
            output_format,
            ptr::null_mut(),
            output_file.as_ptr(),
        );

        // Set up the codec and parameters
        let codec = ffmpeg::avcodec_find_encoder(ffmpeg::AVCodecID::AV_CODEC_ID_H264);
        let codec_context = ffmpeg::avcodec_alloc_context3(codec);
        let stream = ffmpeg::avformat_new_stream(format_context, ptr::null_mut());

        (*codec_context).width = 1920;
        (*codec_context).height = 1080;
        (*codec_context).time_base = ffmpeg::AVRational { num: 1, den: 25 };
        (*codec_context).framerate = ffmpeg::AVRational { num: 25, den: 1 };
        (*codec_context).gop_size = 10;
        (*codec_context).max_b_frames = 1;
        (*codec_context).pix_fmt = ffmpeg::AVPixelFormat::AV_PIX_FMT_YUV420P;

        // Open the codec
        ffmpeg::avcodec_open2(codec_context, codec, ptr::null_mut());
        ffmpeg::avcodec_parameters_from_context((*stream).codecpar, codec_context);

        // Allocate a frame and set its fields
        let mut frame = ffmpeg::av_frame_alloc();
        (*frame).format = ffmpeg::AVPixelFormat::AV_PIX_FMT_YUV420P as i32;
        (*frame).width = 1920;
        (*frame).height = 1080;

        if ffmpeg::av_frame_get_buffer(frame, 32) < 0 {
            eprintln!("Could not allocate frame data");
            ffmpeg::av_frame_free(&mut frame);
            return;
        }

        ffmpeg::av_dump_format(format_context, 0, output_file.as_ptr(), 1);
        ffmpeg::avio_open(
            &mut (*format_context).pb,
            output_file.as_ptr(),
            AVIO_FLAG_WRITE,
        );
        ffmpeg::avformat_write_header(format_context, ptr::null_mut());
        (*frame).pts = 0;
        let mut pkt = ffmpeg::av_packet_alloc();

        if pkt.is_null() {
            eprintln!("Failed to allocate packet");
            ffmpeg::av_frame_free(&mut frame);
            return;
        }
        for i in 0..(25 * 5) {
            // 25 fps * 5 seconds
            (*frame).pts = i;

            // Fill the frame with black pixels
            // For YUV420P, Y plane should be 0, U and V planes should be 128
            let y_plane = (*frame).data[0];
            let u_plane = (*frame).data[1];
            let v_plane = (*frame).data[2];
            let y_line_size = (*frame).linesize[0] as usize;
            let uv_line_size = (*frame).linesize[1] as usize;
            for y in 0..1080 {
                for x in 0..1920 {
                    *y_plane.add(y * y_line_size + x) = 0; // Y plane
                }
            }
            for y in 0..540 {
                for x in 0..960 {
                    *u_plane.add(y * uv_line_size + x) = 128; // U plane
                    *v_plane.add(y * uv_line_size + x) = 128; // V plane
                }
            }

            // Encode the frame
            ffmpeg::av_init_packet(pkt);
            (*pkt).data = ptr::null_mut(); // packet data will be allocated by the encoder
            (*pkt).size = 0;

            if ffmpeg::avcodec_send_frame(codec_context, frame) < 0 {
                eprintln!("Error sending frame for encoding");
                break;
            }

            while ffmpeg::avcodec_receive_packet(codec_context, pkt) == 0 {
                // Write the encoded packet
                if ffmpeg::av_interleaved_write_frame(format_context, pkt) != 0 {
                    eprintln!("Error writing frame");
                    break;
                }
                ffmpeg::av_packet_unref(pkt);
            }

            // Free the packet
            // ffmpeg::av_packet_free(&mut pkt);
        }

        ffmpeg::av_packet_free(&mut pkt);
        ffmpeg::av_frame_free(&mut frame);
        ffmpeg::av_write_trailer(format_context);
        ffmpeg::avcodec_close(codec_context);
        ffmpeg::av_free(frame as *mut libc::c_void);
        ffmpeg::avio_closep(&mut (*format_context).pb);
        ffmpeg::avformat_free_context(format_context);
    }
}
