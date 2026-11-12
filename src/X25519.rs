use crate::errors::Curve25519Error;
use crate::montgomery::MontgomeryPoint;
use crate::scalar::Scalar;

pub fn gen_keypair() -> Result<(Scalar, MontgomeryPoint), Curve25519Error> {
    let sk = match Scalar::random_bytes::<32>() {
        Ok(buf) => Scalar::from(buf),
        Err(e) => Err(e)?,
    };

    let pk = sk.public_key();

    Ok((sk, pk))
}

pub fn x25519(pk: &MontgomeryPoint, sk: &Scalar) -> MontgomeryPoint {
    pk.scalar_mul(sk)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_x25519() {
        let k = match Scalar::random_bytes::<32>() {
            Ok(buf) => Scalar::from(buf),
            Err(e) => panic!("{}", e),
        };

        let (sk, pk) = match gen_keypair() {
            Ok((sk, pk)) => (sk, pk),
            Err(e) => panic!("{}", e),
        }; // recipient's

        let shared = x25519(&pk, &k); // [k] * pk
        let c_prime = x25519(&MontgomeryPoint::X25519_BASEPOINT, &k); // [k] * 9
        let expected = x25519(&c_prime, &sk);

        assert_eq!(
            shared.as_bytes(),
            expected.as_bytes(),
            "x25519 KEM test failed"
        );
    }
}
