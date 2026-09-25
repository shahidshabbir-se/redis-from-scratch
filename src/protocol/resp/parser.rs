use crate::protocol::resp::types::{Error, Frame};
use bytes::{Buf, BytesMut};

fn find_crlf(src: &[u8]) -> Option<usize> {
    src.windows(2).position(|window| window == b"\r\n")
}

pub fn parse(src: &mut BytesMut) -> Result<Frame, Error> {
    if src.is_empty() {
        return Err(Error::Incomplete);
    }

    match src[0] {
        b'+' => match find_crlf(src) {
            Some(idx) => {
                let s = match std::str::from_utf8(&src[1..idx]) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        return Err(Error::Protocol("invalid utf-8".into()));
                    }
                };

                src.advance(idx + 2);

                Ok(Frame::Simple(s))
            }
            None => Err(Error::Incomplete),
        },

        b'-' => match find_crlf(src) {
            Some(idx) => {
                let s = match std::str::from_utf8(&src[1..idx]) {
                    Ok(s) => s.to_string(),
                    Err(_) => {
                        return Err(Error::Protocol("invalid utf-8".into()));
                    }
                };

                src.advance(idx + 2);

                Ok(Frame::Error(s))
            }
            None => Err(Error::Incomplete),
        },

        b':' => match find_crlf(src) {
            Some(idx) => {
                let number = match std::str::from_utf8(&src[1..idx]) {
                    Ok(s) => match s.parse::<i64>() {
                        Ok(number) => number,
                        Err(_) => {
                            return Err(Error::Protocol("invalid integer".into()));
                        }
                    },
                    Err(_) => {
                        return Err(Error::Protocol("invalid utf-8".into()));
                    }
                };

                src.advance(idx + 2);

                Ok(Frame::Integer(number))
            }
            None => Err(Error::Incomplete),
        },

        b'$' => match find_crlf(src) {
            Some(idx) => {
                let length_str = std::str::from_utf8(&src[1..idx])
                    .map_err(|_| Error::Protocol("invalid utf-8".into()))?;
                let length: i64 = length_str
                    .parse()
                    .map_err(|_| Error::Protocol("invalid bulk length".into()))?;

                if length == -1 {
                    src.advance(idx + 2);
                    return Ok(Frame::Null);
                }

                if length < 0 {
                    return Err(Error::Protocol("invalid bulk length".into()));
                }

                let len = length as usize;
                let data_start = idx + 2;
                let total_needed = data_start + len + 2;

                if src.len() < total_needed {
                    return Err(Error::Incomplete);
                }

                // Slice the data!
                let data = bytes::Bytes::copy_from_slice(&src[data_start..data_start + len]);
                src.advance(total_needed);

                Ok(Frame::Bulk(data))
            }

            None => Err(Error::Incomplete),
        },

        b'*' => match find_crlf(src) {
            Some(idx) => {
                let count_str = std::str::from_utf8(&src[1..idx])
                    .map_err(|_| Error::Protocol("invalid utf-8".into()))?;
                let count: i64 = count_str
                    .parse()
                    .map_err(|_| Error::Protocol("invalid array length".into()))?;

                if count == -1 {
                    src.advance(idx + 2);
                    return Ok(Frame::Null);
                }

                if count < 0 {
                    return Err(Error::Protocol("invalid array length".into()));
                }

                let count = count as usize;
                src.advance(idx + 2);

                let mut elements = Vec::with_capacity(count);
                for _ in 0..count {
                    let element = parse(src)?;
                    elements.push(element);
                }

                Ok(Frame::Array(elements))
            }
            None => Err(Error::Incomplete),
        },

        _ => Err(Error::Protocol("unsupported frame type".into())),
    }
}
