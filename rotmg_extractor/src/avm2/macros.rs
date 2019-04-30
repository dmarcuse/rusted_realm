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
    };
}
