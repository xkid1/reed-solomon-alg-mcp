---
description: Teach a Reed-Solomon concept using this server's tools.
agent: ask
tools: ["codebase"]
---

Explain ${input:concept:e.g. syndromes, Berlekamp-Massey, shortened codes} for Reed-Solomon codes over GF(256).

Use a small concrete example with the server's conventions (poly 0x11d, alpha = 2, roots alpha^0..alpha^(nsym-1)). Show which `rs_*` tool call would reproduce each step, and finish with one common pitfall.
