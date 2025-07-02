#![warn(missing_docs)]
#![allow(dead_code)]
//! Crate to parse `.bands` for eigenvalues of k-points.

use derive_builder::Builder;
mod parser;

pub use parser::{BandsParser, BandsParsingError};

use crate::fundamental::{
    BandStructure, EigenvalueVec, KpointVec, KpointWeight, SpinData, SpinPolarized,
};

//------------------------------------------
/// Type-safe electron count representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElectronCount {
    /// Non-spin-polarized: total electrons
    NonPolarized(f64),
    /// Spin-polarized: (up, down) electrons
    Polarized(f64, f64),
}

/// Type-safe Fermi energy representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FermiEnergy {
    /// Single Fermi energy
    NonPolarized(f64),
    /// Separate Fermi energies for spin channels
    Polarized(f64, f64),
}
#[derive(Debug, Clone, Copy, PartialEq)]
/// Unit cell vectors, in column-major order
pub struct LatticeVectors([[f64; 3]; 3]);
#[derive(Debug, Clone, Copy, PartialEq)]
/// K-point representation in `.bands`
pub struct KPoint {
    /// Global index in k-point mesh
    pub index: usize,
    /// Fractional coordinates in reciprocal space
    pub coords: [f64; 3],
    /// Integration weight in BZ sampling
    pub weight: f64,
}

impl KPoint {
    /// Constructor
    pub fn new(index: usize, coords: [f64; 3], weight: f64) -> Self {
        Self {
            index,
            coords,
            weight,
        }
    }
}

/// Complete band structure description
#[derive(Debug, Clone, PartialEq, Builder)]
#[builder()]
pub struct BandsFile {
    /// Spin-polarized settings
    pub spin_polarized: SpinPolarized,
    /// Lattice vectors in Angstroms (row-major)
    pub lattice_vectors: [[f64; 3]; 3],
    /// Fermi energy/energies in Hartree
    pub fermi_energy: FermiEnergy,
    /// Number of electrons in system
    pub electron_count: ElectronCount,
    /// All eigenvalues organized as [spin][kpoint][band]
    pub eigenvalues: Eigenvalues,
    /// Corresponding k-points
    pub kpoints: Vec<KPoint>,
}

impl BandsFile {
    /// Convert to the `BandStructure` with necessary informations
    /// for PDOS calculation
    pub fn to_band_structure(self) -> BandStructure {
        let spin_polarized = self.spin_polarized;
        let kpoint_weights = self
            .kpoints
            .iter()
            .map(|kpt| KpointWeight::new(kpt.weight))
            .collect::<KpointVec<KpointWeight>>();
        let eigenvalues = match self.eigenvalues {
            Eigenvalues::SpinPolarized([up, down]) => {
                let up = up
                    .into_iter()
                    .map(|eigen| eigen.into_iter().collect())
                    .collect::<KpointVec<EigenvalueVec<f64>>>();
                let down = down
                    .into_iter()
                    .map(|eigen| eigen.into_iter().collect())
                    .collect::<KpointVec<EigenvalueVec<f64>>>();
                SpinData::SpinPolarized([up, down])
            }
            Eigenvalues::NonPolarized(items) => {
                let items = items
                    .into_iter()
                    .map(|eigenvalues| eigenvalues.into_iter().collect::<EigenvalueVec<f64>>())
                    .collect::<KpointVec<EigenvalueVec<f64>>>();
                SpinData::NonPolarized(items)
            }
        };
        let fermi_energy = match self.fermi_energy {
            FermiEnergy::NonPolarized(f) => SpinData::NonPolarized(f),
            FermiEnergy::Polarized(up, down) => SpinData::SpinPolarized([up, down]),
        };
        BandStructure {
            spin_polarized,
            kpoint_weights,
            eigenvalues,
            fermi_energy,
        }
    }
}

/// Type-safe, spin-polarized dependent spin-major reprsentation
/// of eigenvalues of eigenvalues of k-points
/// - Indexing: ([spin])[k-point][nth-eigenvalue]
#[derive(Debug, Clone, PartialEq)]
pub enum Eigenvalues {
    /// SpinPolarized [nth spin][nth k-point][nth eigenvalue]
    SpinPolarized([Vec<Vec<f64>>; 2]),
    /// NonPolarized: [nth k-point][nth eigenvalue]
    NonPolarized(Vec<Vec<f64>>),
}

impl Eigenvalues {
    /// For case where spin_polarized: true
    pub fn as_spin_polarized(&self) -> Option<&[Vec<Vec<f64>>; 2]> {
        if let Self::SpinPolarized(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// For case where spin_polarized: false
    pub fn as_non_polarized(&self) -> Option<&Vec<Vec<f64>>> {
        if let Self::NonPolarized(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
