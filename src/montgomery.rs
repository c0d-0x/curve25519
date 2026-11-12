use crate::field::FieldElement;
use crate::scalar::Scalar;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Default, Zeroize, ZeroizeOnDrop)]
pub struct MontgomeryPoint(pub [u8; 32]);

impl From<FieldElement> for MontgomeryPoint {
    fn from(elem: FieldElement) -> Self {
        MontgomeryPoint(elem.to_bytes())
    }
}

impl From<Scalar> for MontgomeryPoint {
    fn from(value: Scalar) -> Self {
        MontgomeryPoint(value.bytes)
    }
}

impl MontgomeryPoint {
    pub(crate) const _121665: FieldElement = FieldElement([
        0xDB41, 0x01, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ]);

    pub(crate) const X25519_BASEPOINT: MontgomeryPoint = MontgomeryPoint([
        0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ]);

    pub fn as_bytes(&self) -> [u8; 32] {
        self.0
    }

    pub fn scalar_mul(&self, scalar: &Scalar) -> Self {
        let mut clamped = scalar.bytes;
        clamped[0] &= 0xf8;
        clamped[31] = (clamped[31] & 0x7f) | 0x40;

        let point = self.ladder(&clamped);
        clamped.zeroize();
        point
    }

    // montgomery ladder
    fn ladder(&self, k: &[u8; 32]) -> Self {
        let u = FieldElement::from_bytes(&self.0);
        let mut x = FieldElement::ONE;
        let mut z = FieldElement::default();

        let mut a = FieldElement::default();
        let mut b = FieldElement::ONE;

        a.0.copy_from_slice(&u.0);

        for i in (0..=254usize).rev() {
            let bit = ((k[i >> 3] >> (i & 7)) & 1) as i16;

            x.cswap(&mut a, bit);
            z.cswap(&mut b, bit);

            let e = x.add(&z);
            x = x.sub(&z);
            z = a.add(&b);
            a = a.sub(&b);

            b = e.sqr();
            let f = x.sqr();
            x = z.mul(&x);
            z = a.mul(&e);

            let e = x.add(&z);
            x = x.sub(&z);
            a = x.sqr();
            z = b.sub(&f);
            x = z.mul(&Self::_121665);
            x = x.add(&b);

            z = z.mul(&x);
            x = b.mul(&f);
            b = a.mul(&u);
            a = e.sqr();

            x.cswap(&mut a, bit);
            z.cswap(&mut b, bit);
        }

        // U = X/Z
        z = z.inv();
        x = x.mul(&z);
        Self(x.to_bytes())
    }

    pub fn scalar_mul_base(scalar: &Scalar) -> Self {
        Self::X25519_BASEPOINT.scalar_mul(scalar)
    }

    //TODO: impliment a hash function for hashing a point
    pub fn hash(&self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // L(P,0) = 0
    #[test]
    fn test_ladder_0g() {
        let k = [0u8; 32];
        let pk = MontgomeryPoint::X25519_BASEPOINT.ladder(&k);
        assert_eq!(pk.0, FieldElement::default().to_bytes());
    }

    #[test]
    fn test_ladder_1g() {
        let mut k = [0u8; 32];
        k[0] = 0x01;
        let pk = MontgomeryPoint::X25519_BASEPOINT.ladder(&k);
        assert_eq!(pk.0, MontgomeryPoint::X25519_BASEPOINT.0);
    }

    // [5]*P = R
    #[test]
    fn test_ladder_5g() {
        let mut k = [0u8; 32];
        k[0] = 0x05;
        let pk = MontgomeryPoint::X25519_BASEPOINT.ladder(&k);
        let expected = FieldElement::from_hex32(
            "877c4978577d530dcb491d58bcc9cba87f9e075e6e02c003f27aee503cecb641",
        );
        assert_eq!(pk.0, expected, "[5] * P(9,y) ");
    }

    // Test from RFC 7748
    #[test]
    fn test_rfc7748_vector1() {
        let scalar = Scalar::from(FieldElement::from_hex32(
            "a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4",
        ));

        let point = MontgomeryPoint(FieldElement::from_hex32(
            "e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c",
        ));

        let expected = FieldElement::from_hex32(
            "c3da55379de9c6908e94ea4df28d084f32eccf03491c71f754b4075577a28552",
        );
        assert_eq!(point.scalar_mul(&scalar).0, expected);
    }

    #[test]
    fn test_rfc7748_vector2() {
        let scalar = Scalar::from(FieldElement::from_hex32(
            "4b66e9d4d1b4673c5ad22691957d6af5c11b6421e0ea01d42ca4169e7918ba0d",
        ));

        let point = MontgomeryPoint(FieldElement::from_hex32(
            "e5210f12786811d3f4b7959d0538ae2c31dbe7106fc03c3efc4cd549c715a493",
        ));

        let expected = FieldElement::from_hex32(
            "95cbde9476e8907d7aade45cb4b873f88b595a68799fa152e6f8f7647aac7957",
        );

        assert_eq!(point.scalar_mul(&scalar).0, expected);
    }
}
