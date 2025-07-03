/// Implementation details of `AngularMomentum`
mod angular_momentum;
/// Data structs and enums related to band structure
/// Create wrapper for `Vec<T>`
/// So we can express the following nested data array:
/// [nth-kpoint][nth-eigenvalue][nth-orbital weight (f64)]
/// as:
/// KpointData<EigenvalueData<Vec<OrbitalWeight>>>
mod data_expression;

/// Traits implementations for custom data structs and enums
mod ergonomics_impl;

mod pdos_file;
/// Spin related structs and enums
mod spins;

const HATREE_TO_EV: f64 = 27.211396641308;

pub use angular_momentum::{AngularChannels, AngularMomentum, AngularMomentumConvertError};
pub use pdos_file::{Header, HeaderBuilder, HeaderBuilderError};
pub use pdos_file::{WeightsPerEigen, WeightsPerKPoint, WeightsPerSpin};

pub use data_expression::{
    EigenvalueVec, KpointVec, KpointWeight, OrbitalState, OrbitalWeight, OrbitalWeightVec, SpinData,
};
pub use spins::{NumSpins, NumSpinsConvertError, SpinIndex, SpinIndexConvertError, SpinPolarized};

#[derive(Debug, Clone, PartialEq)]
/// Parsed `.pdos_weights` with necessary data
pub struct PDOSWeights {
    /// Setting of spin polarization
    pub spin_polarized: SpinPolarized,
    /// Orbital metadatas
    pub orbital_states: Vec<OrbitalState>,
    /// The orbital weights, organized in a 4D array:
    /// [spin][k-point][eigenvalue][orbital weight]
    /// dim: 2   nkpt     n_eigen    n_orbs
    pub orbital_weights: SpinData<KpointVec<EigenvalueVec<OrbitalWeightVec>>>,
}

impl PDOSWeights {
    /// Constructor
    pub fn new(
        spin_polarized: SpinPolarized,
        orbital_states: Vec<OrbitalState>,
        orbital_weights: SpinData<KpointVec<EigenvalueVec<OrbitalWeightVec>>>,
    ) -> Self {
        Self {
            spin_polarized,
            orbital_states,
            orbital_weights,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Parsed `.bands` with necessary data
pub struct BandStructure {
    /// Setting of spin polarization
    pub spin_polarized: SpinPolarized,
    /// Fermi energy
    pub fermi_energy: SpinData<f64>,
    /// K-point weights in the same order of the k-points
    pub kpoint_weights: KpointVec<KpointWeight>,
    /// Eigenvalues, organized in a 3D array
    /// [spin][k-point][eigenvalue]
    pub eigenvalues: SpinData<KpointVec<EigenvalueVec<f64>>>,
}

impl BandStructure {
    /// Construtor
    pub fn new(
        spin_polarized: SpinPolarized,
        fermi_energy: SpinData<f64>,
        kpoint_weights: KpointVec<KpointWeight>,
        eigenvalues: SpinData<KpointVec<EigenvalueVec<f64>>>,
    ) -> Self {
        Self {
            spin_polarized,
            fermi_energy,
            kpoint_weights,
            eigenvalues,
        }
    }

    /// Returns min and max eigenvalue energy in eV
    /// For spin polarized bands, the min and max of two spins are compared
    /// and returns the smaller and larger one, respectively
    pub fn energy_range(&self) -> (f64, f64) {
        let spins_range = self.eigenvalues.map(|kpts| {
            kpts.iter()
                .map(|kpt_eigenvalues| {
                    // CASTEP has already sorted the eigenvalues in ascending order
                    let min = kpt_eigenvalues.first().unwrap() * HATREE_TO_EV;
                    let max = kpt_eigenvalues.last().unwrap() * HATREE_TO_EV;
                    (min, max)
                })
                .reduce(|(mut acc_min, mut acc_max), (curr_min, curr_max)| {
                    acc_min = acc_min.min(curr_min);
                    acc_max = acc_max.max(curr_max);
                    (acc_min, acc_max)
                })
                .unwrap()
        });
        match spins_range {
            SpinData::NonPolarized(e_range) => e_range,
            SpinData::SpinPolarized([(up_min, up_max), (down_min, down_max)]) => {
                (f64::min(up_min, down_min), f64::max(up_max, down_max))
            }
        }
    }
}

#[cfg(test)]
mod test;
