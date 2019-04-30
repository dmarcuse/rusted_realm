//! Definitions of packet structures, adapters, and related types

// re-export things
pub use self::unified_definitions::{client, server, Packet, PacketType};

/// Define the structure of a packet
macro_rules! define_structure {
    (
        $name:ident { $(
            $fieldname:ident : $fieldtype:ty
        ),* $(,)? }
    ) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[allow(missing_docs)]
        pub struct $name {
            $(
                pub $fieldname : $fieldtype
            ),*
        }
    };
}

/// Define an adapter for a packet
macro_rules! define_adapter {
    (
        $name:ident { $(
            $fieldname:ident : $fieldtype:ty
        ),* $(,)? }
    ) => {
        impl Adapter for $name {
            fn get_be(_bytes: &mut dyn Buf) -> Result<Self> {
                $( let $fieldname = Adapter::get_be(_bytes)?; )*

                Ok(Self { $( $fieldname ),* })
            }

            fn put_be(&self, _bytes: &mut dyn BufMut) -> Result<()> {
                let Self { $( $fieldname ),* } = self;

                $( $fieldname.put_be(_bytes)?; )*

                Ok(())
            }
        }
    }
}

/// Define a single packet struct and (optionally) adapter
macro_rules! define_single_packet {
    ($side:tt $name:ident (ManualAdapter) $fields:tt) => {
        define_structure! { $name $fields }
    };
    ($side:tt $name:ident $fields:tt) => {
        define_single_packet! { $side $name (ManualAdapter) $fields }
        define_adapter! { $name $fields }
    };
}

/// Whether a packet is sent by the server
macro_rules! is_server {
    (Client) => {
        false
    };
    (Server) => {
        true
    };
}

/// Define the module for the given side
macro_rules! define_side {
    (Client: $( $name:ident ),* $(,)?) => {
        /// Packets sent by the client
        pub mod client { $( pub use super::$name; )* }
    };
    (Server: $( $name:ident ),* $(,)?) => {
        /// Packets sent by the server
        pub mod server { $( pub use super::$name; )* }
    };
}

/// Consumes a token tree and expands to nothing
macro_rules! consume {
    ($tokens:tt) => {};
}

