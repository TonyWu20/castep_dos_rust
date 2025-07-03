use serde::{Deserialize, Serialize};

use castep_dos_core::projectors::PDOSConfig;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Config file for projector specifications
pub struct ProgramConfig {
    #[serde(rename = "pdos")]
    pub pdos_config: PDOSConfig,
    #[serde(default)]
    pub energy_grid: EnergyGridConfig,
}

impl ProgramConfig {
    pub fn example() -> Self {
        Self {
            pdos_config: PDOSConfig::example(),
            energy_grid: EnergyGridConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(default)]
pub struct EnergyGridConfig {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub points_per_ev: usize,
    pub smearing: f64,
}

#[derive(Debug, Error)]
/// Error in reading config file
pub enum ConfigError {
    #[error("During deserialization: {0}")]
    Deserialize(#[from] toml::de::Error),
    #[error("During serialization: {0}")]
    Serialize(#[from] toml::ser::Error),
    /// Min and max of energy grid is reversed
    #[error("The min and max value of energy grid is reversed.")]
    ReverseMinMax,
}

impl EnergyGridConfig {
    pub fn new(min: Option<f64>, max: Option<f64>, points_per_ev: usize, smearing: f64) -> Self {
        Self {
            min,
            max,
            points_per_ev,
            smearing,
        }
    }

    /// Constructor
    /// Generate the grid for PDOS computation
    pub fn get_energy_grid(&self) -> Result<Vec<f64>, ConfigError> {
        let (max, min) = (self.max.unwrap_or(20.0), self.min.unwrap_or(-20.0));
        let total_ev = max - min;
        if total_ev < 0.0 {
            return Err(ConfigError::ReverseMinMax);
        }
        let total_points = (total_ev * self.points_per_ev as f64).ceil() as usize + 1;
        Ok((0..total_points)
            .map(|i| {
                let fraction = i as f64 / (total_points - 1) as f64;
                min + fraction * total_ev
            })
            .collect())
    }
}

impl Default for EnergyGridConfig {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            points_per_ev: 100,
            smearing: 0.1,
        }
    }
}

#[cfg(test)]
mod test {

    use super::ProgramConfig;

    const MOS2_CONFIG: &str = r#"
[pdos]
mapping = [{species="Mo", rank=2}, {species="S", rank=1}]

[[pdos.projector]]
name="Mo 2 and S 1"
[[pdos.projector.selections]]
species = "Mo"
atoms = [2]
[[pdos.projector.selections]]
species = "S"
atoms= [1]
[energy_grid]
points_per_ev=100
smearing=0.2
"#;
    const PT_CONFIG: &str = r#"[[projector]]
[[projector.selections]]
species = "Pt"
atoms = [1]
[[projector]]
[[projector.selections]]
species = "O"
atoms= [1]
"#;
    const PDOS_FILE: &str = "/home/tony/Downloads/cosxmos2_DOS/cosxmos2_DOS.pdos_weights";
    const PT_PDOS: &str = "/home/tony/Downloads/Pt_311/Pt_311_12lyr_v20_CO.pdos_weights";
    const CAO_CONFIG: &str = r#"[[projector]]
[[projector.selections]]
species = "Ca"
atoms = [1]
[[projector]]
[[projector.selections]]
species = "O"
atoms= [1]
"#;
    const CAO_PDOS_FILE: &str = "/home/tony/Downloads/CaO_CASTEP_Energy/CaO.pdos_weights";
    #[test]
    fn test_run_config() {
        let config = toml::from_str::<ProgramConfig>(MOS2_CONFIG).unwrap();
        dbg!(config);
    }
}
