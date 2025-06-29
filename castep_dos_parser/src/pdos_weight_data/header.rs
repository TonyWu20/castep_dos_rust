use super::{AngularMomentum, NumSpins};
use derive_builder::Builder;

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
    pub orbital_am: Vec<AngularMomentum>,
}
