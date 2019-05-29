//! A representation of packets that have been received and decrypted, but have
//! not yet been deserialized into `Packet` instances

use bytes::{Bytes, IntoBuf};
use failure_derive::Fail;
use rotmg_packets::adapter::Error as AdapterError;
use rotmg_packets::mappings::Mappings;
use rotmg_packets::packets::{Packet, PacketType};
use std::fmt::Debug;

/// A decrypted and properly framed packet represented as bytes.
///
/// By using an intermediary representation between the raw stream and `Packet`
/// instances, greater efficiency and fault tolerance can be achieved - the
/// packet can be kept as a `RawPacket` and only converted to a `Packet` after
/// inspecting the type, and if there is an error converting the `RawPacket` to
/// a `Packet`, the packet is still available in binary form and may still be
/// used.
#[derive(Clone)]
pub struct RawPacket {
    bytes: Bytes,
}

/// An error converting between a `RawPacket` and `Packet`.
/// The type parameter `T` represents the packet type - either `u8` or
/// `PacketType` - that is known in case there is no mapping for that value.
#[derive(Debug, Fail)]
pub enum Error<T: Debug + Send + Sync + 'static> {
    /// An error raised by the `Adapter` implementation for the `Packet` type
    #[fail(display = "Adapter error: {}", _0)]
    AdapterError(AdapterError),

    /// No mapping exists for the given packet type
    #[fail(display = "Unmapped packet type: {:?}", _0)]
    UnmappedPacketType(T),
}

impl RawPacket {
    /// Create a new `RawPacket` from the given bytes
    pub(crate) fn new(bytes: Bytes) -> RawPacket {
        debug_assert!(bytes.len() >= 5, "packet must be at least 5 bytes");
        Self { bytes }
    }

    /// Convert this `RawPacket` into the underlying `Bytes`
    pub(crate) fn into_bytes(self) -> Bytes {
        self.bytes
    }

    /// Get the total length of this packet, in bytes.
    pub fn total_len(&self) -> usize {
        self.bytes.len()
    }

    /// Get the length of the content of this packet (total length - header
    /// length).
    pub fn content_len(&self) -> usize {
        self.total_len() - 5
    }

    /// Get the ROTMG ID for this packet.
    ///
    /// A `Mappings` instance can be used to convert this ID to a `PacketType`
    /// variant.
    pub fn packet_id(&self) -> u8 {
        self.bytes[4]
    }

    /// Get the `PacketType` variant representing the type of this packet.
    ///
    /// Note that mappings must be provided in order to convert the ID used by
    /// the game to the appropriate `PacketType` value. If no pair is present in
    /// the `Mappings` instance for the ID of this packet, `None` will be
    /// returned.
    pub fn packet_type(&self, mappings: &Mappings) -> Option<PacketType> {
        mappings.to_internal(self.packet_id())
    }

    /// Get the decrypted binary contents of this packet
    pub fn raw_contents(&self) -> &[u8] {
        &self.bytes[5..]
    }

    /// Convert this `RawPacket` to a `Packet` instance using the given
    /// `Mappings`.
    ///
    /// An error will be returned if no mapping exists for this type of packet
    /// (`Error::UnmappedPacketType`) or if an error is returned by the
    /// `Adapter` implementation for this packet type (`Error::AdapterError`).
    pub fn to_packet(&self, mappings: &Mappings) -> Result<Packet, Error<u8>> {
        if let Some(typ) = self.packet_type(mappings) {
            unsafe { Packet::from_bytes(typ, &mut self.raw_contents().into_buf()) }
                .map_err(Error::AdapterError)
        } else {
            Err(Error::UnmappedPacketType(self.packet_id()))
        }
    }

    /// Convert the given `Packet` into a `RawPacket` using the given
    /// `Mappings`.
    ///
    /// An error will be returned if no mapping exists for this type of packet
    /// (`Error::UnmappedPacketType`) or if an error is returned by the
    /// `Adapter` implementation for this packet type (`Error::AdapterError`).
    pub fn from_packet(
        packet: &Packet,
        mappings: &Mappings,
    ) -> Result<RawPacket, Error<PacketType>> {
        if let Some(id) = mappings.to_game(packet.get_type()) {
            // create a buffer, reserve enough space to fit the packet size
            let mut buf = vec![0u8; 4];

            // store the packet id
            buf.push(id);

            // serialize the packet
            unsafe { packet.to_bytes(&mut buf).map_err(Error::AdapterError)? };

            // go back and store the total size of the packet
            let len = buf.len() as u32;
            (&mut buf[..4]).copy_from_slice(&len.to_be_bytes());

            Ok(Self::new(buf.into()))
        } else {
            Err(Error::UnmappedPacketType(packet.get_type()))
        }
    }
}
