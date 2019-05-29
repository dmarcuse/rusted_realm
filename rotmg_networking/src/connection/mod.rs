//! Implementations of client and server connections
//!
//! This module provides functions (`client_listener` and `server_connection`)
//! to create low-level tokio streams operating on `RawPacket` instances. This
//! allows acting as either a ROTMG server or client (or even both at once).
//!
//! Submodules of this module expose the code which is used to implement these
//! utility functions, in case you want to do something even more low-level or
//! customize the behavior.

pub mod codec;
pub mod policy;
pub mod raw_packet;

use self::codec::Codec;
use self::policy::handle_policy_request;
use futures::{Future, Stream};
use rotmg_packets::mappings::Mappings;
use std::convert::identity;
use std::io::{Error as IoError, Result as IoResult};
use std::net::SocketAddr;
use tokio::codec::{Decoder, Framed};
use tokio::net::{TcpListener, TcpStream};

/// A framed TCP connection that operates on `RawPacket` instances
pub type Connection = Framed<TcpStream, Codec>;

/// Configure a stream for either client or server communication
fn configure_stream(s: TcpStream) -> IoResult<TcpStream> {
    s.set_nodelay(true)?;

    Ok(s)
}

/// Start a listener accepting ROTMG client connections on the given socket
/// address, using the given mappings.
///
/// A stream of framed connections is returned, providing bidirectional
/// communication by way of `RawPacket` instances. Policy file requests will
/// also be handled automatically by this function.
pub fn client_listener(
    address: &SocketAddr,
    mappings: impl AsRef<Mappings> + Send + 'static,
) -> IoResult<impl Stream<Item = Connection, Error = IoError> + Send> {
    let stream = TcpListener::bind(address)?
        .incoming()
        .and_then(configure_stream)
        .and_then(handle_policy_request)
        .filter_map(identity)
        .map(move |s| Codec::new_as_server(mappings.as_ref()).framed(s));

    Ok(stream)
}

/// Open a connection to the ROTMG server at the given socket address, using the
/// encryption keys provided by the given mappings.
///
/// A framed connection is returned, providing bidirectional communication by
/// way of `RawPacket` instances.
pub fn server_connection(
    address: &SocketAddr,
    mappings: impl AsRef<Mappings> + Send + 'static,
) -> impl Future<Item = Connection, Error = IoError> + Send {
    TcpStream::connect(address)
        .and_then(configure_stream)
        .map(move |s| Codec::new_as_client(mappings.as_ref()).framed(s))
}
