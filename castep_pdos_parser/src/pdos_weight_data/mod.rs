use serde::{Deserialize, Serialize};

/// Implementation details of `AngularMomentum`
mod angular_momentum;
mod header;
/// Implementation details of `NumSpins`
mod num_spins;
/// Orbital specified by species, ion, and angular mometum;
mod orbital;
/// Implementation details of `SpinIndex`
mod spin_index;

pub use angular_momentum::AngularMomentumConvertError;
pub use header::{Header, HeaderBuilder, HeaderBuilderError};
pub use num_spins::NumSpinsConvertError;
pub use orbital::Orbital;
pub use spin_index::SpinIndexConvertError;

#[derive(Debug)]
/// A struct represent the `.pdos_weight`
pub struct PDOSWeight {
    /// Header section, contains the overview stats
    pub header: Header,
    /// PDOS weight data of each k-point
    pub kpoints: Vec<WeightsPerKPoint>,
}

impl PDOSWeight {
    /// New
    pub fn new(header: Header, kpoints: Vec<WeightsPerKPoint>) -> Self {
        Self { header, kpoints }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// The number of spins only has two variant:
pub enum NumSpins {
    /// - One: when `spin_polarised : false`
    One,
    /// - Two: when `spin_polarised : true`
    Two,
}

impl NumSpins {
    /// Returns spin count, either 1 or 2
    pub fn spin_count(&self) -> usize {
        match self {
            NumSpins::One => 1,
            NumSpins::Two => 2,
        }
    }
}

#[derive(Debug, Clone)]
/// Data written for each k-point
pub struct WeightsPerKPoint {
    /// global k-point index
    pub index: u32,
    /// k-point coordinate
    pub kpoint: [f64; 3],
    /// spin components
    pub spins: Vec<WeightsPerSpin>,
}

impl WeightsPerKPoint {
    /// New
    pub fn new(index: u32, kpoint: [f64; 3], spins: Vec<WeightsPerSpin>) -> Self {
        Self {
            index,
            kpoint,
            spins,
        }
    }
}

#[derive(Debug, Clone)]
/// Grouped data written by `CASTEP` for every
/// spin components
pub struct WeightsPerSpin {
    /// Spin 1 or 2
    pub index: SpinIndex,
    /// Number of occupied bands
    pub nbands_occ: u32,
    /// Grouped data of pdos weight for every eigenvalues
    pub bands: Vec<BandData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// The number of spins only has two variant:
pub enum SpinIndex {
    /// - One: default
    One,
    /// - Two: Only when `spin_polarised : true`
    Two,
}

impl WeightsPerSpin {
    /// New
    pub fn new(index: SpinIndex, nbands_occ: u32, bands: Vec<BandData>) -> Self {
        Self {
            index,
            nbands_occ,
            bands,
        }
    }
}

#[derive(Debug, Clone)]
/// PDOS weights for the band at each eigenvalues
pub struct BandData {
    /// PDOS weight, flattened according to the projectors
    pub weights: Vec<f64>,
}

impl BandData {
    /// New
    pub fn new(weights: Vec<f64>) -> Self {
        Self { weights }
    }
}

/// The angular momentum expression
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AngularMomentum {
    /// l = 0
    S,
    /// l = 1
    P,
    /// l = 2
    D,
    /// l = 3
    F,
}
