use std::ops::{Deref, DerefMut, Index, IndexMut};

use rayon::iter::{FromParallelIterator, ParallelIterator};

use super::{
    EigenvalueVec, KpointVec, KpointWeight, OrbitalWeight, OrbitalWeightVec, SpinData, SpinIndex,
};

impl<T> Deref for KpointVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for KpointVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Index<usize> for KpointVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for KpointVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T> Deref for EigenvalueVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for EigenvalueVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Index<usize> for EigenvalueVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for EigenvalueVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Deref for OrbitalWeightVec {
    type Target = Vec<OrbitalWeight>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OrbitalWeightVec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Index<usize> for OrbitalWeightVec {
    type Output = OrbitalWeight;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for OrbitalWeightVec {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl From<f64> for OrbitalWeight {
    fn from(value: f64) -> Self {
        let checked_value = if value < -1.0e-6 { 0.0 } else { value };
        Self(checked_value)
    }
}

impl From<OrbitalWeight> for f64 {
    fn from(value: OrbitalWeight) -> Self {
        value.0
    }
}

impl From<f64> for KpointWeight {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<KpointWeight> for f64 {
    fn from(value: KpointWeight) -> Self {
        value.0
    }
}

impl OrbitalWeight {
    /// Natural access to value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Constructor
    pub fn new(value: f64) -> Self {
        let checked_value = if value < -1.0e-6 { 0.0 } else { value };
        Self(checked_value)
    }
}

impl KpointWeight {
    /// Natural access to value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Constructor
    pub fn new(value: f64) -> Self {
        Self(value)
    }
}

impl<T> EigenvalueVec<T> {
    /// Constructor
    pub fn new(data: Vec<T>) -> Self {
        Self(data)
    }

    /// Into inner type
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl OrbitalWeightVec {
    /// Constructor
    pub fn new(data: Vec<OrbitalWeight>) -> Self {
        Self(data)
    }

    /// Into inner type
    pub fn into_inner(self) -> Vec<OrbitalWeight> {
        self.0
    }
}

impl<T> KpointVec<T> {
    /// Constructor
    pub fn new(data: Vec<T>) -> Self {
        Self(data)
    }

    /// Into inner type
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<U> FromIterator<U> for KpointVec<U> {
    fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<U> FromIterator<U> for EigenvalueVec<U> {
    fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl FromIterator<OrbitalWeight> for OrbitalWeightVec {
    fn from_iter<T: IntoIterator<Item = OrbitalWeight>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T> SpinData<T> {
    /// Access data based on spin configuration
    pub fn get(&self, spin: SpinIndex) -> Option<&T> {
        match (self, spin) {
            (SpinData::NonPolarized(data), SpinIndex::One) => Some(data),
            (SpinData::NonPolarized(_), SpinIndex::Two) => None,
            (SpinData::SpinPolarized([up, _]), SpinIndex::One) => Some(up),
            (SpinData::SpinPolarized([_, down]), SpinIndex::Two) => Some(down),
        }
    }

    /// Map data while preserving spin structure
    /// This helper function directly map on the `T` that for each spin channel
    pub fn map<U, F: Fn(&T) -> U>(&self, f: F) -> SpinData<U> {
        match self {
            SpinData::NonPolarized(data) => SpinData::NonPolarized(f(data)),
            SpinData::SpinPolarized([up, down]) => SpinData::SpinPolarized([f(up), f(down)]),
        }
    }

    /// Execute function on the data array of k-points for each spin component
    pub fn for_each<F: FnMut(&T)>(&self, mut f: F) {
        match self {
            SpinData::NonPolarized(data) => f(data),
            SpinData::SpinPolarized([up, down]) => {
                f(up);
                f(down);
            }
        }
    }

    /// Map data on two `SpinData` with same spin polarization settings with a function
    /// act on the two different data types and produce one new `SpinData`
    pub fn map_pair<U, V, F: Fn(&T, &U) -> V>(&self, rhs: &SpinData<U>, f: F) -> SpinData<V> {
        match (self, rhs) {
            (SpinData::NonPolarized(data), SpinData::NonPolarized(rhs_data)) => {
                SpinData::NonPolarized(f(data, rhs_data))
            }
            (SpinData::SpinPolarized([up, down]), SpinData::SpinPolarized([rhs_up, rhs_down])) => {
                SpinData::SpinPolarized([f(up, rhs_up), f(down, rhs_down)])
            }
            _ => unimplemented!(),
        }
    }
}
impl<T> SpinData<KpointVec<EigenvalueVec<T>>> {
    /// Map data for eigenvalues
    pub fn map_on_data_of_eigenvalue<U, F>(&self, f: F) -> SpinData<KpointVec<EigenvalueVec<U>>>
    where
        F: Fn(&T) -> U,
        U: Clone,
    {
        match self {
            SpinData::NonPolarized(kpt_eigen_data) => {
                SpinData::NonPolarized(map_for_values_of_eigen(kpt_eigen_data, &f))
            }
            SpinData::SpinPolarized([up, down]) => SpinData::SpinPolarized([
                map_for_values_of_eigen(up, &f),
                map_for_values_of_eigen(down, &f),
            ]),
        }
    }
}

/// This helper function map on the data of each eigenvalue
fn map_for_values_of_eigen<T, U, F>(
    channel: &KpointVec<EigenvalueVec<T>>,
    f: &F,
) -> KpointVec<EigenvalueVec<U>>
where
    U: Clone,
    F: Fn(&T) -> U,
{
    channel
        .iter()
        .map(|eigens| eigens.iter().map(f).collect())
        .collect()
}

impl Extend<OrbitalWeightVec> for KpointVec<OrbitalWeightVec> {
    fn extend<U: IntoIterator<Item = OrbitalWeightVec>>(&mut self, iter: U) {
        self.0.extend(iter);
    }
}

impl<T> Extend<EigenvalueVec<T>> for KpointVec<EigenvalueVec<T>> {
    fn extend<U: IntoIterator<Item = EigenvalueVec<T>>>(&mut self, iter: U) {
        self.0.extend(iter);
    }
}

impl<T: Send> FromParallelIterator<T> for EigenvalueVec<T> {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: rayon::prelude::IntoParallelIterator<Item = T>,
    {
        Self(par_iter.into_par_iter().collect())
    }
}
