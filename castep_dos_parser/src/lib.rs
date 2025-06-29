#![warn(missing_docs)]
#![allow(dead_code)]
//! Parsing implementation for `.pdos_weight`
/// All the data struct corresponding to the data in `.pdos_weight`
pub mod pdos_weight_data;

/// Helper functions for parsing
pub mod helper;

/// Parsing logics and function routines
pub mod parser;

// /// Filtering projectors for PDOS calculations
// pub mod projectors {
//     mod config;
//     mod engine {}
//     pub use config::{ConfigError, EnergyGridConfig, PDOSConfig, ProjectorConfig};
// }
