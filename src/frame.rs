use bytes::Bytes;
use std::num::ParseIntError;
use tracing::debug;

/// An actual chunk
#[derive(Debug, Clone)]
pub struct Frame {
    body: Vec<u8>,
}

impl TryFrom<Bytes> for Frame {
    type Error = ParseIntError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let mut header = String::new();
        let mut body_it = value.into_iter();
        while let Some(byte) = body_it.next() {
            if byte == b'\r' {
                body_it.next();
                break;
            }
            header.push(byte.into());
        }
        let chunk_len = usize::from_str_radix(&header, 16)?;
        debug!("Chunk length is: {chunk_len:?}");

        let body: Vec<u8> = body_it.take(chunk_len).collect();
        debug!("Frame body: {:?}", String::from_utf8(body.clone()).unwrap());

        Ok(Self { body })
    }
}

impl From<Frame> for Bytes {
    fn from(value: Frame) -> Self {
        let hex_len = format!("{:x}\r\n", value.body.len());
        [hex_len.as_bytes(), &value.body[..], b"\r\n"]
            .concat()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frame_ok() {
        let bytes = Bytes::from_static(b"a\r\n0123456789\r\n");
        assert!(matches!(Frame::try_from(bytes), Ok(_)));
    }

    #[test]
    fn frame_into_bytes() {
        let expected = Bytes::from_static(b"a\r\n0123456789\r\n");
        let frame = Frame {
            body: b"0123456789".to_vec(),
        };
        assert_eq!(Bytes::from(frame), expected);
    }
}
