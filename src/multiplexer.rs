//! Multiplexer: encodes and sends frames for multiple streams.

use crate::channel::Channel;
use crate::frame::Frame;
use crate::stream::StreamId;

/// Multiplexer that takes data from streams and produces frames.
#[derive(Debug)]
pub struct Multiplexer {
    pub channel: Channel,
    pub output_buffer: Vec<u8>,
}

impl Multiplexer {
    pub fn new(channel_id: u32, max_streams: usize) -> Self {
        Multiplexer {
            channel: Channel::new(channel_id, max_streams),
            output_buffer: Vec::new(),
        }
    }

    /// Open a new stream. Returns the stream ID.
    pub fn open_stream(&mut self) -> Option<StreamId> {
        let sid = self.channel.create_stream()?;
        let frame = Frame::open_stream(sid.0);
        self.output_buffer.extend(frame.encode());
        Some(sid)
    }

    /// Send data on a stream.
    pub fn send(&mut self, stream_id: StreamId, data: &[u8]) -> bool {
        let stream = match self.channel.get_stream_mut(stream_id) {
            Some(s) => s,
            None => return false,
        };
        if !stream.is_writable() {
            return false;
        }
        let written = stream.write_tx(data);
        if written > 0 {
            let payload: Vec<u8> = stream.tx_buffer.drain(..written).collect();
            let frame = Frame::data(stream_id.0, payload);
            self.output_buffer.extend(frame.encode());
            true
        } else {
            false
        }
    }

    /// Close a stream.
    pub fn close_stream(&mut self, stream_id: StreamId) -> bool {
        if self.channel.close_stream(stream_id) {
            let frame = Frame::close_stream(stream_id.0);
            self.output_buffer.extend(frame.encode());
            true
        } else {
            false
        }
    }

    /// Send a ping.
    pub fn ping(&mut self, opaque: u32) {
        let frame = Frame::ping(opaque);
        self.output_buffer.extend(frame.encode());
    }

    /// Drain the output buffer.
    pub fn drain_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.output_buffer)
    }

    /// Number of pending output bytes.
    pub fn pending_bytes(&self) -> usize {
        self.output_buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FrameType;

    #[test]
    fn test_multiplexer_open_stream() {
        let mut mux = Multiplexer::new(1, 10);
        let sid = mux.open_stream().unwrap();
        assert_eq!(sid.0, 1);
        assert!(mux.pending_bytes() > 0);
    }

    #[test]
    fn test_multiplexer_send_data() {
        let mut mux = Multiplexer::new(1, 10);
        let sid = mux.open_stream().unwrap();
        mux.drain_output(); // clear open frame
        assert!(mux.send(sid, b"hello"));
        let output = mux.drain_output();
        let (frame, _) = Frame::decode(&output).unwrap();
        assert_eq!(frame.header.frame_type, FrameType::Data);
        assert_eq!(frame.payload, b"hello");
    }

    #[test]
    fn test_multiplexer_close_stream() {
        let mut mux = Multiplexer::new(1, 10);
        let sid = mux.open_stream().unwrap();
        mux.drain_output();
        assert!(mux.close_stream(sid));
        let output = mux.drain_output();
        let (frame, _) = Frame::decode(&output).unwrap();
        assert_eq!(frame.header.frame_type, FrameType::CloseStream);
    }

    #[test]
    fn test_multiplexer_send_nonexistent() {
        let mut mux = Multiplexer::new(1, 10);
        assert!(!mux.send(StreamId(99), b"hello"));
    }

    #[test]
    fn test_multiplexer_ping() {
        let mut mux = Multiplexer::new(1, 10);
        mux.ping(42);
        let output = mux.drain_output();
        let (frame, _) = Frame::decode(&output).unwrap();
        assert_eq!(frame.header.frame_type, FrameType::Ping);
    }
}
