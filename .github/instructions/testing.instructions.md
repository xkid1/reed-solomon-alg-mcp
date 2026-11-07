---
description: Conventions for unit tests in this repository.
applyTo: "src/**/*.rs"
---

- Unit tests live in `#[cfg(test)] mod tests` at the bottom of the file they test. Integration-style stdio checks go in `tests/` if added.
- Name tests as behavior: `every_single_error_position_and_value_is_corrected`, not `test1`.
- One behavior per test; assert on concrete values (exact positions and magnitudes), not just `is_ok()`.
- Every public routine needs: a happy path, a boundary (min/max length, nsym 1, n = 255), an invalid-input case, and an over-capacity case.
- Over-capacity decoding may legitimately return `Err` or miscorrect to another valid codeword. Assert `Err(_)` or `codeword != original`, never that it always errors.
- Use the deterministic `Lcg` helper for random data with a fixed seed. No new dev-dependencies.
- Add `assert_eq!` messages with the failing inputs in loops (`"pos {pos} mag {mag}"`).
- Keep each test under about 1 second in debug mode.
- Run `cargo test` before and after changes.
