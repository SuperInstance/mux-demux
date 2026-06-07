//! Channel management for multiplexed connections.

use crate::stream::{Stream, StreamId, StreamState};
use std::collections::HashMap;

/// A multiplexed channel containing multiple streams.
#[derive(Debug)]
pub struct Channel {
    pub id: u32,
    pub streams: HashMap<StreamId, Stream>,
    pub max_streams: usize,
    pub active: bool,
}

impl Channel {
    pub fn new(id: u32, max_streams: usize) -> Self {
        Channel {
            id,
            streams: HashMap::new(),
            max_streams,
            active: true,
        }
    }

    /// Create a new stream in this channel.
    pub fn create_stream(&mut self) -> Option<StreamId> {
        if self.streams.len() >= self.max_streams || !self.active {
            return None;
        }
        let id = self.next_stream_id();
        let stream = Stream::new(id, StreamState::Open);
        self.streams.insert(id, stream);
        Some(id)
    }

    /// Get the next available stream ID.
    fn next_stream_id(&self) -> StreamId {
        let max_id = self.streams.keys().max().map_or(0, |k| k.0);
        StreamId(max_id + 1)
    }

    /// Close a stream.
    pub fn close_stream(&mut self, id: StreamId) -> bool {
        if let Some(stream) = self.streams.get_mut(&id) {
            stream.state = StreamState::Closed;
            true
        } else {
            false
        }
    }

    /// Remove a stream entirely.
    pub fn remove_stream(&mut self, id: StreamId) -> Option<Stream> {
        self.streams.remove(&id)
    }

    /// Get a stream by ID.
    pub fn get_stream(&self, id: StreamId) -> Option<&Stream> {
        self.streams.get(&id)
    }

    /// Get a mutable stream by ID.
    pub fn get_stream_mut(&mut self, id: StreamId) -> Option<&mut Stream> {
        self.streams.get_mut(&id)
    }

    /// Number of open streams.
    pub fn open_stream_count(&self) -> usize {
        self.streams.values().filter(|s| s.state == StreamState::Open).count()
    }

    /// Total streams (including closed).
    pub fn total_stream_count(&self) -> usize {
        self.streams.len()
    }

    /// Close the entire channel.
    pub fn close(&mut self) {
        self.active = false;
        for stream in self.streams.values_mut() {
            stream.state = StreamState::Closed;
        }
    }

    /// Check if the channel is active.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_create() {
        let ch = Channel::new(1, 10);
        assert!(ch.is_active());
        assert_eq!(ch.open_stream_count(), 0);
    }

    #[test]
    fn test_channel_create_stream() {
        let mut ch = Channel::new(1, 10);
        let sid = ch.create_stream().unwrap();
        assert_eq!(ch.open_stream_count(), 1);
        let stream = ch.get_stream(sid).unwrap();
        assert_eq!(stream.state, StreamState::Open);
    }

    #[test]
    fn test_channel_max_streams() {
        let mut ch = Channel::new(1, 2);
        ch.create_stream();
        ch.create_stream();
        assert!(ch.create_stream().is_none());
    }

    #[test]
    fn test_channel_close_stream() {
        let mut ch = Channel::new(1, 10);
        let sid = ch.create_stream().unwrap();
        assert!(ch.close_stream(sid));
        assert_eq!(ch.open_stream_count(), 0);
        assert_eq!(ch.total_stream_count(), 1);
    }

    #[test]
    fn test_channel_close() {
        let mut ch = Channel::new(1, 10);
        ch.create_stream();
        ch.create_stream();
        ch.close();
        assert!(!ch.is_active());
        assert_eq!(ch.open_stream_count(), 0);
    }

    #[test]
    fn test_channel_remove_stream() {
        let mut ch = Channel::new(1, 10);
        let sid = ch.create_stream().unwrap();
        let removed = ch.remove_stream(sid).unwrap();
        assert_eq!(removed.state, StreamState::Open);
        assert_eq!(ch.total_stream_count(), 0);
    }
}
