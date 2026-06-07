# mux-demux

Multiplexing and demultiplexing with channel management, stream identification, and frame parsing.

A zero-dependency Rust library for building multiplexed connections with multiple streams over a single transport.

## Features

- **Channel** — Multi-stream channel management with configurable limits
- **Stream** — Bidirectional streams with TX/RX buffers and state tracking
- **Frame** — Binary frame encoding/decoding with 8-byte headers (type, stream ID, length, flags)
- **Multiplexer** — Encodes stream data into frames for transmission
- **Demultiplexer** — Decodes frames and dispatches to streams
- Frame types: Data, Open/Close/Reset Stream, Window Update, Ping/Pong, Go Away
- Zero external dependencies — pure `std`

## Usage

```rust
use mux_demux::{Multiplexer, Demultiplexer, StreamId};

let mut mux = Multiplexer::new(1, 10);
let sid = mux.open_stream().unwrap();
mux.send(sid, b"hello");
let output = mux.drain_output();

let mut demux = Demultiplexer::new(1, 10);
demux.process(&output);
let mut buf = [0u8; 5];
demux.read(sid, &mut buf);
```

## License

MIT OR Apache-2.0
