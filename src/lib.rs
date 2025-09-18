// Imports
use heapless::Deque;

// ---- SBUS constants (type-level usable) ----
const FRAME_LENGTH: usize = 25;
const SBUS_START_BYTE: u8 = 0x0F;

/// Strongly typed SBUS channel container
pub struct Channels([u16; 16]);
impl Channels {
    /// Always zero-initialized
    pub fn new() -> Self {
        Self([0u16; 16])
    }

    /// Internal (crate-only) mutable access for driver
    pub(crate) fn update(&mut self, new: [u16; 16]) {
        self.0 = new;
    }

    /// Public read-only access
    pub fn channel(&self, idx: usize) -> Option<u16> {
        self.0.get(idx).copied()
    }
}


// #### SBUS Parser Implementation ####
pub struct SbusReceiver {
    buffer: Deque<u8, FRAME_LENGTH>,
}
impl SbusReceiver {
    pub fn new() -> Self { Self { buffer: Deque::new() } }

    /// Push a byte; return true when a full frame is ready.
    pub fn process_byte(&mut self, byte: u8) -> bool {
        if self.buffer.is_empty() && byte != SBUS_START_BYTE { return false; }
        if self.buffer.push_back(byte).is_err() {
            self.buffer.pop_front();
            let _ = self.buffer.push_back(byte);
        }
        self.buffer.len() >= FRAME_LENGTH
    }

    /// Pop a complete frame from the front.
    pub fn take_frame(&mut self) -> Option<[u8; FRAME_LENGTH]> {
        if self.buffer.len() < FRAME_LENGTH { return None; }
        let mut frame = [0u8; FRAME_LENGTH];
        for b in &mut frame { *b = self.buffer.pop_front().unwrap(); }
        Some(frame)
    }

    pub fn validate_checksum(frame: &[u8; FRAME_LENGTH]) -> bool {
        let cs: u8 = frame.iter().take(23).fold(0, |a, &b| a ^ b);
        cs == frame[23]
    }

    pub fn extract_channels(frame: &[u8; FRAME_LENGTH], out: &mut Channels) {
        let mut ch = [0u16; 16];
        for i in 0..16 {
            let b1 = frame[i * 2 + 1] as u16;
            let b2 = frame[i * 2 + 2] as u16;
            ch[i] = (b1 << 8) | b2;
        }
        out.update(ch); // only driver can update
    }

    pub fn scale_1000_2000(&self, ch: &[u16; 16]) -> [u16; 16] {
        let mut out = [1500u16; 16];
        for (i, &v) in ch.iter().enumerate() {
            let s = 1000 + (v as i32 * 1000) / 2047;
            out[i] = s.clamp(1000, 2000) as u16;
        }
        out
    }
}