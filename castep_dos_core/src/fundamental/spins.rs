use thiserror::Error;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// The number of spins only has two variant:
pub enum SpinIndex {
    /// - One: default
    One = 1,
    /// - Two: Only when `spin_polarised : true`
    Two = 2,
}

#[derive(Debug, Error)]
#[error("Try to convert to `SpinIndex` from value out of 1 and 2.")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Bool enum to mark if the data is spin-polarized
pub enum SpinPolarized {
    /// True
    True,
    /// False
    False,
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
