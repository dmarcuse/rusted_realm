//! Tokio codec for framing ROTMG packets as `RawPacket` instances

use super::raw_packet::RawPacket;
use crate::rc4::Rc4;
use bytes::{Buf, BytesMut};
use failure_derive::Fail;
use rotmg_packets::mappings::{Mappings, RC4_LEN};
use std::io::{Cursor, Error as IoError};
use tokio::codec::{Decoder, Encoder};

/// Get the two RC4 ciphers
pub fn get_ciphers(mappings: &Mappings) -> (Rc4, Rc4) {
    let (key0, key1) = mappings.rc4().split_at(RC4_LEN / 2);
    (Rc4::new(key0), Rc4::new(key1))
}

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

    /// The packet size was invalid
    #[fail(display = "Invalid packet size: {}", _0)]
    InvalidSize(usize),
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
        let (recv_rc4, send_rc4) = get_ciphers(mappings);
        Self { recv_rc4, send_rc4 }
    }

    /// Construct a new codec for communicating with a game client - i.e. with
    /// this side of the connection acting as the client
    pub fn new_as_client(mappings: &Mappings) -> Self {
        let (send_rc4, recv_rc4) = get_ciphers(mappings);
        Self { recv_rc4, send_rc4 }
    }
}

impl Decoder for Codec {
    type Item = RawPacket;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            // we need more bytes to determine the packet size
            return Ok(None);
        }

        // get the total length of the packet
        let packet_size = {
            let mut cursor = Cursor::new(&src);
            cursor.get_u32_be() as usize
        };

        // the smallest valid packet is just a header, 5 bytes
        if packet_size < 5 {
            return Err(CodecError::InvalidSize(packet_size));
        }

        if src.len() < packet_size {
            // we haven't received the full packet yet, we need more bytes
            return Ok(None);
        }

        // full packet has been received
        // remove the entire packet from the buffer
        let mut data = src.split_to(packet_size);

        // decrypt the packet contents
        self.recv_rc4.process(&mut data[5..]);

        // yield the raw packet
        Ok(Some(RawPacket::new(data.freeze())))
    }
}

impl Encoder for Codec {
    type Item = RawPacket;
    type Error = CodecError;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // convert the packet back into bytes
        let packet = item.into_bytes();

        // make the packet mutable so we can encrypt the data
        let mut packet = BytesMut::from(packet);

        // encrypt the packet contents
        self.send_rc4.process(&mut packet[5..]);

        // finally, write the packet
        dst.extend_from_slice(&packet[..]);
        Ok(())
    }
}
