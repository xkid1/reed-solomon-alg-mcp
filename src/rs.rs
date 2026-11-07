//! Reed-Solomon over GF(2^8), primitive polynomial 0x11d, generator alpha = 2,
//! first consecutive root alpha^0. Polynomials are stored highest-degree first.

pub struct Gf {
    exp: [u8; 512],
    log: [u8; 256],
}

#[derive(Debug)]
pub struct Decoded {
    pub message: Vec<u8>,
    pub codeword: Vec<u8>,
    pub error_positions: Vec<usize>,
    pub error_magnitudes: Vec<u8>,
}

impl Gf {
    pub fn new() -> Self {
        let mut exp = [0u8; 512];
        let mut log = [0u8; 256];
        let mut x: u16 = 1;
        for (i, e) in exp.iter_mut().enumerate().take(255) {
            *e = x as u8;
            log[x as usize] = i as u8;
            x <<= 1;
            if x & 0x100 != 0 {
                x ^= 0x11d;
            }
        }
        for i in 255..512 {
            exp[i] = exp[i - 255];
        }
        Gf { exp, log }
    }
    pub fn mul(&self, a: u8, b: u8) -> u8 {
        if a == 0 || b == 0 {
            0
        } else {
            self.exp[self.log[a as usize] as usize + self.log[b as usize] as usize]
        }
    }
    pub fn inv(&self, a: u8) -> Result<u8, String> {
        if a == 0 {
            return Err("zero has no multiplicative inverse".into());
        }
        Ok(self.exp[255 - self.log[a as usize] as usize])
    }
    pub fn div(&self, a: u8, b: u8) -> Result<u8, String> {
        if b == 0 {
            return Err("division by zero".into());
        }
        if a == 0 {
            return Ok(0);
        }
        Ok(self.exp[(self.log[a as usize] as usize + 255 - self.log[b as usize] as usize) % 255])
    }
    pub fn pow(&self, a: u8, n: u32) -> u8 {
        if n == 0 {
            return 1;
        }
        if a == 0 {
            return 0;
        }
        self.exp[(self.log[a as usize] as u64 * n as u64 % 255) as usize]
    }
    pub fn alpha(&self, i: usize) -> u8 {
        self.exp[i % 255]
    }
    pub fn log_of(&self, a: u8) -> Option<u8> {
        if a == 0 {
            None
        } else {
            Some(self.log[a as usize])
        }
    }

    fn poly_scale(&self, p: &[u8], x: u8) -> Vec<u8> {
        p.iter().map(|&c| self.mul(c, x)).collect()
    }
    fn poly_add(&self, p: &[u8], q: &[u8]) -> Vec<u8> {
        let n = p.len().max(q.len());
        let mut r = vec![0u8; n];
        for (i, &c) in p.iter().enumerate() {
            r[n - p.len() + i] ^= c;
        }
        for (i, &c) in q.iter().enumerate() {
            r[n - q.len() + i] ^= c;
        }
        r
    }
    pub fn poly_mul(&self, p: &[u8], q: &[u8]) -> Vec<u8> {
        let mut r = vec![0u8; p.len() + q.len() - 1];
        for (j, &qc) in q.iter().enumerate() {
            for (i, &pc) in p.iter().enumerate() {
                r[i + j] ^= self.mul(pc, qc);
            }
        }
        r
    }
    pub fn poly_eval(&self, p: &[u8], x: u8) -> u8 {
        let mut y = p[0];
        for &c in &p[1..] {
            y = self.mul(y, x) ^ c;
        }
        y
    }

    pub fn generator_poly(&self, nsym: usize) -> Vec<u8> {
        let mut g = vec![1u8];
        for i in 0..nsym {
            g = self.poly_mul(&g, &[1, self.alpha(i)]);
        }
        g
    }

    pub fn encode(&self, msg: &[u8], nsym: usize) -> Result<Vec<u8>, String> {
        check_params(msg.len(), nsym, false)?;
        let gen = self.generator_poly(nsym);
        let mut out = msg.to_vec();
        out.resize(msg.len() + nsym, 0);
        for i in 0..msg.len() {
            let coef = out[i];
            if coef != 0 {
                for j in 1..gen.len() {
                    out[i + j] ^= self.mul(gen[j], coef);
                }
            }
        }
        let mut cw = msg.to_vec();
        cw.extend_from_slice(&out[msg.len()..]);
        Ok(cw)
    }

