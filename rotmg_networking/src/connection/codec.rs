//! A tokio codec to frame ROTMG packets

use crate::mappings::Mappings;
use crate::rc4::Rc4;
use failure_derive::Fail;
use std::io::Error as IoError;

/// The codec for framing and encrypting/decrypting ROTMG packets. This struct
/// contains the minimum state necessary - just the RC4 ciphers for sending and
/// receiving packets.
#[derive(Clone)]
pub struct Codec {
    recv_rc4: Rc4,
    send_rc4: Rc4,
}

/// An error that occurred while reading or writing a packet
#[derive(Debug, Fail)]
pub enum CodecError {
    /// A low level IO error
    #[fail(display = "IO error: {}", _0)]
    IoError(IoError),
}

impl From<IoError> for CodecError {
    fn from(e: IoError) -> Self {
        CodecError::IoError(e)
    }
}

impl Codec {
    /// Construct a new codec for communicating with a game client - i.e. with
    /// this side of the connection acting as the server
    pub fn new_as_server(mappings: &Mappings) -> Self {
        let (recv_rc4, send_rc4) = mappings.get_ciphers();
        Self { recv_rc4, send_rc4 }
    }

    /// Construct a new codec for communicating with a game client - i.e. with
    /// this side of the connection acting as the client
    pub fn new_as_client(mappings: &Mappings) -> Self {
        let (send_rc4, recv_rc4) = mappings.get_ciphers();
        Self { recv_rc4, send_rc4 }
    }
}