/// Define all the other stuff
macro_rules! define_packets {
    (
        $(
            $side:ident {
                $(
                    $name: ident $( ( $adapterspec:tt ) )? {
                        $(
                            $fieldname:ident : $fieldtype:ty
                        ),* $(,)?
                    }
                ),* $(,)?
            }
        ),* $(,)?
    ) => {
        // define each packet struct and adapter
        $(
            $(
                define_single_packet! {
                    $side $name $( ( $adapterspec ) )* {
                        $( $fieldname : $fieldtype ),*
                    }
                }
            )*

            // also define modules for each side
            define_side! { $side : $( $name ),* }
        )*

        // next, the all-powerful Packet enum
        /// A packet of any type from either the server or the client
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        #[serde(tag = "type")]
        #[allow(missing_docs)]
        pub enum Packet {
            $( // each side
                $( // each packet
                    $name($name)
                ),*
            ),*
        }

        /// A compact (one byte) representation of the type of a packet.
        ///
        /// These values, when converted to a byte, do NOT match the values used
        /// by the official ROTMG client.
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
        #[repr(u8)]
        #[allow(missing_docs)]
        pub enum PacketType {
            $(
                $(
                    $name
                ),*
            ),*
        }

        $(
            $(
                // convert a Packet into a concrete type
                impl TryFrom<Packet> for $name {
                    type Error = Packet;

                    fn try_from(packet: Packet) -> StdResult<$name, Packet> {
                        match packet {
                            Packet::$name(v) => Ok(v),
                            p => Err(p),
                        }
                    }
                }

                // convert a Packet into a concrete type, by reference
                impl<'a> TryFrom<&'a Packet> for &'a $name {
                    type Error = &'a Packet;

                    fn try_from(packet: &'a Packet) -> StdResult<&'a $name, &'a Packet> {
                        match packet {
                            Packet::$name(v) => Ok(v),
                            p => Err(p),
                        }
                    }
                }

                // convert a concrete packet into a Packet
                impl From<$name> for Packet {
                    fn from(data: $name) -> Packet {
                        Packet::$name(data)
                    }
                }
            )*
        )*

        impl Packet {
            /// Attempt to downcast this packet into a specific type. On success,
            /// `Ok(T)` is returned, otherwise, `Err(Packet)` is returned.
            ///
            /// This method is provided for convenience and simply delegates to
            /// `TryInto`.
            pub fn downcast<T>(self) -> StdResult<T, Self>
            where
                T: TryFrom<Packet, Error = Packet>,
            {
                self.try_into()
            }

            /// Attempt to downcast this packet into a specific type by reference. On
            /// success, `Ok(&T)` is returned, otherwise, `Err(&Packet)` is returned.
            ///
            /// This method is provided for convenience and simply delegates to
            /// `TryInto`.
            ///
            /// # Example
            ///
            /// ```
            /// # use rotmg_networking::packets::{Packet, client::{CancelTrade, AcceptTrade}};
            ///
            /// // create a Packet
            /// let packet = Packet::CancelTrade(CancelTrade {});
            ///
            /// // attempt to downcast it
            /// assert_eq!(Ok(&CancelTrade {}), packet.downcast_ref());
            /// assert_eq!(Err(&packet), packet.downcast_ref::<AcceptTrade>());
            /// ```
            pub fn downcast_ref<'a, T>(&'a self) -> StdResult<&'a T, &'a Self>
            where
                &'a T: TryFrom<&'a Packet, Error = &'a Packet>
            {
                self.try_into()
            }

            /// Get the `PacketType` of this packet
            pub fn get_type(&self) -> PacketType {
                match self {
                    $(
                        $(
                            Packet::$name(_) => PacketType::$name
                        ),*
                    ),*
                }
            }

            /// Create a packet from the given type and contents
            pub(crate) fn from_bytes(typ: PacketType, contents: &mut dyn Buf) -> Result<Packet> {
                typ.get_deserializer()(contents)
            }

            /// Write the contents of this packet to the given buffer
            pub(crate) fn to_bytes(&self, buf: &mut dyn BufMut) -> Result<()> {
                self.get_type().get_serializer()(self, buf)
            }
        }

        type PacketDeserializer = fn(&mut dyn Buf) -> Result<Packet>;
        type PacketSerializer = fn(&Packet, &mut dyn BufMut) -> Result<()>;
        impl PacketType {
            const VALID_TYPES: [Option<PacketType>; 256] = {
                let mut arr = [None; 256];

                $(
                    $(
                        arr[PacketType::$name as usize] = Some(PacketType::$name);
                    )*
                )*

                arr
            };

            /// Attempt to get the `PacketType` represented by the given byte.
            /// Note that this byte representation does not match the one used
            /// by the official ROTMG client.
            pub fn from_byte(byte: u8) -> Option<PacketType> {
                Self::VALID_TYPES[byte as usize]
            }

            /// Get a set of all packet types, as a reference to a static,
            /// lazily-initialized `HashSet`.
            pub fn get_all_types() -> &'static HashSet<PacketType> {
                lazy_static! {
                    static ref ALL_TYPES: HashSet<PacketType> = {
                        let mut set = HashSet::with_capacity(256);

                        $(
                            $(
                                set.insert(PacketType::$name);
                            )*
                        )*

                        set.shrink_to_fit();
                        set
                    };
                }

                &ALL_TYPES
            }

            /// The number of different packet types
            pub const NUM_TYPES: usize = {
                let mut count = 0;

                $(
                    $(
                        count += 1;
                        consume!($name);
                    )*
                )*

                count
            };

            const DESERIALIZERS: [Option<PacketDeserializer>; 256] = {
                let mut arr: [Option<PacketDeserializer>; 256] = [None; 256];

                $(
                    $(
                        arr[PacketType::$name as usize] = Some(|b| {
                            $name::get_be(b).map(Packet::$name)
                        });
                    )*
                )*

                arr
            };

            fn get_deserializer(self) -> PacketDeserializer {
                Self::DESERIALIZERS[self as usize].unwrap()
            }

            const SERIALIZERS: [Option<PacketSerializer>; 256] = {
                let mut arr: [Option<PacketSerializer>; 256] = [None; 256];

                $(
                    $(
                        arr[PacketType::$name as usize] = Some(|p, b| {
                            let concrete: &$name = p.downcast_ref().unwrap();
                            concrete.put_be(b)
                        });
                    )*
                )*

                arr
            };

            fn get_serializer(self) -> PacketSerializer {
                Self::SERIALIZERS[self as usize].unwrap()
            }

            /// Get a map of packet types to names, as a reference to a static,
            /// lazily-initialized `HashMap`
            pub fn get_name_mappings() -> &'static HashMap<PacketType, &'static str> {
                lazy_static! {
                    static ref NAMES: HashMap<PacketType, &'static str> = {
                        let mut map = HashMap::with_capacity(256);

                        $(
                            $(
                                map.insert(PacketType::$name, stringify!($name));
                            )*
                        )*

                        map.shrink_to_fit();
                        map
                    };
                }

                &NAMES
            }

            /// Get the name for this packet type
            pub fn get_name(self) -> &'static str {
                Self::get_name_mappings()[&self]
            }

            const SERVERSIDE: [bool; 256] = {
                let mut arr = [false; 256];

                $(
                    $(
                        arr[PacketType::$name as usize] = is_server!($side);
                    )*
                )*

                arr
            };

            /// Check whether this packet type is sent by the server
            pub fn is_server(self) -> bool {
                Self::SERVERSIDE[self as usize]
            }

            /// Check whether this packet type is sent by the client
            pub fn is_client(self) -> bool {
                !Self::SERVERSIDE[self as usize]
            }
        }

        /// A trait indicating that a type represents the contents of a packet
        pub trait PacketData: Adapter {
            const PACKET_TYPE: PacketType;
        }

        $(
            $(
                impl PacketData for $name {
                    const PACKET_TYPE: PacketType = PacketType::$name;
                }
            )*
        )*
    };
}

