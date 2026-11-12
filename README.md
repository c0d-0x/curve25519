# curve25519

A Curve25519 and the X25519 Diffie-Hellman function implementation in Rust, with as few external dependencies as possible.

> [!WARNING]
>
> An EDUCATIONAL implementation, NOT AUDITED. Do not use in production.

## Features

- Field arithmetic in F_p, p = 2^255 - 19: addition, subtraction, multiplication (reduction mod 2p), carry, inversion (Fermat's little theorem), constant-time conditional swap, canonical conversion to and from bytes
- Field element representation: 16 limbs of 16 bits held in `i64` (following TweetNaCl)
- Montgomery ladder, 255 iterations, no secret-dependent branches or indices
- Scalar clamping, with scalars zeroized after use
- Key-pair generation from the OS random source
- X25519 interface: fixed 32-byte keys and shared secrets

## Layout

```
src
├── errors.rs       library errors
├── field.rs        F_p implementation
├── lib.rs          library entry
├── montgomery.rs   Montgomery ladder and scalar multiplication
├── scalar.rs       scalar type
└── X25519.rs       X25519 interface
```

## Usage

```rust
use curve25519::{gen_keypair, x25519};

fn main() {
    let (alice_sk, alice_pk) = gen_keypair().expect("keygen failed");
    let (bob_sk, bob_pk) = gen_keypair().expect("keygen failed");

    let alice_shared = x25519(&bob_pk, &alice_sk);
    let bob_shared = x25519(&alice_pk, &bob_sk);

    assert_eq!(alice_shared, bob_shared);
}
```

## API

| Function                                                              | Description                                     |
| --------------------------------------------------------------------- | ----------------------------------------------- |
| `gen_keypair() -> Result<(Scalar, MontgomeryPoint), Curve25519Error>` | Random secret scalar and its public key         |
| `x25519(pk: &MontgomeryPoint, sk: &Scalar) -> MontgomeryPoint`        | Diffie-Hellman shared secret                    |
| `MontgomeryPoint::scalar_mul_base(scalar: &Scalar)`                   | Scalar multiplication of the base point (u = 9) |
| `MontgomeryPoint::scalar_mul(&self, scalar: &Scalar)`                 | Clamps the scalar, then runs the ladder         |

## Design notes

- Curve: y^2 = x^3 + 486662x^2 + x over F_p. Only the u-coordinate is used, so every 32-byte string is a valid public key.
- Clamping: clear the 3 low bits (multiple of the cofactor 8), clear bit 255, set bit 254. This blocks small-subgroup confinement and fixes the ladder iteration count.
- Reduction relies on 2^256 = 38 (mod 2p); the final reduction to mod p happens in `to_bytes`.
- Constant-time behaviour comes from construction (masked `cswap`, fixed loop bounds, Fermat inversion).

## Limitations

- Constant-time behaviour is not verified on the compiled output.
- No security audit.
