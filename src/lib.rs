extern crate futures;
extern crate tokio_core;

use std::io::Cursor;
use std::io::{Read, Write};
use std::io::Result as IoResult;
use tokio_core::io::Io;

pub struct BufStream<T>(Cursor<T>);

impl Default for BufStream<Vec<u8>> {
    fn default() -> BufStream<Vec<u8>> {
        BufStream::new(Vec::new())
    }
}

impl<T> BufStream<T> {
    pub fn new(buf: T) -> BufStream<T> {
        BufStream(Cursor::new(buf))
    }

    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
}

impl<T> Read for BufStream<T> where T: AsRef<[u8]> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.0.read(buf)
    }
}

impl Write for BufStream<Vec<u8>> {
     fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
         self.0.write(buf)
     }

     fn flush(&mut self) -> IoResult<()> {
         self.0.flush()
     }
}

impl Io for BufStream<Vec<u8>> {}

impl<'a> Write for BufStream<&'a mut [u8]> {
     fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
         self.0.write(buf)
     }

     fn flush(&mut self) -> IoResult<()> {
         self.0.flush()
     }
}

impl<'a> Io for BufStream<&'a mut [u8]> {}

impl Write for BufStream<Box<[u8]>> {
     fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
         self.0.write(buf)
     }

     fn flush(&mut self) -> IoResult<()> {
         self.0.flush()
     }
}

impl<'a> Io for BufStream<Box<[u8]>> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Result as IoResult;
    use futures::Stream;
    use futures::Future;
    use futures::Sink;
    use tokio_core::io::Codec;
    use tokio_core::io::EasyBuf;
    use tokio_core::io::Io;

    struct Byte;

    impl Codec for Byte {
        type In = u8;
        type Out = String;

        fn decode(&mut self, buf: &mut EasyBuf) -> IoResult<Option<Self::In>> {
            let buf = buf.drain_to(1);
            Ok(Some(buf.as_slice()[0]))
        }
        fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> IoResult<()> {
            buf.extend_from_slice(msg.as_bytes());
            Ok(())
        }
    }

    #[test]
    fn read_via_framed() {
        let result = BufStream::new(Vec::from("hello world"))
            .framed(Byte)
            .collect();

        assert_eq!(result.wait().expect("Ok result"), vec![104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]);
    }

    #[test]
    fn read_via_framed_mut_slice() {
        let mut buf = Vec::from("hello world");
        let result = BufStream::new(buf.as_mut_slice())
            .framed(Byte)
            .collect();

        assert_eq!(result.wait().expect("Ok result"), vec![104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]);
    }

    #[test]
    fn write_via_framed() {
        let result = BufStream::new(Vec::new())
            .framed(Byte)
            .send("hello world".to_string());

        let encoder = result.wait().expect("Ok result").into_inner();
        assert_eq!(encoder.into_inner(), vec![104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]);

    }

    #[test]
    fn write_via_framed_mut_slice() {
        let mut buf = vec![0; 11];
        let result = BufStream::new(buf.as_mut_slice())
            .framed(Byte)
            .send("hello world".to_string());

        let encoder = result.wait().expect("Ok result").into_inner();
        assert_eq!(encoder.into_inner(), [104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100]);
    }
}
