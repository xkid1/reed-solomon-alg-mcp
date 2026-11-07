---
description: Audit existing tests and list missing cases without editing files.
agent: ask
tools: ["codebase", "search"]
---

Review the tests in `src/` and list gaps, ordered by risk. For each gap give: the behavior, why it matters, and a one-line sketch of the test. Pay special attention to decoder paths (Berlekamp-Massey degree check, Chien root count mismatch, residual-syndrome check), shortened codes, parity extremes, and hex/UTF-8 parsing in `main.rs`.
