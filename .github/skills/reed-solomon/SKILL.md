---
name: reed-solomon
description: Reed-Solomon error-correction knowledge for GF(2^8): choosing n/k/parity, encoding, syndrome decoding, error vs erasure capacity, shortened codes, and debugging decoder failures. Use when sizing a code, explaining RS, or investigating wrong or failed decodes.
---

# Reed-Solomon over GF(256)

## Conventions in this repo

- Field: GF(2^8), primitive polynomial `0x11d`, alpha = 2.
- Generator: `g(x) = (x-a^0)(x-a^1)...(x-a^(nsym-1))`. Systematic encoding: message bytes then `nsym` parity bytes.
- Polynomials highest-degree first. Syndrome vector is `[0, S0..S(nsym-1)]`.

## Sizing a code

- RS(n,k): n <= 255, parity p = n-k. Corrects `t = floor(p/2)` unknown-position errors, or `p` erasures, or any mix with `2*errors + erasures <= p`.
- Minimum distance is p+1. Rate is k/n.
- Burst tolerance: each corrupted byte counts once, so an 8-bit-aligned burst of up to 8t bits costs only about t symbols.
- n < 255 is a shortened code (implicit leading zeros); the same math applies.

## Decoding pipeline

1. Syndromes `S_j = r(a^j)`; all zero means valid.
2. Berlekamp-Massey gives the error locator.
3. Chien search finds roots, which map to byte positions.
4. Forney computes error magnitudes; XOR them out.
5. Recompute syndromes to verify.

## Debugging checklist

- Wrong decode with no error: more than t errors can miscorrect to another valid codeword. This is inherent; add a CRC outside the code if detection matters.
- Position off by one: confirm highest-degree-first ordering and that position 0 is the first message byte.
- Mismatch with another library: check polynomial (0x11d vs 0x11b), first consecutive root (0 vs 1), and parity placement.
- Length error: n must be <= 255.

## Quick checks with the server

- `rs_code_parameters` for capacity, `rs_encode` then `rs_inject_errors` then `rs_decode` for a round trip, `gf256_math` for field spot checks.