    /// Returns [0, S0, S1, ..., S_{nsym-1}] (leading pad, as used by the BM/Forney code below).
    pub fn syndromes(&self, cw: &[u8], nsym: usize) -> Vec<u8> {
        let mut s = vec![0u8];
        for i in 0..nsym {
            s.push(self.poly_eval(cw, self.alpha(i)));
        }
        s
    }

    fn error_locator(&self, synd: &[u8], nsym: usize) -> Result<Vec<u8>, String> {
        let mut err_loc = vec![1u8];
        let mut old_loc = vec![1u8];
        let shift = synd.len().saturating_sub(nsym);
        for i in 0..nsym {
            let k = i + shift;
            let mut delta = synd[k];
            for j in 1..err_loc.len() {
                delta ^= self.mul(err_loc[err_loc.len() - 1 - j], synd[k - j]);
            }
            old_loc.push(0);
            if delta != 0 {
                if old_loc.len() > err_loc.len() {
                    let new_loc = self.poly_scale(&old_loc, delta);
                    old_loc = self.poly_scale(&err_loc, self.inv(delta)?);
                    err_loc = new_loc;
                }
                err_loc = self.poly_add(&err_loc, &self.poly_scale(&old_loc, delta));
            }
        }
        while err_loc.len() > 1 && err_loc[0] == 0 {
            err_loc.remove(0);
        }
        let errs = err_loc.len() - 1;
        if errs * 2 > nsym {
            return Err(format!(
                "too many errors: locator degree {} exceeds correction capacity {}",
                errs,
                nsym / 2
            ));
        }
        Ok(err_loc)
    }

    fn find_errors(&self, err_loc_rev: &[u8], n: usize) -> Result<Vec<usize>, String> {
        let errs = err_loc_rev.len() - 1;
        let mut pos = vec![];
        for i in 0..n {
            if self.poly_eval(err_loc_rev, self.alpha(i)) == 0 {
                pos.push(n - 1 - i);
            }
        }
        if pos.len() != errs {
            return Err("could not locate errors (too many errors, decoding failure)".into());
        }
        Ok(pos)
    }

    fn correct(&self, cw: &[u8], synd: &[u8], pos: &[usize]) -> Result<(Vec<u8>, Vec<u8>), String> {
        let n = cw.len();
        let coef_pos: Vec<usize> = pos.iter().map(|&p| n - 1 - p).collect();
        // errata locator
        let mut loc = vec![1u8];
        for &c in &coef_pos {
            loc = self.poly_mul(&loc, &self.poly_add(&[1], &[self.alpha(c), 0]));
        }
        // evaluator: (synd_rev * loc) mod x^(deg+1)
        let synd_rev: Vec<u8> = synd.iter().rev().cloned().collect();
        let prod = self.poly_mul(&synd_rev, &loc);
        let keep = loc.len();
        let eval: Vec<u8> = if prod.len() > keep {
            prod[prod.len() - keep..].to_vec()
        } else {
            prod
        };
        let xs: Vec<u8> = coef_pos.iter().map(|&c| self.alpha(c)).collect();
        let mut out = cw.to_vec();
        let mut mags = vec![];
        for (i, &xi) in xs.iter().enumerate() {
            let xi_inv = self.inv(xi)?;
            let mut prime = 1u8;
            for (j, &xj) in xs.iter().enumerate() {
                if j != i {
                    prime = self.mul(prime, 1 ^ self.mul(xi_inv, xj));
                }
            }
            let y = self.mul(xi, self.poly_eval(&eval, xi_inv));
            let m = self.div(y, prime)?;
            out[pos[i]] ^= m;
            mags.push(m);
        }
        Ok((out, mags))
    }

    pub fn decode(&self, cw: &[u8], nsym: usize) -> Result<Decoded, String> {
        check_params(cw.len(), nsym, true)?;
        let synd = self.syndromes(cw, nsym);
        let k = cw.len() - nsym;
        if synd.iter().all(|&s| s == 0) {
            return Ok(Decoded {
                message: cw[..k].to_vec(),
                codeword: cw.to_vec(),
                error_positions: vec![],
                error_magnitudes: vec![],
            });
        }
        let loc = self.error_locator(&synd, nsym)?;
        let rev: Vec<u8> = loc.iter().rev().cloned().collect();
        let mut pos = self.find_errors(&rev, cw.len())?;
        pos.sort();
        let (fixed, mags) = self.correct(cw, &synd, &pos)?;
        if self.syndromes(&fixed, nsym).iter().any(|&s| s != 0) {
            return Err("decoding failure: residual syndromes non-zero after correction".into());
        }
        Ok(Decoded {
            message: fixed[..k].to_vec(),
            codeword: fixed,
            error_positions: pos,
            error_magnitudes: mags,
        })
    }
}

