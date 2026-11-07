---
description: Rust conventions for this repository.
applyTo: "**/*.rs"
---

- Edition 2021, builds on Rust 1.75+.
- No `unsafe`. No `unwrap()`/`expect()` on values derived from user input; return `Result<_, String>`.
- Use `u8` for field elements and `usize` for lengths and positions; cast `u8` to `usize` explicitly when indexing tables.
- Keep GF operations in `rs.rs` on `Gf` methods; keep JSON handling out of `rs.rs`.
- Add a `#[cfg(test)]` test for every new coding routine, including a failure case (too many errors).
- Run `cargo fmt` and `cargo clippy -- -D warnings` before finishing.
