use std::collections::HashMap;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::fundamental::{
    AngularChannels, AngularMomentum, EigenvalueVec, KpointVec, OrbitalState, OrbitalWeightVec,
    PDOSWeights, SpinData,
};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Config file for projector specifications
pub struct PDOSConfig {
    /// Species mapping defined following the seed.cell
    #[serde(rename = "mapping")]
    pub species_mapping: Vec<Mapping>,
    #[serde(rename = "projector")]
    /// Groups of projector config
    pub projectors: Vec<ProjectorConfig>,
    // #[serde(default)]
    // ///
    // pub energy_grid: EnergyGridConfig,
}

impl PDOSConfig {
    /// Generate the species_mapping `HashMap`
    pub fn species_mapping(&self) -> HashMap<&str, u32> {
        self.species_mapping
            .iter()
            .map(|mapping| (mapping.species.as_ref(), mapping.rank))
            .collect()
    }

    /// Generate example
    pub fn example() -> Self {
        Self {
            species_mapping: vec![Mapping {
                species: SpeciesSymbol("C".to_string()),
                rank: 1,
            }],
            projectors: vec![ProjectorConfig {
                name: Some("Example".to_string()),
                label: None,
                selections: Some(vec![Selection {
                    species: SpeciesSymbol("C".to_string()),
                    atoms: Some(vec![1, 2]),
                }]),
            }],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Mapping {
    /// Species symbol
    species: SpeciesSymbol,
    /// Species id in seed.cell. It is ranked by the species's atomic number.
    rank: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
/// Newtype to represent the symbol of species
pub struct SpeciesSymbol(String);

impl Deref for SpeciesSymbol {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for SpeciesSymbol {
    fn as_ref(&self) -> &str {
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
    species: SpeciesSymbol,
    /// Ions id of this species (Start at 1!!)
    /// If none is provided, default to all atoms of this species.
    atoms: Option<Vec<u32>>,
}

impl Selection {
    /// Access method
    pub fn species(&self) -> &SpeciesSymbol {
        &self.species
    }

    /// Access method
    pub fn atoms(&self) -> Option<&Vec<u32>> {
        self.atoms.as_ref()
    }
}

// #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
// #[serde(default)]
// pub struct EnergyGridConfig {
//     pub min: f64,
//     pub max: f64,
//     pub points_per_ev: usize,
//     pub smearing: f64,
// }

// impl EnergyGridConfig {
//     /// Constructor
//     pub fn new(min: f64, max: f64, points_per_ev: usize, smearing: f64) -> Self {
//         Self {
//             min,
//             max,
//             points_per_ev,
//             smearing,
//         }
//     }
//     /// Generate the grid for PDOS computation
//     pub fn get_energy_grid(&self) -> Result<Vec<f64>, ConfigError> {
//         let total_ev = self.max - self.min;
//         if total_ev < 0.0 {
//             return Err(ConfigError::ReverseMinMax);
//         }
//         let total_points = (total_ev * self.points_per_ev as f64).ceil() as usize + 1;
//         Ok((0..total_points)
//             .map(|i| {
//                 let fraction = i as f64 / (total_points - 1) as f64;
//                 self.min + fraction * total_ev
//             })
//             .collect())
//     }
// }

// impl Default for EnergyGridConfig {
//     fn default() -> Self {
//         Self {
//             min: -20.0,
//             max: 20.0,
//             points_per_ev: 100,
//             smearing: 0.1,
//         }
//     }
// }
impl ProjectorConfig {
    /// Project PDOS weights for a single projector configuration
    pub fn project_pdos_from_config(
        &self,
        species_mapping: &HashMap<&str, u32>,
        pdos_weights: &PDOSWeights,
    ) -> SpinData<KpointVec<EigenvalueVec<AngularChannels>>> {
        // obtain selected orbital ids
        let orbital_states = &pdos_weights.orbital_states;

        let selected_orbital_ids = self
            .selections
            .as_ref()
            .map(|selections| extract_selections(selections, species_mapping, orbital_states))
            .unwrap_or((0..pdos_weights.orbital_states.len()).collect());
        let angular_indices = [
            AngularMomentum::S,
            AngularMomentum::P,
            AngularMomentum::D,
            AngularMomentum::F,
        ]
        .map(|am| {
            selected_orbital_ids
                .iter()
                .filter(|&&idx| orbital_states[idx].angular_momentum == am)
                .copied()
                .collect::<Vec<usize>>()
        });
        pdos_weights
            .orbital_weights
            .map_on_data_of_eigenvalue(|weights| AngularChannels {
                s: sum_weights_per_eigenvalue(&angular_indices[0], weights),
                p: sum_weights_per_eigenvalue(&angular_indices[1], weights),
                d: sum_weights_per_eigenvalue(&angular_indices[2], weights),
                f: sum_weights_per_eigenvalue(&angular_indices[3], weights),
            })
    }
}

/// From selections, compare the species symbol and ion index,
/// obtain the usize index in each orbital weight array
fn extract_selections(
    selections: &[Selection],
    species_mapping: &HashMap<&str, u32>,
    orbital_states: &[OrbitalState],
) -> Vec<usize> {
    // For every selection
    selections.iter().flat_map(|sel| {
        let species_id = species_mapping.get(sel.species().as_str()).expect("The species symbol in `selection` does not match with any record in `mapping. Please double check the config toml file.`");
        let atoms = sel.atoms();
        match atoms {
            // If we have selected atoms
            Some(atoms) =>{
                atoms.iter().flat_map(|atom_id| {
                    orbital_states.par_iter().enumerate().filter_map(|(i,state)| {
                        if state.species_id == *species_id && state.ion_id == *atom_id {
                            Some(i)
                        }else{
                            None
                        }
                    }).collect::<Vec<usize>>()
                }).collect::<Vec<usize>>()
            }
            None => {
                orbital_states.par_iter().enumerate().filter_map(|(i, state)| {
                    if state.species_id == *species_id {
                        Some(i)
                    } else {
                        None
                    }
                }).collect()
            }
        }
    }).collect()
}

/// Gather weights at each eigenvalue's orbital weight array
fn sum_weights_per_eigenvalue(indices: &[usize], weights: &OrbitalWeightVec) -> f64 {
    indices
        .par_iter()
        .map(|&idx| weights[idx].value())
        // .filter(|&w| w > 0.0)
        .sum()
}

#[cfg(test)]
mod test {
    use std::fs::read;

    use crate::pdos_weights_parser::parse_pdos_weight_file;

    use super::PDOSConfig;

    const CONFIG: &str = r#"
mapping=[{species="Mo", rank=2}, {species="S", rank=1}]
[[projector]]
[[projector.selections]]
species = "Mo"
atoms = [1,]
[[projector.selections]]
species = "S"
atoms = [1,]
"#;
    #[test]
    fn test_config() {
        let pdos_file =
            read("/home/tony/Downloads/cosxmos2_DOS/cosxmos2_DOS.pdos_weights").unwrap();
        let pdos_weights = parse_pdos_weight_file(&mut &pdos_file[..]).unwrap();
        let config = toml::from_str::<PDOSConfig>(CONFIG).unwrap();
        dbg!(&config);
        let species_mapping = config.species_mapping();
        println!("{}", toml::to_string(&config).unwrap());
        config.projectors.iter().for_each(|projector| {
            dbg!(projector.project_pdos_from_config(&species_mapping, &pdos_weights));
        });
    }
}
