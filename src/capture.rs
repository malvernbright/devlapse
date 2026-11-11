/// One encoded screenshot of one monitor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Zero-based monitor index.
    pub monitor: usize,
    /// Encoded image bytes (PNG or JPEG), ready to write to disk.
    pub bytes: Vec<u8>,
}

/// Anything that can grab the screens. Real impl: `XcapCapturer`; tests use fakes.
pub trait ScreenCapturer {
    fn capture(&mut self) -> Result<Vec<Frame>, String>;
}
