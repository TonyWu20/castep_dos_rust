use derive_builder::Builder;
use thiserror::Error;

#[derive(Debug)]
/// A struct represent the `.pdos_weight`
pub struct PDOSWeight {
    /// Header section, contains the overview stats
    pub header: Header,
    /// PDOS weight data of each k-point
    pub kpoints: Vec<KPoint>,
}

impl PDOSWeight {
    /// New
    pub fn new(header: Header, kpoints: Vec<KPoint>) -> Self {
        Self { header, kpoints }
    }
}

#[derive(Debug, Builder)]
#[builder()]
/// The header sections of the `.pdos_weight` file
pub struct Header {
    /// Total number of kpoints
    pub total_kpoints: u32,
    /// Number of spins in the system, either one
    /// or two
    pub num_spins: NumSpins,
    /// Number of orbitals
    pub num_orbitals: u32,
    /// Maximum number of bands
    /// This value correspond to the maximum of "Total electrons" in `.bands`
    pub max_bands: u32,
    /// A `vec` holding the species ID from the unit cell definition
    /// in `.cell` or binary output `.check` and `.castep_bin`
    pub orbital_species: Vec<u32>,
    /// A `vec` holding the "rank" ID (indices of atoms of the same species, start
    /// from 1) for the corresponding orbitals. The ranks comes from the unit cell
    /// definition in `.cell` or binary output `.check` and `.castep_bin`
    pub orbital_ion: Vec<u32>,
    /// A `vec` holding the angular momentum value `l` of the corresponding orbital.
    pub orbital_am: Vec<u32>,
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

#[derive(Debug, Error)]
#[error("Try to convert to `NumSpins` from value out of 1 and 2.")]
/// Error for implementation of `TryFrom<u32>` for NumSpins
pub struct NumSpinsConvertError;

impl TryFrom<u32> for NumSpins {
    type Error = NumSpinsConvertError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            _ => Err(NumSpinsConvertError),
        }
    }
}

#[derive(Debug, Clone)]
/// Data written for each k-point
pub struct KPoint {
    /// global k-point index
    pub index: u32,
    /// k-point coordinate (x)
    pub kx: f64,
    /// k-point coordinate (y)
    pub ky: f64,
    /// k-point coordinate (z)
    pub kz: f64,
    /// spin components
    pub spins: Vec<WeightsPerSpin>,
}

impl KPoint {
    /// New
    pub fn new(index: u32, kx: f64, ky: f64, kz: f64, spins: Vec<WeightsPerSpin>) -> Self {
        Self {
            index,
            kx,
            ky,
            kz,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// The number of spins only has two variant:
pub enum SpinIndex {
    /// - One: default
    One,
    /// - Two: Only when `spin_polarised : true`
    Two,
}

#[derive(Debug, Error)]
#[error("Try to convert to `NumSpins` from value out of 1 and 2.")]
/// Error for implementation of `TryFrom<u32>` for NumSpins
pub struct SpinIndexConvertError;

impl TryFrom<u32> for SpinIndex {
    type Error = SpinIndexConvertError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            _ => Err(SpinIndexConvertError),
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
