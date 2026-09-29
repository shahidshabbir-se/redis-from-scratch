use crate::protocol::resp::types::Frame;
use bytes::{BufMut, BytesMut};

pub fn encode(frame: &Frame, buf: &mut BytesMut) {
    match frame {
        Frame::Simple(s) => {
            buf.put_u8(b'+');
            buf.put_slice(s.as_bytes());
            buf.put_slice(b"\r\n");
        }
        Frame::Error(s) => {
            buf.put_u8(b'-');
            buf.put_slice(s.as_bytes());
            buf.put_slice(b"\r\n");
        }
        Frame::Integer(i) => {
            buf.put_u8(b':');
            buf.put_slice(i.to_string().as_bytes());
            buf.put_slice(b"\r\n");
        }
        Frame::Bulk(bytes) => {
            buf.put_u8(b'$');
            buf.put_slice(bytes.len().to_string().as_bytes());
            buf.put_slice(b"\r\n");
            buf.put_slice(&bytes);
            buf.put_slice(b"\r\n");
        }
        Frame::Null => {
            buf.put_slice(b"$-1\r\n");
        }
        Frame::Array(frames) => {
            buf.put_u8(b'*');
            buf.put_slice(frames.len().to_string().as_bytes());
            buf.put_slice(b"\r\n");
            for frame in frames {
                encode(frame, buf);
            }
        }
    }
}
