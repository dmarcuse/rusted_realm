macro_rules! flag_enum {
    (
        $name:ident {
            $(
                $flag:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            serde::Serialize,
            serde::Deserialize,
        )]
        #[repr(u8)]
        pub enum $name {
            $(
                $flag = $value
            ),*
        }

        impl $name {
            const VALID: [Option<$name>; 256] = {
                let mut arr = [None; 256];

                $(
                    arr[$name::$flag as usize] = Some($name::$flag);
                )*

                arr
            };

            pub fn from_u8(byte: u8) -> Result<Self, $crate::avm2::ParseError> {
                use failure_derive::Fail;

                #[derive(Debug, Fail)]
                #[fail(display = "Flag value {} ({:#x}) is invalid for type {}", _0, _0, _1)]
                struct InvalidFlag(u8, &'static str);

                Self::VALID[byte as usize]
                    .ok_or(InvalidFlag(byte, stringify!($name)))
                    .map_err(|e| $crate::avm2::ParseError::Other(e.into()))
            }

            pub fn to_u8(self) -> u8 {
                self as u8
            }
        }

        impl From<$name> for u8 {
            fn from(flag: $name) -> u8 {
                flag.to_u8()
            }
        }

        impl $crate::avm2::Parse for $name {
            fn parse_avm2(buf: &mut dyn bytes::Buf) -> Result<Self, $crate::avm2::ParseError> {
                if buf.remaining() >= 1 {
                    $name::from_u8(buf.get_u8())
                } else {
                    Err($crate::avm2::ParseError::InsufficientBytes {
                        remaining: buf.remaining(),
                        needed: 1
                    })
                }
            }
        }
    };
}

macro_rules! data_struct {
    (
        $name:ident {
            $(
                $field:ident : $type:ty
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            $(
                $field : $type
            ),*
        }

        impl $crate::avm2::Parse for $name {
            fn parse_avm2(buf: &mut dyn bytes::Buf) -> Result<Self, $crate::avm2::ParseError> {
                $(
                    let $field = $crate::avm2::Parse::parse_avm2(buf)?;
                )*

                Ok(Self {
                    $(
                        $field
                    ),*
                })
            }
        }
    }
}
