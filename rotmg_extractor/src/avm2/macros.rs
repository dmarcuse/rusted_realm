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

            pub fn from_u8(byte: u8) -> Option<Self> {
                Self::VALID[byte as usize]
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
                use failure_derive::Fail;

                #[derive(Debug, Fail)]
                #[fail(display = "Flag value {} (0x${:x}) is invalid for type {}", _0, _0, _1)]
                struct InvalidFlag(u8, &'static str);

                if buf.remaining() >= 1 {
                    let byte = buf.get_u8();
                    Ok($name::from_u8(byte)
                        .ok_or(InvalidFlag(byte, stringify!($name)))
                        .map_err(|e| $crate::avm2::ParseError::Other(e.into()))?)
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
