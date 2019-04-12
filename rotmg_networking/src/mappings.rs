//! Mappings to convert official game packet IDs to and from `PacketType` and
//! store RC4 keys.

use crate::packets::PacketType;
use crate::rc4::Rc4;
use bimap::BiHashMap;
use failure_derive::Fail;
use hex::FromHexError;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

/// The length of RC4 keys in their binary representation, in bytes
pub const RC4_LEN: usize = 26;

/// A set of mappings, used to convert ROTMG packet IDs to/from `PacketType` and
/// store initial RC4 cipher states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mappings {
    mappings: BiHashMap<u8, PacketType>,
    binary_rc4: [u8; RC4_LEN],
}

/// An error with the RC4 key when constructing mappings
#[derive(Debug, Clone, Fail)]
pub enum RC4KeyError {
    /// Caused by invalid hexadecimal characters in the RC4 key
    #[fail(display = "Invalid RC4 key hex: {} for key {}", _1, _0)]
    InvalidRC4Hex(String, FromHexError),

    /// Caused by invalid RC4 key lengths
    #[fail(display = "Invalid RC4 key length: {} for key {}", _1, _0)]
    InvalidRC4Len(String, usize),
}

impl Mappings {
    /// Construct a `Mappings` instance using the given map between ROTMG packet
    /// IDs and internal packet types, and the given hexadecimal RC4 key
    pub fn new(mappings: BiHashMap<u8, PacketType>, hex_rc4: &str) -> Result<Self, RC4KeyError> {
        let binary_rc4 = match hex::decode(hex_rc4) {
            Err(e) => return Err(RC4KeyError::InvalidRC4Hex(hex_rc4.to_string(), e)),
            Ok(ref b) if b.len() != RC4_LEN => {
                return Err(RC4KeyError::InvalidRC4Len(hex_rc4.to_string(), b.len()))
            }
            Ok(b) => b[..].try_into().unwrap(),
        };

        Ok(Self {
            binary_rc4,
            mappings,
        })
    }

    /// Get a reference to the internal map used by this instance
    pub fn get_map(&self) -> &BiHashMap<u8, PacketType> {
        &self.mappings
    }

    /// Get an iterator over the `PacketType` variants that are missing from
    /// this set of mappings
    pub fn find_unmapped(&self) -> impl Iterator<Item = PacketType> + '_ {
        PacketType::get_all_types()
            .iter()
            .cloned()
            .filter(move |t| !self.mappings.contains_right(t))
    }

    /// Attempt to convert the given ROTMG packet ID to an internal type. `None`
    /// indicates that no pair is present for the given ID.
    pub fn to_internal(&self, id: u8) -> Option<PacketType> {
        self.mappings.get_by_left(&id).cloned()
    }

    /// Attempt to convert the given internal packet type to a ROTMG ID. `None`
    ///indicates that no pair is present for the given ID.
    pub fn to_game(&self, packet_type: PacketType) -> Option<u8> {
        self.mappings.get_by_right(&packet_type).cloned()
    }

    /// Get the two RC4 ciphers
    pub fn get_ciphers(&self) -> (Rc4, Rc4) {
        let (key0, key1) = self.binary_rc4.split_at(RC4_LEN / 2);
        (Rc4::new(key0), Rc4::new(key1))
    }
}
