use crate::fundamental::{AngularMomentum, NumSpins, OrbitalState, SpinPolarized};
use derive_builder::Builder;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

#[derive(Debug, Builder, Clone)]
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

impl Header {
    /// Determine if this `.pdos_weights` calculation
    /// was spin-polarized
    pub fn spin_polarized(&self) -> SpinPolarized {
        match self.num_spins {
            NumSpins::One => SpinPolarized::False,
            NumSpins::Two => SpinPolarized::True,
        }
    }

    /// The original three arrays are not cache-friendly
    /// for projector filtering, which requires comparing the species,
    /// ion id, and angular momentum together.
    /// Hence we group them into array of struct `OrbitalState`
    pub fn extract_orbital_states(&self) -> Vec<OrbitalState> {
        self.orbital_species
            .par_iter()
            .zip(self.orbital_ion.par_iter())
            .zip(self.orbital_am.par_iter())
            .map(|((species_id, ion_id), angular_momentum)| {
                OrbitalState::new(*species_id, *ion_id, *angular_momentum)
            })
            .collect()
    }
}
