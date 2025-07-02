use std::{
    iter::Sum,
    ops::{Add, AddAssign},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The angular momentum expression
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AngularMomentum {
    /// l = 0
    S,
    /// l = 1
    P,
    /// l = 2
    D,
    /// l = 3
    F,
}

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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
/// Struct to collect orbital weights for each angular momentum
/// channels
pub struct AngularChannels {
    /// At s-channel
    pub s: f64,
    /// At p-channel
    pub p: f64,
    /// At d-channel
    pub d: f64,
    /// At f-channel
    pub f: f64,
}

impl AngularChannels {
    /// Constructor
    pub fn new(s: f64, p: f64, d: f64, f: f64) -> Self {
        Self { s, p, d, f }
    }
    /// Initialize as four zeroes
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

impl Add for AngularChannels {
    type Output = AngularChannels;

    fn add(self, rhs: Self) -> Self::Output {
        AngularChannels {
            s: self.s + rhs.s,
            p: self.p + rhs.p,
            d: self.d + rhs.d,
            f: self.f + rhs.f,
        }
    }
}

impl Add<&AngularChannels> for AngularChannels {
    type Output = AngularChannels;

    fn add(self, rhs: &AngularChannels) -> Self::Output {
        self + *rhs
    }
}

impl AddAssign for AngularChannels {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sum<AngularChannels> for AngularChannels {
    fn sum<I: Iterator<Item = AngularChannels>>(iter: I) -> Self {
        iter.fold(AngularChannels::zero(), |acc, e| acc + e)
    }
}

impl<'a> Sum<&'a AngularChannels> for AngularChannels {
    fn sum<I: Iterator<Item = &'a AngularChannels>>(iter: I) -> Self {
        iter.fold(AngularChannels::zero(), |acc, e| acc + e)
    }
}

#[cfg(test)]
mod test {
    use super::AngularChannels;

    #[test]
    fn angular_channel() {
        let angular_channels = vec![
            AngularChannels::new(0.1, 0.0, 0.3, 0.0),
            AngularChannels::new(0.0, 0.2, 0.2, 0.4),
        ];
        let sum_on_refs: AngularChannels = angular_channels.iter().sum();
        dbg!(sum_on_refs);
        let sum: AngularChannels = angular_channels.into_iter().sum();
        debug_assert_eq!(sum, AngularChannels::new(0.1, 0.2, 0.5, 0.4))
    }
}
