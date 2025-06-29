use super::AngularMomentum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Orbital specified by species, ion, and angular momentum
/// This is for convenience in projector extractions
/// from the parsed orbital weight data.
pub struct Orbital {
    species: u32,
    ion_id: u32,
    angular_momentum: AngularMomentum,
}

impl Orbital {
    /// Constructor
    pub fn new(species: u32, ion_rank: u32, angular_momentum: AngularMomentum) -> Self {
        Self {
            species,
            ion_id: ion_rank,
            angular_momentum,
        }
    }

    /// Returns the species id.
    pub fn species(&self) -> u32 {
        self.species
    }

    /// Returns the ion id.
    pub fn ion_id(&self) -> u32 {
        self.ion_id
    }

    /// Returns the angular_momentum
    pub fn angular_momentum(&self) -> AngularMomentum {
        self.angular_momentum
    }
}
