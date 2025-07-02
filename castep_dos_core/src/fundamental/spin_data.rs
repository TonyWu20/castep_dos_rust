/// Unified spin polarization handling
pub enum SpinData<T> {
    /// Non polarized variant
    NonPolarized(T),
    /// Spin polarized has the extra dimension
    SpinPolarized([T; 2]),
}

impl<T> SpinData<T> {
    /// Transform the data of the variants in a uniform way
    /// with the Higher Order Function
    fn map<U, F: Fn(T) -> U>(self, f: F) -> SpinData<U> {
        match self {
            SpinData::NonPolarized(data) => SpinData::NonPolarized(f(data)),
            SpinData::SpinPolarized([up, down]) => SpinData::SpinPolarized([f(up), f(down)]),
        }
    }

    /// Apply function to the variants in a uniform way
    /// with the Higher Order Function
    fn for_each<F: FnMut(&T)>(&self, mut f: F) {
        match self {
            SpinData::NonPolarized(data) => f(data),
            SpinData::SpinPolarized([up, down]) => {
                f(up);
                f(down);
            }
        }
    }
}