mod unified_definitions {
    use crate::adapter::{Adapter, Error, Result, RLE};
    use crate::packets::packet_data::*;
    use bytes::{Buf, BufMut};
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};
    use std::collections::{HashMap, HashSet};
    use std::convert::{TryFrom, TryInto};
    use std::result::Result as StdResult;

    define_packets! {
        Client {
            AcceptTrade { my_offer: RLE<Vec<bool>>, your_offer: RLE<Vec<bool>> },
            ActivePetUpdateRequest { command_type: u8, instance_id: u32 },
            AoeAck { time: u32, pos: WorldPosData },
            Buy { object_id: u32, quantity: u32 },
            CancelTrade {},
            ChangeGuildRank { name: RLE<String>, guild_rank: u32 },
            ChangeTrade { offer: RLE<Vec<bool>> },
            CheckCredits {},
            ChooseName { name: RLE<String> },
            ClaimLoginRewardMsg { claim_key: RLE<String>, typ: RLE<String> },
            Create { class_type: u16, skin_type: u16 },
            CreateGuild { name: RLE<String> },
            EditAccountList { account_list_id: u32, add: bool, object_id: u32 },
            EnemyHit { time: u32, bullet_id: u8, target_id: u32, kill: bool },
            EnterArena { currency: u32 },
            Escape {},
            GotoAck { time: u32 },
            GroundDamage { time: u32, pos: WorldPosData },
            GuildInvite { name: RLE<String> },
            GuildRemove { name: RLE<String> },
            Hello {
                build_version: RLE<String>,
                game_id: u32,
                guid: RLE<String>,
                rand1: u32,
                password: RLE<String>,
                rand2: u32,
                secret: RLE<String>,
                key_time: u32,
                key: RLE<Vec<u8>>,
                map_json: RLE<String, u32>,
                entry_tag: RLE<String>,
                game_net: RLE<String>,
                game_net_user_id: RLE<String>,
                play_platform: RLE<String>,
                platform_token: RLE<String>,
                user_token: RLE<String>,
            },
            InvDrop { slot: SlotObjectData },
            InvSwap { time: u32, pos: WorldPosData, slot1: SlotObjectData, slot2: SlotObjectData },
            JoinGuild { guild_name: RLE<String> },
            KeyInfoRequest { item_type: u32 },
            Load { char_id: u32, from_arena: bool },
            Move {
                tick_id: u32,
                time: u32,
                new_pos: WorldPosData,
                records: RLE<Vec<MoveRecord>>
            },
            OtherHit { time: u32, bullet_id: u8, object_id: u32, target_id: u32 },
            PetChangeFormMsg { instance_id: u32, picked_new_pet_type: u32, item: SlotObjectData },
            PetChangeSkinMsg { pet_id: u32, skin_type: u32, currency: u32 },
            PetUpgradeRequest {
                pet_trans_type: u8,
                pid_one: u32,
                pid_two: u32,
                object_id: u32,
                payment_trans_type: u8,
                slots: RLE<Vec<SlotObjectData>>
            },
            PlayerHit { bullet_id: u8, object_id: u32 },
            PlayerShoot {
                time: u32,
                bullet_id: u8,
                container_type: u16,
                starting_pos: WorldPosData,
                angle: f32
            },
            PlayerText { text: RLE<String> },
            QuestRedeem { quest_id: RLE<String>, item: u32, slots: RLE<Vec<SlotObjectData>> },
            QuestRoomMsg {},
            Pong { serial: u32, time: u32 },
            RequestTrade { name: RLE<String> },
            ResetDailyQuests {},
            Reskin { skin_id: u32 },
            SetCondition { effect: u8, duration: f32 },
            ShootAck { time: u32 },
            SquareHit { time: u32, bullet_id: u8, object_id: u32 },
            Teleport { object_id: u32 },
            UpdateAck {},
            UseItem { time: u32, slot: SlotObjectData, pos: WorldPosData, use_type: u32 },
            UsePortal { object_id: u32 },

            // TODO: these are blind guesses
            QuestFetchAsk {},
            AcceptArenaDeath {},
        },
        Server {
            AccountList {
                account_list_id: u32,
                account_ids: RLE<Vec<RLE<String>>>,
                lock_action: u32
            },
            ActivePetUpdate { instance_id: u32 },
            AllyShoot { bullet_id: u8, owner_id: u32, container_type: u16, angle: f32 },
            Aoe {
                pos: WorldPosData,
                radius: f32,
                damage: u16,
                effect: u8,
                duration: f32,
                orig_type: u16,
                color: u32,
                armor_pierce: bool
            },
            ArenaDeath { cost: u32 },
            BuyResult { result: u32, result_string: RLE<String> }, // TODO: consts for this?
            ClientStat { name: RLE<String>, value: u32 },
            CreateSuccess { object_id: u32, char_id: u32 },
            Damage {
                target_id: u32,
                effects: RLE<Vec<u8>, u8>,
                damage_amount: u16,
                kill: bool,
                armor_pierce: bool,
                bullet_id: u8,
                object_id: u32
            },
            Death {
                account_id: RLE<String>,
                char_id: u32,
                killed_by: RLE<String>,
                zombie_type: u32,
                zombie_id: u32,
            },
            DeletePet { pet_id: u32 },
            EnemyShoot {
                bullet_id: u8,
                owner_id: u32,
                bullet_type: u8,
                starting_pos: WorldPosData,
                angle: f32,
                damage: u16,
                num_shots: Option<u8>,
                angle_inc: Option<f32>
            },
            EvolvePet { pet_id: u32, initial_skin: u32, final_skin: u32 },
            Failure { error_id: u32, error_description: RLE<String> }, // TODO: consts?
            File { filename: RLE<String>, file: RLE<String, u32> }, // TODO: investigate this
            GlobalNotification { notification_type: u32, text: RLE<String> },
            Goto { object_id: u32, pos: WorldPosData },
            GuildResult { success: bool, line_builder_json: RLE<String> },
            HatchPet { pet_name: RLE<String>, pet_skin: u32, item_type: u32 },
            InvResult { result: u32 },
            InvitedToGuild { name: RLE<String>, guild_name: RLE<String> },
            ImminentArenaWave { current_runtime: u32 },
            KeyInfoResponse { name: RLE<String>, description: RLE<String>, creator: RLE<String> },
            LoginRewardMsg { item_id: u32, quantity: u32, gold: u32 },
            MapInfo { // TODO: double check this, maybe use manual adapter
                width: u32,
                height: u32,
                name: RLE<String>,
                display_name: RLE<String>,
                fp: u32,
                background: u32,
                difficulty: u32,
                allow_player_teleport: bool,
                show_displays: bool,
                client_xml: RLE<Vec<RLE<String, u32>>>,
                extra_xml: RLE<Vec<RLE<String, u32>>>
            },
            NameResult { success: bool, error_text: RLE<String> },
            NewAbility { typ: u32 },
            NewTick { tick_id: u32, tick_time: u32, statuses: RLE<Vec<ObjectStatusData>> },
            Notification { object_id: u32, message: RLE<String>, color: u32 },
            PasswordPrompt { clean_password_status: u32 },
            PetYardUpdate { typ: u32 },
            Pic(ManualAdapter) { w: u32, h: u32, bitmap_data: Vec<u8> },
            Ping { serial: u32 },
            PlaySound { owner_id: u32, sound_id: u8 },
            QuestObjId { object_id: u32 },
            QuestFetchResponse { quests: RLE<Vec<QuestData>>, next_refresh_price: u32 },
            QuestRedeemResponse { ok: bool, message: RLE<String> },
            RealmHeroLeftMsg { number_of_realm_heroes: u32 },
            Reconnect {
                name: RLE<String>,
                host: RLE<String>,
                stats: RLE<String>,
                port: u32,
                game_id: u32,
                key_time: u32,
                is_from_arena: bool,
                key: RLE<Vec<u8>>
            },
            ReskinUnlock { skin_id: u32, is_pet_skin: u32 },
            ServerPlayerShoot {
                bullet_id: u8,
                owner_id: u32,
                container_type: u32,
                starting_pos: WorldPosData,
                angle: f32,
                damage: u16
            },
            ShowEffect { // TODO: consts?
                effect_type: u8,
                target_object_id: u32,
                pos1: WorldPosData,
                pos2: WorldPosData,
                color: u32,
                duration: f32
            },
            Text {
                name: RLE<String>,
                object_id: u32,
                num_stars: u32,
                bubble_time: u8,
                recipient: RLE<String>,
                text: RLE<String>,
                clean_text: RLE<String>,
                is_supporter: bool
            },
            TradeAccepted { my_offer: RLE<Vec<bool>>, your_offer: RLE<Vec<bool>> },
            TradeChanged { offer: RLE<Vec<bool>> },
            TradeDone { code: u32, description: RLE<String> }, // TODO: consts?
            TradeRequested { name: RLE<String> },
            TradeStart {
                my_items: RLE<Vec<TradeItem>>,
                your_name: RLE<String>,
                your_items: RLE<Vec<TradeItem>>
            },
            Update {
                tiles: RLE<Vec<GroundTileData>>,
                new_objs: RLE<Vec<ObjectData>>,
                drops: RLE<Vec<u32>>
            },
            VerifyEmail {}
        }
    }

    // manually implemented adapter for Pic packet
    impl Adapter for Pic {
        fn get_be(bytes: &mut dyn Buf) -> Result<Self> {
            let w = u32::get_be(bytes)?;
            let h = u32::get_be(bytes)?;

            let reqd_bytes = (w as usize) * (h as usize) * 4;

            if bytes.remaining() >= reqd_bytes {
                let mut bitmap_data = vec![0u8; reqd_bytes];
                bytes.copy_to_slice(&mut bitmap_data[..]);
                Ok(Self { w, h, bitmap_data })
            } else {
                Err(Error::InsufficientBytes {
                    remaining: bytes.remaining(),
                    needed: reqd_bytes,
                })
            }
        }

        fn put_be(&self, bytes: &mut dyn BufMut) -> Result<()> {
            self.w.put_be(bytes)?;
            self.h.put_be(bytes)?;
            bytes.put_slice(&self.bitmap_data[..]);
            Ok(())
        }
    }

}
