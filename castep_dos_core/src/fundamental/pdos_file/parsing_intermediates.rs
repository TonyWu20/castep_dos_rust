use crate::fundamental::SpinIndex;

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
    pub bands: Vec<WeightsPerEigen>,
}

impl WeightsPerSpin {
    /// New
    pub fn new(index: SpinIndex, nbands_occ: u32, bands: Vec<WeightsPerEigen>) -> Self {
        Self {
            index,
            nbands_occ,
            bands,
        }
    }
}

#[derive(Debug, Clone)]
/// PDOS weights for the band at each eigenvalues
pub struct WeightsPerEigen {
    /// PDOS weight, flattened according to the projectors
    pub weights: Vec<f64>,
}

impl WeightsPerEigen {
    /// New
    pub fn new(weights: Vec<f64>) -> Self {
        Self { weights }
    }
}
