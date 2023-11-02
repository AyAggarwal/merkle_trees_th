//! # Merkle Validation Library
//!
//! This library provides tools and structures for Merkle tree validation, error handling, and utility functions.
//!
//! ## Modules
//!
//! - **errors**: Contains custom error types `ValidationError` and `MerkleError` for detailed error reporting.
//!
//! - **merkle_tree**: Core module for Merkle tree structures and operations.
//!
//! - **utils**: Functions and helpers for Merkle tree operations.
//!
//! ## Usage
//!
//! Use this library for precise error handling and utilities in Merkle tree operations.
//!
//! Refer to each module's documentation for detailed information.
//!
//! **Note**: This library depends on external crates, including `hex`, for error handling.

pub mod errors;
pub mod merkle_tree;
pub mod utils;
