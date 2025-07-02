use super::AngularMomentum;

/// ---------------------------------------
/// Represent every `T` for a spin is data belongs to a k-point
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KpointVec<T>(pub Vec<T>);
/// Represent every `T` for a k-point belongs to an eigenvalue
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EigenvalueVec<T>(pub Vec<T>);
/// Represent an array of Orbital Weight
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OrbitalWeightVec(pub Vec<OrbitalWeight>);
/// Represent the orbtial weights (`f64`)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct OrbitalWeight(pub f64);

/// Wrapper newtype of k-point weight (`f64`)
/// This is the only thing we need to calculate dos,
/// throw away the k-point index and coordinate
/// information from `.bands`
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct KpointWeight(pub f64);

/// Unified spin polarization handling
/// Whether the spin is polarized or not, the data we have
/// always carry a dimension of spin. Since k-point is always
/// a dimension higher than the eigenvalue and others,
/// the inner type of the variants is inherently `KpointVec<T>`
#[derive(Debug, Clone, PartialEq)]
pub enum SpinData<T> {
    /// Non polarized variant
    NonPolarized(T),
    /// Spin polarized has the extra dimension
    SpinPolarized([T; 2]),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
/// Orbital representation for better searching
pub struct OrbitalState {
    /// Species id
    pub species_id: u32,
    /// Ion index
    pub ion_id: u32,
    /// Angular momentum
    pub angular_momentum: AngularMomentum,
}

impl OrbitalState {
    /// Constructor
    pub fn new(species_id: u32, ion_id: u32, angular_momentum: AngularMomentum) -> Self {
        Self {
            species_id,
            ion_id,
            angular_momentum,
        }
    }
}
