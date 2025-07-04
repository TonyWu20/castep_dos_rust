#![warn(missing_docs)]
#![allow(dead_code)]
//! Parsing implementation for `.pdos_weight`
/// All the data struct corresponding to the data in `.pdos_weight`
pub mod fundamental;

/// Helper functions for parsing
pub mod helper;

/// Parsing logics and function routines
pub mod pdos_weights_parser;

pub mod bands;

/// Projector preprocess
pub mod projectors;

/// calculation of PDOS
pub mod pdos_compute;
