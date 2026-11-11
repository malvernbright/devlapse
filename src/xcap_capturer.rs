//! Real screen capture via `xcap` (Windows, macOS, Linux X11/Wayland).

use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, RgbaImage};
use xcap::Monitor;

use crate::capture::{Frame, ScreenCapturer};
use crate::config::ImageFormat;

pub const JPEG_QUALITY: u8 = 85;

pub struct XcapCapturer {
    format: ImageFormat,
}

impl XcapCapturer {
    pub fn new(format: ImageFormat) -> Self {
        Self { format }
    }
}

impl ScreenCapturer for XcapCapturer {
    fn capture(&mut self) -> Result<Vec<Frame>, String> {
        let monitors = Monitor::all().map_err(|e| format!("could not list monitors: {e}"))?;
        if monitors.is_empty() {
            return Err("no monitors found".into());
        }
        let mut frames = Vec::with_capacity(monitors.len());
        let mut failures = Vec::new();
        for (i, m) in monitors.iter().enumerate() {
            match m.capture_image().map_err(|e| e.to_string()).and_then(|img| encode(&img, self.format)) {
                Ok(bytes) => frames.push(Frame { monitor: i, bytes }),
                Err(e) => failures.push(format!("screen{}: {e}", i + 1)),
            }
        }
        // One bad monitor shouldn't cost us the others; only fail if nothing worked.
        if frames.is_empty() {
            return Err(failures.join("; "));
        }
        for f in failures {
            eprintln!("warning: {f}");
        }
        Ok(frames)
    }
}

pub fn encode(img: &RgbaImage, format: ImageFormat) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    match format {
        ImageFormat::Png => DynamicImage::ImageRgba8(img.clone()).write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png),
        // JPEG has no alpha channel; drop it first.
        ImageFormat::Jpeg => DynamicImage::ImageRgba8(img.clone())
            .to_rgb8()
            .write_with_encoder(JpegEncoder::new_with_quality(&mut buf, JPEG_QUALITY)),
    }
    .map_err(|e| format!("encode failed: {e}"))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RgbaImage {
        RgbaImage::from_fn(64, 32, |x, y| image::Rgba([x as u8 * 4, y as u8 * 8, 128, 255]))
    }

    #[test]
    fn png_encoding_produces_a_valid_png() {
        let bytes = encode(&sample(), ImageFormat::Png).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        let back = image::load_from_memory(&bytes).unwrap();
        assert_eq!((back.width(), back.height()), (64, 32));
    }

    #[test]
    fn jpeg_encoding_produces_a_valid_jpeg_despite_alpha() {
        let bytes = encode(&sample(), ImageFormat::Jpeg).unwrap();
        assert_eq!(&bytes[..2], &[0xFF, 0xD8]);
        let back = image::load_from_memory(&bytes).unwrap();
        assert_eq!((back.width(), back.height()), (64, 32));
    }

    /// Touches the real display. Run on your desktop with: cargo test -- --ignored
    #[test]
    #[ignore]
    fn captures_the_real_screen() {
        let frames = XcapCapturer::new(ImageFormat::Jpeg).capture().expect("capture should work on a desktop session");
        assert!(!frames.is_empty());
        for f in &frames {
            let img = image::load_from_memory(&f.bytes).unwrap();
            assert!(img.width() > 100 && img.height() > 100, "screen{} looks too small", f.monitor + 1);
        }
    }
}
