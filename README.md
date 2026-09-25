# Redis From Scratch (Rust)

A production-grade, Redis-compatible in-memory data store and RESP protocol implementation built from scratch in Rust using Tokio.

The goal of this project is to understand systems programming fundamentals from first principles: raw network byte streams, zero-copy buffer manipulation, protocol framing, asynchronous runtimes, and concurrent storage architectures—without relying on existing Redis client/server crates.

---

## Architecture & Pipeline

```text
[ TCP Socket / Client (e.g. redis-cli) ]
                    │
                    ▼
           [ Tokio AsyncRead ]
         socket.read_buf(&mut buffer)
                    │
                    ▼
           [ BytesMut Buffer ]
     (Contiguous, growable heap memory)
                    │
                    ▼
          [ RESP2 Framing Parser ]
      parse(&mut BytesMut) -> Result<Frame, Error>
                    │
                    ├── Incomplete ──► Wait for more network bytes (buffer preserved)
                    ├── Protocol   ──► Reject malformed client data
                    └── Ok(Frame)  ──► Advance buffer, dispatch frame
```

---

## Current Progress & Milestones

- [x] **Milestone 1: RESP2 Protocol Framing & Parsing**
  - [x] **Simple Strings (`+`)**: Line-based UTF-8 status replies (`+OK\r\n`).
  - [x] **Errors (`-`)**: Line-based UTF-8 error messages (`-ERR ...\r\n`).
  - [x] **Integers (`:`)**: 64-bit signed numeric values (`:1000\r\n`).
  - [x] **Bulk Strings (`$`)**: Length-prefixed, binary-safe data payloads (`$5\r\nhello\r\n`), including Null Bulk Strings (`$-1\r\n`).
  - [x] **Arrays (`*`)**: Composite and recursive aggregate frames (`*2\r\n$3\r\nGET\r\n$4\r\nuser\r\n`), including Null Arrays (`*-1\r\n`).
  - [x] **Incomplete Frame Handling**: Incremental TCP parsing without buffer corruption on partial reads.
  - [x] **Multiple Frame Handling**: Proper buffer slicing across packet coalescing (multiple commands in one TCP read).
- [x] **Milestone 2: Async Networking Foundation**
  - [x] Tokio-based asynchronous TCP listener bound to `127.0.0.1:6379`.
  - [x] Multi-client concurrent connection lifecycle management using `tokio::spawn`.
  - [x] Zero-copy socket ingestion directly into `BytesMut` with `read_buf`.

---

## Systems Engineering Decisions & Concepts

### 1. TCP Byte Streaming vs. Application Framing
TCP guarantees an ordered, reliable stream of bytes, not discrete messages. An operating system may deliver a single Redis command across multiple TCP packets, or coalesce multiple distinct commands into a single `read` call.
- The parser does not assume one read equals one message.
- If a frame is incomplete, the parser returns `Error::Incomplete` and **leaves the buffer untouched**, allowing the next network read to append data.
- Once a frame is fully parsed, `Buf::advance` consumes only the exact byte range of that frame, leaving any subsequent pipelined commands in the buffer.

### 2. Zero-Copy Buffer Management (`bytes::BytesMut` & `bytes::Bytes`)
- Standard `Vec<u8>` requires expensive heap reallocations or `O(N)` memory copies when slicing frames from the front of a buffer.
- `BytesMut` enables splitting off byte slices and advancing read offsets via pointer arithmetic and reference counting, avoiding unnecessary heap copies on high-throughput network paths.

### 3. Binary Safety in Bulk Strings
- Rust `String` and `&str` require valid UTF-8 by contract.
- Redis Bulk Strings can store arbitrary binary payloads (e.g. compressed data, serialized protocol buffers, images) containing arbitrary bytes such as `\r`, `\n`, or null bytes `\0`.
- Bulk Strings are modeled as `bytes::Bytes` rather than `String`, using length prefixes rather than delimiters to locate boundaries.

### 4. Recursive Array Parsing
- Redis client commands are sent on the wire as Arrays of Bulk Strings (e.g. `*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n`).
- The parser uses bounded recursion: parsing an Array extracts its length, consumes the header, and recursively invokes `parse()` for each element, naturally supporting arbitrarily nested structures.

---

## Verifying Locally

### Run the Server
```bash
cargo run
```

### Connect with Official `redis-cli` or `netcat`
```bash
# Using netcat
printf "+OK\r\n" | nc 127.0.0.1 6379
printf ":100\r\n" | nc 127.0.0.1 6379

# Using official redis-cli
redis-cli ping
redis-cli set mykey hello
redis-cli get mykey
```
The server will parse incoming frames and log the structured representations to standard output.
