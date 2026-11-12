use zeroize::{Zeroize, ZeroizeOnDrop};

// 256 bit field element (x,y) 128 bit each
// X = [64bit x16 ], Y = calculatable from X
#[derive(Debug, Clone, Default, Zeroize, ZeroizeOnDrop)]
pub struct FieldElement(pub [i64; 16]);

impl FieldElement {
    pub const ONE: FieldElement = FieldElement([
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);

    pub const TWO: FieldElement = FieldElement([
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);

    pub const THREE: FieldElement = FieldElement([
        0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);

    pub const FIVE: FieldElement = FieldElement([
        0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);

    pub const NINE: FieldElement = FieldElement([
        0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);

    pub const ELEVEN: FieldElement = FieldElement([
        0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);

    // normalize from 16bit operation of each limb
    pub(crate) fn carry(&mut self) -> &mut Self {
        let mut carry;
        for i in 0..16 {
            carry = self.0[i] >> 16;
            self.0[i] -= carry << 16;
            if i < 15 {
                self.0[i + 1] += carry;
            } else {
                self.0[0] += 38 * carry;
            }
        }
        self
    }

    pub fn add(&self, b: &FieldElement) -> Self {
        let mut ret = Self::default();
        for i in 0..16 {
            ret.0[i] = self.0[i] + b.0[i];
        }
        ret
    }

    pub fn sub(&self, b: &FieldElement) -> Self {
        let mut ret = Self::default();

        for i in 0..16 {
            ret.0[i] = self.0[i] - b.0[i];
        }
        ret
    }

    pub fn mul(&self, b: &FieldElement) -> Self {
        let mut ret = Self::default();
        let mut product = [0i64; 31];

        for i in 0..16 {
            for j in 0..16 {
                product[i + j] += self.0[i] * b.0[j];
            }
        }

        for i in 0..15 {
            product[i] += 38 * product[i + 16]; // 2p = 2^(256) - 38
        }

        ret.0.copy_from_slice(&product[..16]);
        ret.carry().carry().carry();
        product.zeroize();
        ret
    }

    pub fn sqr(&self) -> Self {
        self.mul(self)
    }

    pub fn pow(&self, n: u32) -> Self {
        let mut result = Self::default();
        result.0[0] = 0x01; // multiplicative identity

        let mut base = Self::default();
        base.0.copy_from_slice(&self.0);

        let mut exp = n;
        while exp > 0 {
            if exp & 1 == 1 {
                result = result.mul(&base);
            }
            base = base.sqr();
            exp >>= 1;
        }

        result
    }

    pub fn inv(&self) -> Self {
        let mut inverse = Self::default();
        inverse.0.copy_from_slice(&self.0);

        for i in (0..=253).rev() {
            inverse = inverse.sqr();
            if i != 2 && i != 4 {
                inverse = inverse.mul(self);
            }
        }

        inverse
    }

    pub fn cswap(&mut self, elem: &mut FieldElement, bit: i16) {
        let mut tmp: i64;
        let cbit: i64 = !(bit - 1) as i64;

        for i in 0..16 {
            tmp = cbit & (self.0[i] ^ elem.0[i]);
            self.0[i] ^= tmp;
            elem.0[i] ^= tmp;
        }
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let mut elem = Self::default();

        for i in 0..16 {
            // packing bytes to a field elem in little endianne
            elem.0[i] = bytes[2 * i] as i64 + ((bytes[2 * i + 1] as i64) << 8);
        }
        // clear bit 255 (255-bit per FieldElement)
        elem.0[15] &= 0x7fff;
        elem
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        let mut tmp = FieldElement(self.0);
        let mut m = Self::default();
        let mut carry: i16;
        // normalizing to [2^(16)-1] max
        tmp.carry().carry().carry();

        // normalize a field element in mod P.
        for _ in 0..2 {
            m.0[0] = tmp.0[0] - 0xffed;
            for i in 1..15 {
                m.0[i] = tmp.0[i] - 0xffff - ((m.0[i - 1] >> 16) & 1);
                m.0[i - 1] &= 0xffff;
            }

            m.0[15] = tmp.0[15] - 0x7fff - ((m.0[14] >> 16) & 1);
            carry = ((m.0[15] >> 16) & 1) as i16;
            m.0[14] &= 0xffff;
            tmp.cswap(&mut m, 1 - carry);
        }

        // loading bytes array in little endianne
        for i in 0..16 {
            bytes[2 * i] = (tmp.0[i] & 0xff) as u8;
            bytes[2 * i + 1] = (tmp.0[i] >> 8) as u8;
        }

        bytes
    }

    #[allow(dead_code)]
    pub(crate) fn from_hex32(s: &str) -> [u8; 32] {
        let mut out = [0u8; 32];
        for i in 0..32 {
            out[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap();
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverse_of_9() {
        let inv = FieldElement::NINE.inv();

        let product = FieldElement::NINE.mul(&inv);
        let bytes = product.to_bytes();
        let mut expected = [0u8; 32];
        expected[0] = 1;

        // Verify: 9 * inv(9) == 1
        assert_eq!(bytes, expected, "9 * inverse(9) should equal 1");
    }

    #[test]
    fn test_inverse_of_2() {
        let inv = FieldElement::TWO.inv();
        let product = FieldElement::TWO.mul(&inv);
        let bytes = product.to_bytes();

        let mut expected = [0u8; 32];
        expected[0] = 1;
        assert_eq!(bytes, expected, "2 * inverse(2) should equal 1");
    }

    #[test]
    fn test_pow_basic() {
        let mut expected = [0u8; 32];
        expected[0] = 1;
        // 3^0 = 1
        assert_eq!(
            FieldElement::THREE.pow(0).to_bytes(),
            expected,
            "3^0 should equal 1"
        );

        expected[0] = 3;
        // 3^1 = 3
        assert_eq!(
            FieldElement::THREE.pow(1).to_bytes(),
            expected,
            "3^1 should equal 3"
        );

        expected[0] = 243;
        // 3^5 = 243
        assert_eq!(
            FieldElement::THREE.pow(5).to_bytes(),
            expected,
            "3^5 should equal 243"
        );

        let mut expected = [0u8; 32];
        expected[0] = 0; // 65536 = 0x10000
        expected[1] = 0;
        expected[2] = 1;
        // 2^16 = 65536
        assert_eq!(
            FieldElement::TWO.pow(16).to_bytes(),
            expected,
            "2^16 should equal 65536"
        );
    }

    #[test]
    fn test_add_and_sub() {
        let a = FieldElement::NINE;

        let b = FieldElement::ELEVEN;

        let mut expected = [0u8; 32];
        expected[0] = 2;
        let ret = b.sub(&a);
        let ret_bytes = ret.to_bytes();
        assert_eq!(ret_bytes, expected, "11 - 9 should be equal 2");

        let ret = a.add(&b);
        let ret = ret.pow(100).add(&a);
        // (a+b)^(100) + a = 19513394001496187003077916873180965119738524345383891134363041106384549505048 decimal
        let expected = FieldElement::from_hex32(
            "18e89e8e51d9a969625c6953db7d3aae48504139fb6e2b0000f1e8828a30242b",
        );

        let ret_bytes = ret.to_bytes();

        assert_eq!(
            ret_bytes, expected,
            "11 + 9 = 20, 20^(100) + 11 should be equal decimal"
        );
    }
}
