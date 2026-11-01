//! Compile-fail tests for the attribute's error surface.
//!
//! These exercise the `proc_macro` entry point (argument parsing, input
//! validation) that cannot be reached from plain unit tests. Every case
//! fails before any filesystem access, so no fixture tree is needed here.
//! Gated off on Windows because trybuild's `.stderr` snapshots embed
//! platform-specific path separators.

#![cfg(not(windows))]

#[test]
fn ui() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}
