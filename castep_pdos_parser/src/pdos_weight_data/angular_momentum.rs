use thiserror::Error;

use super::AngularMomentum;

#[derive(Debug, Error)]
#[error("Invalid value ({0}) for an angular momentum")]
/// Error type for `TryFrom`
pub struct AngularMomentumConvertError(u32);

impl TryFrom<u32> for AngularMomentum {
    type Error = AngularMomentumConvertError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AngularMomentum::S),
            1 => Ok(AngularMomentum::P),
            2 => Ok(AngularMomentum::D),
            3 => Ok(AngularMomentum::F),
            other => Err(AngularMomentumConvertError(other)),
        }
    }
}

impl From<AngularMomentum> for u32 {
    fn from(value: AngularMomentum) -> Self {
        match value {
            AngularMomentum::S => 0,
            AngularMomentum::P => 1,
            AngularMomentum::D => 2,
            AngularMomentum::F => 3,
        }
    }
}
