use std::{
    fs::{read, read_to_string, write},
    io,
    path::Path,
    time::Instant,
};

use castep_dos::{
    config::{ConfigError, EnergyGridConfig, ProgramConfig},
    plot::plot,
};
use castep_dos_core::{
    bands::{BandsParser, BandsParsingError},
    fundamental::{BandStructure, PDOSWeights, SpinData},
    pdos_compute::{PDOSResult, calculate_pdos},
    pdos_weights_parser::{ParsingError, parse_pdos_weight_file},
};
use clap::{Parser, Subcommand};
use plotters::prelude::DrawingAreaErrorKind;
use thiserror::Error;

#[derive(Debug, clap::Parser)]
pub struct ProgArgs {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run { seed: String },
    Example,
}

#[derive(Debug, Error)]
pub enum ExeError {
    #[error("IOError: {0}")]
    IOError(#[from] io::Error),
    #[error("Error in config: {0}")]
    ConfigError(#[from] castep_dos::config::ConfigError),
    #[error("Error when parsing `.pdos_weights` or `.pdos_bin`: {0}")]
    PDOSWeightsParsing(#[from] ParsingError),
    #[error("Error when parsing `.bands`: {0}")]
    BandsParsing(#[from] BandsParsingError),
    #[error("Error when plotting pdos result: {0}")]
    Drawing(#[from] DrawingAreaErrorKind<std::io::Error>),
}
/// Execution
fn main() -> Result<(), ExeError> {
    let args = ProgArgs::parse();
    match args.commands {
        Commands::Example => write(
            Path::new("example.toml"),
            toml::to_string_pretty(&ProgramConfig::example()).map_err(ConfigError::Serialize)?,
        )
        .map_err(ExeError::IOError),
        Commands::Run { seed } => run(&seed),
    }
}

fn run(seed: &str) -> Result<(), ExeError> {
    let seed_stem = Path::new(&seed);
    let (prog_config, pdos_weights, bands) = load_pdos_calc_files(seed_stem)?;
    let (e_min, e_max) = determine_energy_range(&bands, &prog_config.energy_grid);
    let energy_grid = generate_grid(e_min, e_max, prog_config.energy_grid.points_per_ev);
    let species_mapping = prog_config.pdos_config.species_mapping();
    let before = Instant::now();
    prog_config
        .pdos_config
        .projectors
        .iter()
        .enumerate()
        .map(|(i, proj_conf)| {
            (
                proj_conf
                    .name
                    .clone()
                    .unwrap_or(format!("setting_{}", i + 1)),
                proj_conf.project_pdos_from_config(&species_mapping, &pdos_weights),
            )
        })
        .try_for_each(|(proj_name, projected_weights)| {
            let result = calculate_pdos(
                &bands,
                &projected_weights,
                &energy_grid,
                prog_config.energy_grid.smearing,
            );
            result_output(result, seed, &proj_name, &prog_config, &energy_grid)
        })?;
    println!(
        "PDOS calculations of {} finished in {:.2?}",
        seed,
        before.elapsed()
    );
    Ok(())
}

fn load_pdos_calc_files(
    seed_stem: &Path,
) -> Result<(ProgramConfig, PDOSWeights, BandStructure), ExeError> {
    let bands_file = seed_stem.with_extension("bands");
    let pdos_weights_file = seed_stem.with_extension("pdos_bin");
    let config_file = seed_stem.with_extension("toml");

    let prog_config = read_to_string(config_file)
        .map_err(ExeError::IOError)
        .and_then(|content| {
            toml::from_str::<ProgramConfig>(&content)
                .map_err(|e| ExeError::ConfigError(ConfigError::Deserialize(e)))
        })?;
    let pdos_weights = read(pdos_weights_file)
        .or_else(|_| read(seed_stem.with_extension("pdos_weights")).map_err(ExeError::IOError))
        .and_then(|content| {
            parse_pdos_weight_file(&mut &content[..]).map_err(ExeError::PDOSWeightsParsing)
        })?;

    let bands = read_to_string(bands_file)
        .map_err(ExeError::IOError)
        .and_then(|content| {
            let bands_parser = BandsParser::new(&content);
            bands_parser
                .parse_bands_file()
                .map_err(ExeError::BandsParsing)
        })
        .map(|bands_file| bands_file.to_band_structure())?;
    Ok((prog_config, pdos_weights, bands))
}

fn generate_grid(e_min: f64, e_max: f64, points_per_ev: usize) -> Vec<f64> {
    let total_points = ((e_max - e_min) * points_per_ev as f64) as usize + 1;
    (0..total_points)
        .map(|i| {
            let fraction = i as f64 / (total_points - 1) as f64;
            e_min + fraction * (e_max - e_min)
        })
        .collect()
}

/// The user settings is prioritized.
/// If the energy range is not set in `EnergyGridConfig` from the `toml`,
/// use the results computed from `.bands`.
fn determine_energy_range(
    bands: &BandStructure,
    energy_grid_config: &EnergyGridConfig,
) -> (f64, f64) {
    let (e_min, e_max) = (energy_grid_config.min, energy_grid_config.max);
    let (band_min, band_max) = bands.energy_range(energy_grid_config.smearing);
    match (e_min, e_max) {
        (None, None) => (band_min, band_max),
        (None, Some(max)) => (band_min, max),
        (Some(min), None) => (min, band_max),
        (Some(min), Some(max)) => (min, max),
    }
}

fn result_output(
    result: SpinData<PDOSResult>,
    seed: &str,
    proj_name: &str,
    prog_config: &ProgramConfig,
    energy_grid: &[f64],
) -> Result<(), ExeError> {
    let backup_name = format!("{}_pdos_{}_config_backup", seed, proj_name);
    let backup_stem = Path::new(&backup_name);
    match result {
        castep_dos_core::fundamental::SpinData::NonPolarized(no_spin) => {
            let result_name = format!("{}_pdos_{}", seed, proj_name);
            let result_stem = Path::new(&result_name);
            let csv_path = &result_stem.with_extension("csv");
            let toml_path = &backup_stem.with_extension("toml");
            write(csv_path, no_spin.csv_output(energy_grid))?;
            write(
                toml_path,
                toml::to_string_pretty(prog_config)
                    .map_err(ConfigError::Serialize)
                    .map_err(ExeError::ConfigError)?,
            )?;
            plot(energy_grid, &no_spin, &result_name)?;
            Ok::<(), ExeError>(())
        }
        castep_dos_core::fundamental::SpinData::SpinPolarized([up, down]) => {
            let up_name = format!("{}_pdos_{}_spin_up", seed, proj_name);
            let up_stem = Path::new(&up_name);
            let down_name = format!("{}_pdos_{}_spin_down", seed, proj_name);
            let down_stem = Path::new(&down_name);
            write(up_stem.with_extension("csv"), up.csv_output(energy_grid))?;
            write(
                down_stem.with_extension("csv"),
                down.csv_output(energy_grid),
            )?;
            write(
                backup_stem.with_extension("toml"),
                toml::to_string_pretty(prog_config)
                    .map_err(ConfigError::Serialize)
                    .map_err(ExeError::ConfigError)?,
            )?;
            plot(energy_grid, &up, &up_name)?;
            plot(energy_grid, &down, &down_name)?;
            Ok::<(), ExeError>(())
        }
    }
}
