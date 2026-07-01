---
title: WebAssembly (WASM)
type: glossary-entry
status: active
tags: [web, runtime, future]
aliases: [WASM, WebAssembly, wasm]
description: "A binary instruction format for a stack-based virtual machine, enabling VantaDB to run in browsers."
links: "[[README.md]]"
---

# WebAssembly (WASM)

WebAssembly (WASM) is a binary instruction format designed to be a compilation target for languages like Rust, allowing code to run on the web at near-native speed.

## Future Use in VantaDB

Because VantaDB is written in Rust, compiling to WASM allows for a "VantaDB-Cloud" or edge version that runs directly inside the user's browser using OPFS (Origin Private File System) for persistence. This heavily aligns with VantaDB's [[local-first]] strategy, removing the need for dedicated backend vector infrastructure.

## See Also
- [[local-first|Local-first Strategy]]
- [[ROADMAP.md]]
