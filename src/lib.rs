//! # openapi-generator-cli
//!
//! Rust CLI wrapper for [OpenAPI Generator](https://openapi-generator.tech/).
//!
//! This crate downloads the latest OpenAPI Generator CLI JAR (and optionally a JRE)
//! at build time and provides helpers to locate them at runtime.

pub use path::{bundled_jre_path, java_path, openapi_generator_jar_path};

pub mod path;
