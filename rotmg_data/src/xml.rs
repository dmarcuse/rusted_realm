//! Types representing objects specified in XML embedded in the ROTMG client

#![allow(missing_docs)]

mod hex {
    use num::Integer;
    use serde::{de, Deserializer, Serializer};
    use std::fmt::{Display, LowerHex};
    use std::fmt::{Formatter, Result as FmtResult};
    use std::marker::PhantomData;

    pub fn serialize<T: LowerHex, S: Serializer>(value: &T, ser: S) -> Result<S::Ok, S::Error> {
        // TODO: wrap T in something implementing Display to use collect_str?
        let hex = format!("0x{:x}", value);
        ser.serialize_str(&hex)
    }

    struct HexVisitor<T>(PhantomData<T>);

    impl<T> Default for HexVisitor<T> {
        fn default() -> Self {
            HexVisitor(PhantomData::default())
        }
    }

    impl<'de, T> de::Visitor<'de> for HexVisitor<T>
    where
        T: Integer,
        T::FromStrRadixErr: Display,
    {
        type Value = T;

        fn expecting(&self, f: &mut Formatter) -> FmtResult {
            write!(f, "a hex integer, optionally prefixed with 0x")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            T::from_str_radix(v.trim_start_matches("0x"), 16).map_err(E::custom)
        }
    }

    pub fn deserialize<'de, T, D: Deserializer<'de>>(de: D) -> Result<T, D::Error>
    where
        T: Integer,
        T::FromStrRadixErr: Display,
    {
        de.deserialize_str(HexVisitor::default())
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Texture {
    pub file: String,
    #[serde(with = "hex")]
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[repr(u8)]
pub enum ObjectClass {
    ArenaGuard,
    ArenaPortal,
    CaveWall,
    Character,
    CharacterChanger,
    ClosedGiftChest,
    ClosedVaultChest,
    ConnectedWall,
    Container,
    DailyLoginRewards,
    DoubleWall,
    Dye,
    Equipment,
    FortuneGround,
    FortuneTeller,
    GameObject,
    GuildBoard,
    GuildChronicle,
    GuildHallPortal,
    GuildMerchant,
    GuildRegister,
    Merchant,
    MoneyChanger,
    MysteryBoxGround,
    NameChanger,
    OneWayContainer,
    Pet,
    PetAbility,
    PetBehavior,
    PetSkin,
    PetUpgrader,
    Player,
    Portal,
    Projectile,
    QuestRewards,
    ReskinVendor,
    Sign,
    Skin,
    SpiderWeb,
    Stalagmite,
    Wall,
    YardUpgrader,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct Object {
    #[serde(rename = "type", with = "hex")]
    typ: u32,
    #[serde(rename = "id")]
    id: String,
    class: ObjectClass,
    texture: Texture,
    size: i32,
    #[serde(default)]
    shadow_size: Option<i32>,
    #[serde(default)]
    enemy: bool,
    #[serde(default)]
    invincible: bool,
    #[serde(default)]
    flying: bool,
}
