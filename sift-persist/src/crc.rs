// Precomputed CRC table generated at compile time
static CRC_TABLE: [u32; 256] = {
    let mut table = [0; 256];
    let mut n: u32 = 0;
    while n < 256 {
        let mut c = n;
        let mut i = 0;
        while i < 8 {
            c = if c & 1 != 0 {
                0xedb8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
            i += 1;
        }
        table[n as usize] = c;
        n += 1;
    }
    table
};

// Function to calculate CRC32. This is a simple implementation of the CRC32
// algorithm. It produces (High-level Data Link Control) HDLC CRCs as found in
// ITU-T Recommendation V.42 (March 2022)
// (https://www.itu.int/rec/T-REC-V.42/en)
//
// This is also the same CRC32 algorithm used by zlib, gzip, and PNG.
fn compute(crc: u32, buf: &[u8]) -> u32 {
    let mut c = crc;
    for byte in buf {
        c = CRC_TABLE[((c ^ u32::from(*byte)) & 0xff) as usize] ^ (c >> 8);
    }
    c
}

pub struct Digest {
    value: u32,
}

impl Digest {
    pub fn new() -> Self {
        Self { value: 0xffff_ffff }
    }

    pub fn update(&mut self, buf: &[u8]) {
        self.value = compute(self.value, buf);
    }

    #[must_use]
    pub fn finalize(&self) -> u32 {
        self.value ^ 0xffff_ffff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compute(data: &[u8]) -> u32 {
        let mut crc = Digest::new();
        crc.update(data);
        crc.finalize()
    }

    #[test]
    fn test_crc32() {
        struct TestCase {
            data: &'static str,
            expected_crc: u32,
        }
        let test_cases = [
            TestCase {
                data: "",
                expected_crc: 0x0,
            },
            TestCase {
                // Test case from https://rosettacode.org/wiki/CRC-32
                data: "The quick brown fox jumps over the lazy dog",
                expected_crc: 0x414f_a339,
            },
            TestCase {
                // Test case from
                // https://reveng.sourceforge.io/crc-catalogue/17plus.htm#crc.cat.crc-32-iso-hdlc
                data: "123456789",
                expected_crc: 0xcbf4_3926,
            },
            TestCase {
                // Test case from http://cryptomanager.com/tv.html
                data: "various CRC algorithms input data",
                expected_crc: 0x9bd3_66ae,
            },
            TestCase {
                // Source: http://www.febooti.com/products/filetweak/members/hash-and-crc/test-vectors/
                data: "Test vector from febooti.com",
                expected_crc: 0x0c87_7f61,
            },
        ];
        for test_case in test_cases {
            let crc = compute(test_case.data.as_bytes());

            assert!(
                crc == test_case.expected_crc,
                "CRC mismatch: expected {:08x}, got {:08x}, for data {:?}",
                test_case.expected_crc,
                crc,
                test_case.data
            );
        }
    }

    #[test]
    fn test_crc32_empty_input() {
        let data = b"";
        let expected_crc = 0x0;
        let calculated_crc = compute(data);
        assert_eq!(calculated_crc, expected_crc);
    }

    #[test]
    fn test_check() {
        let data = b"123456789";
        assert_eq!(format!("0x{:08x}", compute(data)), "0xcbf43926");
    }

    #[test]
    fn test_iend() {
        let data = b"IEND";
        let expected_crc: u32 = 0xae42_6082;
        let calculated_crc = compute(data);
        assert_eq!(calculated_crc, expected_crc);
    }
}
