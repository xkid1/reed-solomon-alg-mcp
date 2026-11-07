---
description: Write unit tests for a function, file, or MCP tool.
agent: rs-unit-test-writer
tools: ["codebase", "editFiles", "runCommands"]
---

Write unit tests for ${input:target:function, file, or tool name}.

Cover: happy path, boundaries (nsym = 1, n = 255, empty/short input), invalid input, and over-capacity behavior where relevant. Assert exact values. Run `cargo test` and summarize what each new test protects against.
