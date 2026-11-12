use crate::errors::Curve25519Error;
use crate::montgomery::MontgomeryPoint;
use getrandom;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Default, Zeroize, ZeroizeOnDrop)]
pub struct Scalar {
    pub bytes: [u8; 32],
}

impl From<MontgomeryPoint> for Scalar {
    fn from(point: MontgomeryPoint) -> Self {
        Scalar::from(point.0)
    }
}

impl From<[u8; 32]> for Scalar {
    fn from(bytes: [u8; 32]) -> Self {
        Scalar { bytes }
    }
}

impl Scalar {
    pub fn random_bytes<const N: usize>() -> Result<[u8; N], Curve25519Error> {
        let mut buf = [0u8; N];
        match getrandom::fill(&mut buf) {
            Ok(_) => Ok(buf),
            Err(e) => Err(Curve25519Error::FetchRandomBytes(e)),
        }
    }

    pub fn public_key(&self) -> MontgomeryPoint {
        MontgomeryPoint::scalar_mul_base(self)
    }
}

#[cfg(test)]
mod test {
    use crate::field::FieldElement;

    use super::*;

    #[test]
    fn test_scalar() {
        let buf = match Scalar::random_bytes::<32>() {
            Ok(b) => Scalar::from(b),
            Err(e) => panic!("{}", e),
        };

        assert_eq!(buf.bytes.len(), 32)
    }

    #[test]
    fn test_rfc7748_scalar() {
        let scalar = Scalar::from(FieldElement::from_hex32(
            "77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a",
        ));

        let expected = FieldElement::from_hex32(
            "8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a",
        );

        assert_eq!(scalar.public_key().0, expected);
    }

    // TODO: scalar entropy and statistical(NIST, dieharder,ent) rand test
    #[test]
    fn scalar_zeroized_on_drop() {
        let mut s = core::mem::ManuallyDrop::new(Scalar { bytes: [0xaa; 32] });
        let p = s.bytes.as_ptr();
        unsafe { core::mem::ManuallyDrop::drop(&mut s) };
        let after = unsafe { core::slice::from_raw_parts(p, 32) };
        assert!(after.iter().all(|&b| b == 0));
    }

    #[test]
    fn zeroize_explicit() {
        let mut p = MontgomeryPoint([0xff; 32]);
        p.zeroize();
        assert_eq!(p.0, [0u8; 32]);
    }
}
