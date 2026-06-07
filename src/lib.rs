//! # mux-demux
//!
//! Multiplexing and demultiplexing with channel management, stream identification, and frame parsing.

pub mod channel;
pub mod stream;
pub mod frame;
pub mod multiplexer;
pub mod demultiplexer;

pub use channel::Channel;
pub use stream::{Stream, StreamId, StreamState};
pub use frame::{Frame, FrameType, FrameHeader};
pub use multiplexer::Multiplexer;
pub use demultiplexer::Demultiplexer;
