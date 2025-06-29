use super::NumSpins;
use thiserror::Error;

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
