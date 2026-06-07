//! Demultiplexer: receives frames and dispatches to streams.

use crate::channel::Channel;
use crate::frame::{Frame, FrameType};
use crate::stream::{StreamId, StreamState};

/// Demultiplexer that receives frames and dispatches data to streams.
#[derive(Debug)]
pub struct Demultiplexer {
    pub channel: Channel,
    pub pending_pongs: Vec<u32>,
    pub goaway_received: bool,
    pub last_stream_id: u32,
}

impl Demultiplexer {
    pub fn new(channel_id: u32, max_streams: usize) -> Self {
        Demultiplexer {
            channel: Channel::new(channel_id, max_streams),
            pending_pongs: Vec::new(),
            goaway_received: false,
            last_stream_id: 0,
        }
    }

    /// Process incoming bytes, returning the number of bytes consumed.
    pub fn process(&mut self, data: &[u8]) -> usize {
        let mut offset = 0;
        while offset < data.len() {
            match Frame::decode(&data[offset..]) {
                Some((frame, consumed)) => {
                    self.handle_frame(frame);
                    offset += consumed;
                }
                None => break,
            }
        }
        offset
    }

    /// Handle a single decoded frame.
    pub fn handle_frame(&mut self, frame: Frame) {
        match frame.header.frame_type {
            FrameType::Data => {
                if let Some(stream) = self.channel.get_stream_mut(StreamId(frame.header.stream_id)) {
                    stream.write_rx(&frame.payload);
                }
            }
            FrameType::OpenStream => {
                let sid = StreamId(frame.header.stream_id);
                if self.channel.get_stream(sid).is_none() {
                    self.channel.streams.insert(
                        sid,
                        crate::stream::Stream::new(sid, StreamState::Open),
                    );
                    self.last_stream_id = sid.0;
                }
            }
            FrameType::CloseStream => {
                self.channel.close_stream(StreamId(frame.header.stream_id));
            }
            FrameType::ResetStream => {
                self.channel.remove_stream(StreamId(frame.header.stream_id));
            }
            FrameType::WindowUpdate => {
                // Process window update (would update credit system)
            }
            FrameType::Ping => {
                // Respond with pong
                let opaque = if frame.payload.len() >= 4 {
                    u32::from_be_bytes([
                        frame.payload[0], frame.payload[1],
                        frame.payload[2], frame.payload[3],
                    ])
                } else {
                    0
                };
                self.pending_pongs.push(opaque);
            }
            FrameType::Pong => {
                // Acknowledge pong
            }
            FrameType::GoAway => {
                self.goaway_received = true;
                if frame.payload.len() >= 4 {
                    self.last_stream_id = u32::from_be_bytes([
                        frame.payload[0], frame.payload[1],
                        frame.payload[2], frame.payload[3],
                    ]);
                }
            }
        }
    }

    /// Read data from a specific stream.
    pub fn read(&mut self, stream_id: StreamId, buf: &mut [u8]) -> usize {
        if let Some(stream) = self.channel.get_stream_mut(stream_id) {
            stream.read_rx(buf)
        } else {
            0
        }
    }

    /// Check if a stream has data available.
    pub fn has_data(&self, stream_id: StreamId) -> bool {
        self.channel.get_stream(stream_id).is_some_and(|s| s.is_readable())
    }

    /// Take the next pending pong value.
    pub fn take_pong(&mut self) -> Option<u32> {
        self.pending_pongs.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demux_process_data() {
        let mut demux = Demultiplexer::new(1, 10);
        // First open a stream
        let open = Frame::open_stream(1);
        demux.handle_frame(open);
        // Then send data
        let data = Frame::data(1, b"hello".to_vec());
        demux.handle_frame(data);
        let mut buf = [0u8; 5];
        assert_eq!(demux.read(StreamId(1), &mut buf), 5);
        assert_eq!(&buf, b"hello");
    }

    #[test]
    fn test_demux_process_bytes() {
        let mut demux = Demultiplexer::new(1, 10);
        let open = Frame::open_stream(1);
        let data = Frame::data(1, b"world".to_vec());
        let mut bytes = Vec::new();
        bytes.extend(open.encode());
        bytes.extend(data.encode());
        let consumed = demux.process(&bytes);
        assert_eq!(consumed, bytes.len());
        let mut buf = [0u8; 5];
        assert_eq!(demux.read(StreamId(1), &mut buf), 5);
        assert_eq!(&buf, b"world");
    }

    #[test]
    fn test_demux_close_stream() {
        let mut demux = Demultiplexer::new(1, 10);
        demux.handle_frame(Frame::open_stream(1));
        demux.handle_frame(Frame::close_stream(1));
        let stream = demux.channel.get_stream(StreamId(1)).unwrap();
        assert_eq!(stream.state, StreamState::Closed);
    }

    #[test]
    fn test_demux_reset_stream() {
        let mut demux = Demultiplexer::new(1, 10);
        demux.handle_frame(Frame::open_stream(1));
        demux.handle_frame(Frame {
            header: crate::frame::FrameHeader {
                frame_type: FrameType::ResetStream,
                stream_id: 1,
                length: 0,
                flags: 0,
            },
            payload: Vec::new(),
        });
        assert!(demux.channel.get_stream(StreamId(1)).is_none());
    }

    #[test]
    fn test_demux_ping_pong() {
        let mut demux = Demultiplexer::new(1, 10);
        demux.handle_frame(Frame::ping(42));
        assert_eq!(demux.take_pong(), Some(42));
    }

    #[test]
    fn test_demux_goaway() {
        let mut demux = Demultiplexer::new(1, 10);
        demux.handle_frame(Frame::goaway(99));
        assert!(demux.goaway_received);
        assert_eq!(demux.last_stream_id, 99);
    }

    #[test]
    fn test_demux_has_data() {
        let demux = Demultiplexer::new(1, 10);
        assert!(!demux.has_data(StreamId(1)));
    }
}
