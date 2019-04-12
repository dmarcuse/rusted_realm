//! Internal extensions to third party APIs for convenience

use futures::{try_ready, Async, Future, Poll};
use std::mem::replace;
use tokio::io::Error as IoError;
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct PeekMax {
    stream: Option<TcpStream>,
    max: usize,
}

impl Future for PeekMax {
    type Item = (TcpStream, Vec<u8>);
    type Error = IoError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(ref mut stream) = self.stream {
            // allocate a vector to store results
            let mut bytes = vec![0u8; self.max];

            // attempt to poll the stream
            let bytes_read = try_ready!(stream.poll_peek(&mut bytes[..]));

            // on success, trim to the total bytes read...
            bytes.truncate(bytes_read);

            // ...then return the stream and the bytes
            return Ok(Async::Ready((
                replace(&mut self.stream, None).unwrap(),
                bytes,
            )));
        } else {
            panic!("polled a PeekMax after it's done");
        }
    }
}

/// Extensions for a tokio `TcpStream`
pub trait TcpStreamExt {
    /// Asynchronously peek at up to `max` bytes from this stream, leaving them
    /// in the buffer
    fn peek_max(self, max: usize) -> PeekMax;
}

impl TcpStreamExt for TcpStream {
    fn peek_max(self, max: usize) -> PeekMax {
        PeekMax {
            stream: Some(self),
            max,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Stream;
    use std::net::SocketAddr;
    use std::str::FromStr;
    use tokio::io::write_all;
    use tokio::net::TcpListener;

    #[test]
    fn test_peek() {
        let address = SocketAddr::from_str("127.0.0.1:2050").unwrap();

        // start a listener that accepts one connection and peeks at incoming
        // data twice to ensure it's working properly
        let server = TcpListener::bind(&address)
            .unwrap()
            .incoming()
            .take(1)
            .for_each(|s| {
                s.peek_max(4)
                    .and_then(|(stream, bytes)| {
                        if &bytes[..] == b"abcd" {
                            Ok(stream)
                        } else {
                            panic!("Unexpected data: {:x?}", &bytes[..]);
                        }
                    })
                    .and_then(|s| s.peek_max(4))
                    .and_then(|(_stream, bytes)| {
                        if &bytes[..] == b"abcd" {
                            Ok(())
                        } else {
                            panic!("Unexpected data: {:x?}", &bytes[..]);
                        }
                    })
            });

        // start a client which connects to the server and sends the expected data
        let client = TcpStream::connect(&address).and_then(|s| write_all(s, b"abcd"));;

        // start them together
        tokio::run(
            server
                .join(client)
                .map(|_| {})
                .map_err(|e| panic!("error: {:?}", e)),
        );
    }
}
