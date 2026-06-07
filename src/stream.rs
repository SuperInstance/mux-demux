//! Stream identification and state management.

/// Stream identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StreamId(pub u32);

/// Stream state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closed,
    Reset,
}

/// A multiplexed stream.
#[derive(Debug, Clone)]
pub struct Stream {
    pub id: StreamId,
    pub state: StreamState,
    pub tx_buffer: Vec<u8>,
    pub rx_buffer: Vec<u8>,
    pub max_buffer: usize,
}

impl Stream {
    pub fn new(id: StreamId, state: StreamState) -> Self {
        Stream {
            id,
            state,
            tx_buffer: Vec::new(),
            rx_buffer: Vec::new(),
            max_buffer: 65536,
        }
    }

    /// Write data to the transmit buffer.
    pub fn write_tx(&mut self, data: &[u8]) -> usize {
        let available = self.max_buffer - self.tx_buffer.len();
        let to_write = data.len().min(available);
        self.tx_buffer.extend_from_slice(&data[..to_write]);
        to_write
    }

    /// Read data from the transmit buffer.
    pub fn read_tx(&mut self, buf: &mut [u8]) -> usize {
        let to_read = buf.len().min(self.tx_buffer.len());
        buf[..to_read].copy_from_slice(&self.tx_buffer[..to_read]);
        self.tx_buffer.drain(..to_read);
        to_read
    }

    /// Write data to the receive buffer.
    pub fn write_rx(&mut self, data: &[u8]) -> usize {
        let available = self.max_buffer - self.rx_buffer.len();
        let to_write = data.len().min(available);
        self.rx_buffer.extend_from_slice(&data[..to_write]);
        to_write
    }

    /// Read data from the receive buffer.
    pub fn read_rx(&mut self, buf: &mut [u8]) -> usize {
        let to_read = buf.len().min(self.rx_buffer.len());
        buf[..to_read].copy_from_slice(&self.rx_buffer[..to_read]);
        self.rx_buffer.drain(..to_read);
        to_read
    }

    /// Check if the stream is readable.
    pub fn is_readable(&self) -> bool {
        matches!(self.state, StreamState::Open | StreamState::HalfClosedLocal)
            && !self.rx_buffer.is_empty()
    }

    /// Check if the stream is writable.
    pub fn is_writable(&self) -> bool {
        matches!(self.state, StreamState::Open | StreamState::HalfClosedRemote)
    }

    /// Get the raw stream ID value.
    pub fn id_value(&self) -> u32 {
        self.id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_new() {
        let stream = Stream::new(StreamId(1), StreamState::Open);
        assert_eq!(stream.id, StreamId(1));
        assert_eq!(stream.state, StreamState::Open);
    }

    #[test]
    fn test_stream_write_read_tx() {
        let mut stream = Stream::new(StreamId(1), StreamState::Open);
        stream.write_tx(b"hello");
        let mut buf = [0u8; 5];
        assert_eq!(stream.read_tx(&mut buf), 5);
        assert_eq!(&buf, b"hello");
    }

    #[test]
    fn test_stream_write_read_rx() {
        let mut stream = Stream::new(StreamId(1), StreamState::Open);
        stream.write_rx(b"world");
        let mut buf = [0u8; 5];
        assert_eq!(stream.read_rx(&mut buf), 5);
        assert_eq!(&buf, b"world");
    }

    #[test]
    fn test_stream_states() {
        let open = Stream::new(StreamId(1), StreamState::Open);
        assert!(open.is_readable() || open.is_writable()); // writable but no rx data
        assert!(open.is_writable());

        let half = Stream::new(StreamId(2), StreamState::HalfClosedLocal);
        assert!(half.is_writable() == false);
    }

    #[test]
    fn test_stream_buffer_limit() {
        let mut stream = Stream::new(StreamId(1), StreamState::Open);
        stream.max_buffer = 5;
        assert_eq!(stream.write_tx(b"hello world"), 5);
    }
}
