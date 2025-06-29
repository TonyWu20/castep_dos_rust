use thiserror::Error;

use super::SpinIndex;

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
