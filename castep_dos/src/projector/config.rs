use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("The min and max value of energy grid is reversed.")]
    ReverseMinMax,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Newtype to represent the symbol of species
pub struct SpeciesSymbol(pub String);

impl AsRef<str> for SpeciesSymbol {
    #[inline]
    fn as_ref(&self) -> &str {
        <String as AsRef<str>>::as_ref(&self.0)
    }
}

impl std::ops::Deref for SpeciesSymbol {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
/// Config of projector generation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectorConfig {
    /// Name of this projector config: optional
    pub name: Option<String>,
    /// Label of this projector config: optional
    pub label: Option<String>,
    /// Selections of species and atoms.
    /// If none is provided it is equivalent to calculating
    /// total density of states for the whole system
    pub selections: Option<Vec<Selection>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Bind the ion id and species together
pub struct Selection {
    /// Species symbol
    pub species: SpeciesSymbol,
    /// Ions id of this species (Start at 1!!)
    /// If none is provided, default to all atoms of this species.
    pub atoms: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(default)]
pub struct EnergyGridConfig {
    pub min: f64,
    pub max: f64,
    pub points_per_ev: usize,
    pub smearing: f64,
}

impl EnergyGridConfig {
    /// Constructor
    pub fn new(min: f64, max: f64, points_per_ev: usize, smearing: f64) -> Self {
        Self {
            min,
            max,
            points_per_ev,
            smearing,
        }
    }
    /// Generate the grid for PDOS computation
    pub fn get_energy_grid(&self) -> Result<Vec<f64>, ConfigError> {
        let total_ev = self.max - self.min;
        if total_ev < 0.0 {
            return Err(ConfigError::ReverseMinMax);
        }
        let total_points = (total_ev * self.points_per_ev as f64).ceil() as usize + 1;
        Ok((0..total_points)
            .map(|i| {
                let fraction = i as f64 / (total_points - 1) as f64;
                self.min + fraction * total_ev
            })
            .collect())
    }
}

impl Default for EnergyGridConfig {
    fn default() -> Self {
        Self {
            min: -20.0,
            max: 20.0,
            points_per_ev: 100,
            smearing: 0.1,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::PDOSConfig;

    const CONFIG: &str = r#"[[projector]]
[[projector.selections]]
species = "Mo"
atoms = [1,2,3,4,5,6,7,8]
[[projector.selections]]
species = "S"
"#;
    #[test]
    fn test_config() {
        let config = toml::from_str::<PDOSConfig>(CONFIG).unwrap();
        dbg!(&config);
        println!("{}", toml::to_string(&config).unwrap())
    }
}
