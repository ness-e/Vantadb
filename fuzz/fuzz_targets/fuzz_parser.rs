#![no_main]
//! Fuzz target: VantaDB LISP/query parser.
//!
//! Exercises three parser entry points against arbitrary byte sequences to
//! detect panics, infinite loops, and memory safety violations.
//!
//! Run on Linux with nightly toolchain:
//!   cargo +nightly fuzz run fuzz_parser -- -max_total_time=300

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Only process valid UTF-8 — all parsers operate on string input.
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    // 1. LISP expression parser (highest priority: recursive descent, deepest attack surface)
    let _ = vantadb::parser::lisp::parse(input);

    // 2. Query planner query parser
    let _ = vantadb::parser::parse_query(input);

    // 3. Statement-level parser (DDL-style)
    let _ = vantadb::parser::parse_statement(input);
});
