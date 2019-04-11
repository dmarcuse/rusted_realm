//! Mappings to convert official game packet IDs to and from `PacketType`.

use crate::packets::PacketType;
use bimap::BiHashMap;
use serde::{Deserialize, Serialize};

/// A set of mappings, used to convert ROTMG packet IDs to/from `PacketType`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mappings {
    mappings: BiHashMap<u8, PacketType>,
}

impl Mappings {
    /// Construct a `Mappings` instance using the given map between ROTMG packet
    /// IDs and internal packet types
    pub fn new(mappings: BiHashMap<u8, PacketType>) -> Self {
        Self { mappings }
    }

    /// Get the number of pairs present in these mappings. This may be used
    /// in conjunction with `PacketType::NUM_TYPES` to check whether all
    /// packet types are properly mapped.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Get a reference to the internal map used by this instance
    pub fn get_map(&self) -> &BiHashMap<u8, PacketType> {
        &self.mappings
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
}
