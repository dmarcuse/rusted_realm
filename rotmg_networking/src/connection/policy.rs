//! Handle Flash socket policy file requests
//!
//! Flash clients may request a policy file when opening a socket connection to
//! check whether the server allows the connection. This module provides a
//! function to handle these requests, replying with a policy file which allows
//! connections from any host to any local port from flash clients.

use crate::ext::TcpStreamExt;
use futures::future::Loop;
use futures::{future, Future};
use log::{debug, trace};
use std::io::Error as IoError;
use std::net::Shutdown;
use tokio::io::write_all;
use tokio::net::TcpStream;

/// The binary message denoting a policy file request
pub const POLICY_REQUEST: &[u8] = b"<policy-file-request/>\0";

/// The default policy file, allowing effectively unrestricted access
///
/// This policy file allows connections from all hosts to all ports, and allows
/// policy files from other ports as well.
pub const POLICY_FILE: &[u8] = br#"
<?xml version="1.0"?>
<!DOCTYPE cross-domain-policy SYSTEM "/xml/dtds/cross-domain-policy.dtd">
<cross-domain-policy>
    <site-control permitted-cross-domain-policies="all"/>
    <allow-access-from domain="*" to-ports="*"/>
</cross-domain-policy>
"#;

/// Peek into the given `TcpStream` to detect whether this is a policy file
/// request and handle appropriately.
///
/// `None` will be returned when a policy file request is detected and handled.
/// `Some(TcpStream)` will be returned when a regular connection is detected;
/// all data received from the stream will remain in the buffer.
pub fn handle_policy_request(
    stream: TcpStream,
) -> impl Future<Item = Option<TcpStream>, Error = IoError> {
    future::loop_fn(
        (stream, vec![]),
        move |(stream, bytes)| -> Box<dyn Future<Item = _, Error = _> + Send> {
            if &bytes[..] == POLICY_REQUEST {
                // this is definitely a policy file request
                // send the policy file, then shutdown the socket and break with
                // none to indicate that this wasn't a game connection
                debug!("Sending policy file to {}", stream.peer_addr().unwrap());

                Box::new(
                    write_all(stream, POLICY_FILE)
                        .and_then(|(stream, _)| stream.shutdown(Shutdown::Both))
                        .map(|_| Loop::Break(None)),
                )
            } else if POLICY_REQUEST.starts_with(&bytes[..]) {
                trace!("Potential policy file request: {:?}", bytes);

                // this may be a policy file request, but we need more bytes
                Box::new(stream.peek_max(POLICY_REQUEST.len()).map(Loop::Continue))
            } else {
                trace!("Not a policy file request: {:?}", bytes);

                // this is not a policy file request
                Box::new(future::ok(Loop::Break(Some(stream))))
            }
        },
    )
}