fn check_params(len: usize, nsym: usize, is_cw: bool) -> Result<(), String> {
    if nsym == 0 {
        return Err("parity_symbols must be >= 1".into());
    }
    let total = if is_cw { len } else { len + nsym };
    if total > 255 {
        return Err(format!(
            "codeword length {} exceeds 255 (the maximum for GF(2^8))",
            total
        ));
    }
    if is_cw && len <= nsym {
        return Err("codeword must be longer than parity_symbols".into());
    }
    if !is_cw && len == 0 {
        return Err("message must not be empty".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_with_errors() {
        let gf = Gf::new();
        let msg = b"Hello, Reed-Solomon!".to_vec();
        let cw = gf.encode(&msg, 10).unwrap();
        assert!(gf.syndromes(&cw, 10).iter().all(|&s| s == 0));
        let mut bad = cw.clone();
        for (i, p) in [0usize, 7, 13, 22, 29].iter().enumerate() {
            bad[*p] ^= 0x11 * (i as u8 + 1);
        }
        let d = gf.decode(&bad, 10).unwrap();
        assert_eq!(d.message, msg);
        assert_eq!(d.codeword, cw);
        assert_eq!(d.error_positions, vec![0, 7, 13, 22, 29]);
    }

    #[test]
    fn too_many_errors_detected_or_wrong() {
        let gf = Gf::new();
        let cw = gf.encode(b"abcdefgh", 4).unwrap();
        let mut bad = cw.clone();
        for b in bad.iter_mut().take(5) {
            *b ^= 0x55;
        }
        match gf.decode(&bad, 4) {
            Err(_) => {}
            Ok(d) => assert_ne!(d.codeword, cw),
        }
    }

    #[test]
    fn known_generator() {
        let gf = Gf::new();
        // (x+1)(x+2) = x^2 + 3x + 2
        assert_eq!(gf.generator_poly(2), vec![1, 3, 2]);
    }

    /// Tiny deterministic PRNG so tests are reproducible without extra crates.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 33
        }
    }

    #[test]
    fn every_single_error_position_and_value_is_corrected() {
        let gf = Gf::new();
        let cw = gf.encode(b"exhaustive", 4).unwrap();
        for pos in 0..cw.len() {
            for mag in 1..=255u8 {
                let mut bad = cw.clone();
                bad[pos] ^= mag;
                let d = gf.decode(&bad, 4).unwrap();
                assert_eq!(d.codeword, cw, "pos {pos} mag {mag}");
                assert_eq!(d.error_positions, vec![pos]);
                assert_eq!(d.error_magnitudes, vec![mag]);
            }
        }
    }

    #[test]
    fn random_errors_up_to_capacity_roundtrip() {
        let gf = Gf::new();
        let mut rng = Lcg(42);
        for _ in 0..300 {
            let nsym = 2 + (rng.next() % 30) as usize;
            let k = 1 + (rng.next() % 100) as usize;
            let msg: Vec<u8> = (0..k).map(|_| rng.next() as u8).collect();
            let cw = gf.encode(&msg, nsym).unwrap();
            let t = (rng.next() as usize) % (nsym / 2 + 1);
            let mut bad = cw.clone();
            let mut used = std::collections::BTreeSet::new();
            while used.len() < t {
                used.insert(rng.next() as usize % cw.len());
            }
            for &p in &used {
                bad[p] ^= 1 + (rng.next() % 255) as u8;
            }
            let d = gf.decode(&bad, nsym).unwrap();
            assert_eq!(d.codeword, cw);
            assert_eq!(d.error_positions, used.into_iter().collect::<Vec<_>>());
        }
    }

    #[test]
    fn field_axioms() {
        let gf = Gf::new();
        for a in 1..=255u8 {
            assert_eq!(gf.mul(a, gf.inv(a).unwrap()), 1);
            assert_eq!(gf.div(a, a).unwrap(), 1);
        }
        assert_eq!(gf.pow(2, 8), 0x1d); // alpha^8 reduces by 0x11d
        assert!(gf.inv(0).is_err());
    }

    #[test]
    fn input_validation() {
        let gf = Gf::new();
        assert!(gf.encode(b"", 4).is_err());
        assert!(gf.encode(b"x", 0).is_err());
        assert!(gf.encode(&[0u8; 250], 10).is_err()); // 260 > 255
        assert!(gf.decode(&[1, 2, 3], 3).is_err()); // not longer than parity
    }
}
