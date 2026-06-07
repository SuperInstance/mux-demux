//! Frame encoding and decoding.

/// Frame types for the multiplexing protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Data,
    OpenStream,
    CloseStream,
    ResetStream,
    WindowUpdate,
    Ping,
    Pong,
    GoAway,
}

impl FrameType {
    /// Encode frame type as a byte.
    pub fn to_byte(self) -> u8 {
        match self {
            FrameType::Data => 0,
            FrameType::OpenStream => 1,
            FrameType::CloseStream => 2,
            FrameType::ResetStream => 3,
            FrameType::WindowUpdate => 4,
            FrameType::Ping => 5,
            FrameType::Pong => 6,
            FrameType::GoAway => 7,
        }
    }

    /// Decode frame type from a byte.
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(FrameType::Data),
            1 => Some(FrameType::OpenStream),
            2 => Some(FrameType::CloseStream),
            3 => Some(FrameType::ResetStream),
            4 => Some(FrameType::WindowUpdate),
            5 => Some(FrameType::Ping),
            6 => Some(FrameType::Pong),
            7 => Some(FrameType::GoAway),
            _ => None,
        }
    }
}

/// Frame header (8 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameHeader {
    pub frame_type: FrameType,
    pub stream_id: u32,
    pub length: u16,
    pub flags: u8,
}

impl FrameHeader {
    pub const SIZE: usize = 8;

    /// Parse a frame header from bytes.
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        let frame_type = FrameType::from_byte(data[0])?;
        Some(FrameHeader {
            frame_type,
            stream_id: u32::from_be_bytes([data[1], data[2], data[3], data[4]]),
            length: u16::from_be_bytes([data[5], data[6]]),
            flags: data[7],
        })
    }

    /// Serialize the header to bytes.
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0] = self.frame_type.to_byte();
        buf[1..5].copy_from_slice(&self.stream_id.to_be_bytes());
        buf[5..7].copy_from_slice(&self.length.to_be_bytes());
        buf[7] = self.flags;
        buf
    }
}

/// A complete frame with header and payload.
#[derive(Debug, Clone)]
pub struct Frame {
    pub header: FrameHeader,
    pub payload: Vec<u8>,
}

impl Frame {
    /// Create a new data frame.
    pub fn data(stream_id: u32, payload: Vec<u8>) -> Self {
        Frame {
            header: FrameHeader {
                frame_type: FrameType::Data,
                stream_id,
                length: payload.len() as u16,
                flags: 0,
            },
            payload,
        }
    }

    /// Create an open stream frame.
    pub fn open_stream(stream_id: u32) -> Self {
        Frame {
            header: FrameHeader {
                frame_type: FrameType::OpenStream,
                stream_id,
                length: 0,
                flags: 0,
            },
            payload: Vec::new(),
        }
    }

    /// Create a close stream frame.
    pub fn close_stream(stream_id: u32) -> Self {
        Frame {
            header: FrameHeader {
                frame_type: FrameType::CloseStream,
                stream_id,
                length: 0,
                flags: 0,
            },
            payload: Vec::new(),
        }
    }

    /// Create a window update frame.
    pub fn window_update(stream_id: u32, increment: u32) -> Self {
        Frame {
            header: FrameHeader {
                frame_type: FrameType::WindowUpdate,
                stream_id,
                length: 4,
                flags: 0,
            },
            payload: increment.to_be_bytes().to_vec(),
        }
    }

    /// Create a ping frame.
    pub fn ping(opaque: u32) -> Self {
        Frame {
            header: FrameHeader {
                frame_type: FrameType::Ping,
                stream_id: 0,
                length: 4,
                flags: 0,
            },
            payload: opaque.to_be_bytes().to_vec(),
        }
    }

    /// Create a pong response.
    pub fn pong(opaque: u32) -> Self {
        Frame {
            header: FrameHeader {
                frame_type: FrameType::Pong,
                stream_id: 0,
                length: 4,
                flags: 0,
            },
            payload: opaque.to_be_bytes().to_vec(),
        }
    }

    /// Create a go-away frame.
    pub fn goaway(last_stream_id: u32) -> Self {
        Frame {
            header: FrameHeader {
                frame_type: FrameType::GoAway,
                stream_id: 0,
                length: 4,
                flags: 0,
            },
            payload: last_stream_id.to_be_bytes().to_vec(),
        }
    }

    /// Encode the complete frame to bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(FrameHeader::SIZE + self.payload.len());
        buf.extend_from_slice(&self.header.to_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Decode a frame from bytes. Returns the frame and number of bytes consumed.
    pub fn decode(data: &[u8]) -> Option<(Self, usize)> {
        let header = FrameHeader::parse(data)?;
        let total = FrameHeader::SIZE + header.length as usize;
        if data.len() < total {
            return None;
        }
        let payload = data[FrameHeader::SIZE..total].to_vec();
        Some((Frame { header, payload }, total))
    }

    /// Check if this is a data frame.
    pub fn is_data(&self) -> bool {
        self.header.frame_type == FrameType::Data
    }

    /// Check if this is a control frame.
    pub fn is_control(&self) -> bool {
        !self.is_data()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_type_roundtrip() {
        for ft in [
            FrameType::Data,
            FrameType::OpenStream,
            FrameType::CloseStream,
            FrameType::WindowUpdate,
            FrameType::Ping,
            FrameType::GoAway,
        ] {
            assert_eq!(FrameType::from_byte(ft.to_byte()), Some(ft));
        }
    }

    #[test]
    fn test_frame_header_roundtrip() {
        let hdr = FrameHeader {
            frame_type: FrameType::Data,
            stream_id: 42,
            length: 100,
            flags: 0xFF,
        };
        let bytes = hdr.to_bytes();
        let parsed = FrameHeader::parse(&bytes).unwrap();
        assert_eq!(parsed, hdr);
    }

    #[test]
    fn test_frame_encode_decode() {
        let frame = Frame::data(1, b"hello world".to_vec());
        let encoded = frame.encode();
        let (decoded, consumed) = Frame::decode(&encoded).unwrap();
        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded.header.stream_id, 1);
        assert_eq!(decoded.payload, b"hello world");
    }

    #[test]
    fn test_frame_open_stream() {
        let frame = Frame::open_stream(5);
        assert_eq!(frame.header.frame_type, FrameType::OpenStream);
        assert!(frame.is_control());
    }

    #[test]
    fn test_frame_window_update() {
        let frame = Frame::window_update(1, 1024);
        let (decoded, _) = Frame::decode(&frame.encode()).unwrap();
        let inc = u32::from_be_bytes([
            decoded.payload[0], decoded.payload[1],
            decoded.payload[2], decoded.payload[3],
        ]);
        assert_eq!(inc, 1024);
    }

    #[test]
    fn test_frame_decode_truncated() {
        let data = [0u8; 4]; // too short
        assert!(Frame::decode(&data).is_none());
    }

    #[test]
    fn test_frame_ping_pong() {
        let ping = Frame::ping(12345);
        let pong = Frame::pong(12345);
        assert_eq!(ping.header.frame_type, FrameType::Ping);
        assert_eq!(pong.header.frame_type, FrameType::Pong);
        assert_eq!(ping.payload, pong.payload);
    }

    #[test]
    fn test_frame_goaway() {
        let frame = Frame::goaway(99);
        let encoded = frame.encode();
        let (decoded, n) = Frame::decode(&encoded).unwrap();
        assert_eq!(n, encoded.len());
        assert_eq!(decoded.header.frame_type, FrameType::GoAway);
        let last = u32::from_be_bytes(decoded.payload.try_into().unwrap());
        assert_eq!(last, 99);
    }
}
