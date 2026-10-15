//! Small helpers shared by the ported tools.
//!
//! Every helper in this module mirrors the behaviour of the corresponding
//! Deno/TypeScript helper that the original tools used, so that the generated
//! output stays byte-for-byte compatible.

pub mod chars;
pub mod fmt;
pub mod js;
pub mod string_utils;
pub mod table;
