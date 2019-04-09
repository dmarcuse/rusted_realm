//! Pure rust RC4 implementation that works in-place
//!
//! This is adapted from the `rust-crypto` project, the original source is
//! available [on GitHub](https://github.com/DaGenix/rust-crypto/blob/master/src/rc4.rs]

/// The state of an RC4 cipher
#[derive(Clone)]
pub struct Rc4 {
    i: u8,
    j: u8,
    state: [u8; 256],
}

impl Rc4 {
    /// Create a new RC4 state with the given key
    ///
    /// The key must be between 1 and 256 bytes long, inclusive
    pub fn new(key: &[u8]) -> Self {
        assert!(!key.is_empty(), "key must not be empty");
        assert!(key.len() <= 256, "key may not be longer than 256 bytes");

        // create an empty state
        let mut rc4 = Self {
            i: 0,
            j: 0,
            state: [0u8; 256],
        };

        for (i, x) in rc4.state.iter_mut().enumerate() {
            *x = i as u8;
        }

        let mut j: u8 = 0;
        for i in 0..256 {
            j = j
                .wrapping_add(rc4.state[i])
                .wrapping_add(key[i % key.len()]);
            rc4.state.swap(i, j as usize);
        }

        rc4
    }

    fn next(&mut self) -> u8 {
        self.i = self.i.wrapping_add(1);
        self.j = self.j.wrapping_add(self.state[self.i as usize]);
        self.state.swap(self.i as usize, self.j as usize);
        self.state[(self.state[self.i as usize].wrapping_add(self.state[self.j as usize])) as usize]
    }

    /// Process the given input with this RC4 state
    pub fn process(&mut self, bytes: &mut [u8]) {
        for n in bytes.iter_mut() {
            *n ^= self.next();
        }
    }
}

#[cfg(test)]
mod test {
    use super::Rc4;

    pub struct Test {
        pub key: &'static str,
        pub input: &'static str,
        pub output: Vec<u8>,
    }

    fn tests() -> Vec<Test> {
        vec![
            Test {
                key: "Key",
                input: "Plaintext",
                output: vec![0xBB, 0xF3, 0x16, 0xE8, 0xD9, 0x40, 0xAF, 0x0A, 0xD3],
            },
            Test {
                key: "Wiki",
                input: "pedia",
                output: vec![0x10, 0x21, 0xBF, 0x04, 0x20],
            },
            Test {
                key: "Secret",
                input: "Attack at dawn",
                output: vec![
                    0x45, 0xA0, 0x1F, 0x64, 0x5F, 0xC3, 0x5B, 0x38, 0x35, 0x52, 0x54, 0x4B, 0x9B,
                    0xF5,
                ],
            },
        ]
    }

    #[test]
    fn wikipedia_tests() {
        let tests = tests();
        for t in tests.iter() {
            let mut rc4 = Rc4::new(t.key.as_bytes());
            let mut bytes = t.input.as_bytes().to_vec();
            rc4.process(&mut bytes[..]);
            assert!(bytes == t.output);
        }
    }
}
