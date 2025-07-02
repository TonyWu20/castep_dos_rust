use thiserror::Error;

use crate::{
    fundamental::{
        AngularMomentum, AngularMomentumConvertError, EigenvalueVec, Header, HeaderBuilder,
        HeaderBuilderError, KpointVec, NumSpins, NumSpinsConvertError, OrbitalWeight,
        OrbitalWeightVec, PDOSWeights, SpinData, SpinIndex, SpinIndexConvertError, WeightsPerEigen,
        WeightsPerKPoint, WeightsPerSpin,
    },
    helper::{HelperError, parse_record, parse_scalar, parse_vec, peek_record},
};

#[derive(Debug, Error)]
/// Errors during parsing
pub enum ParsingError {
    #[error("During parsing: {0}")]
    /// Error from converting `u32` to enum `NumSpins`
    NumSpins(#[from] NumSpinsConvertError),
    #[error("When try to convert u32 to  `SpinIndex`: {0}")]
    /// Error from converting `u32` to enum `SpinIndex`
    SpinIndex(#[from] SpinIndexConvertError),
    /// Error from converting `u32` to enum `AngularMomentum`
    #[error("When try to convert u32 to `AngularMomentum: {0}`")]
    AngularMomentum(#[from] AngularMomentumConvertError),
    #[error("During parsing: {0}")]
    /// Error from mod `helper`
    HelperError(#[from] HelperError),
    #[error("During building `Header`: {0}")]
    /// Error from `HeaderBuilder`
    HeaderBuilderError(#[from] HeaderBuilderError),
    /// Invalid format
    #[error("This is neither a valid `.pdos_weights` nor `.pdos_bin`")]
    InvalidFormat,
}
/// Handles both `.pdos_weights` and `.pdos_bin`
pub fn parse_pdos_weight_file<'a>(input: &'a mut &'a [u8]) -> Result<PDOSWeights, ParsingError> {
    // Skip the version and header output in the first two records of `.pdos_bin`
    let _version: Result<f64, HelperError> = parse_scalar::<f64, 8>(input);
    if _version.is_ok() {
        let (_, size) = peek_record(input)?;
        let _pdos_bin_header = parse_record(input, size)?;
        parse_pdos_weight(input)
    } else {
        parse_pdos_weight(input)
    }
}

/// function to parse the `.pdos_weight`
fn parse_pdos_weight(input: &mut &[u8]) -> Result<PDOSWeights, ParsingError> {
    let header = parse_header(input)?;
    let kpoints = (0..header.total_kpoints)
        .map(|_| parse_kpoint(input, &header))
        .collect::<Result<Vec<WeightsPerKPoint>, ParsingError>>()?;
    let spin_polarized = header.spin_polarized();
    let orbital_states = header.extract_orbital_states();
    let per_spin_to_per_kpt_data =
        |weights_per_spin: &WeightsPerSpin| -> EigenvalueVec<OrbitalWeightVec> {
            weights_per_spin
                .bands
                .iter()
                .map(|weights_per_eigen| {
                    weights_per_eigen
                        .weights
                        .iter()
                        .map(|weight| OrbitalWeight::new(*weight))
                        .collect::<OrbitalWeightVec>()
                })
                .collect::<EigenvalueVec<OrbitalWeightVec>>()
        };
    let orbital_weights = match header.num_spins {
        NumSpins::One => {
            SpinData::NonPolarized(
                kpoints
                    .iter() // per k-point
                    .map(
                        |weights_per_kpoint| {
                            per_spin_to_per_kpt_data(weights_per_kpoint.spins.first().unwrap()) // Only one spin
                        }, // per eigenvalue
                    )
                    .collect::<KpointVec<EigenvalueVec<OrbitalWeightVec>>>(),
            )
        }
        NumSpins::Two => {
            let (up, down) = kpoints
                .iter()
                .map(|weights_per_kpoint| {
                    (
                        per_spin_to_per_kpt_data(weights_per_kpoint.spins.first().unwrap()),
                        per_spin_to_per_kpt_data(weights_per_kpoint.spins.get(1).unwrap()),
                    )
                })
                .unzip();
            SpinData::SpinPolarized([up, down])
        }
    };
    Ok(PDOSWeights::new(
        spin_polarized,
        orbital_states,
        orbital_weights,
    ))
}

/// function to parse the header section of  `.pdos_weight`
fn parse_header(input: &mut &[u8]) -> Result<Header, ParsingError> {
    let total_kpoints = parse_scalar::<u32, 4>(input)?;
    let num_spins: NumSpins = parse_scalar::<u32, 4>(input)?.try_into()?;
    let num_orbitals = parse_scalar::<u32, 4>(input)?;
    let max_bands = parse_scalar::<u32, 4>(input)?;

    let orbital_species = parse_vec::<u32, 4>(input, num_orbitals as usize)?;
    let orbital_ion = parse_vec::<u32, 4>(input, num_orbitals as usize)?;
    let orbital_am = parse_vec::<u32, 4>(input, num_orbitals as usize)?
        .into_iter()
        .map(|l| AngularMomentum::try_from(l).map_err(ParsingError::AngularMomentum))
        .collect::<Result<Vec<AngularMomentum>, ParsingError>>()?;

    HeaderBuilder::default()
        .total_kpoints(total_kpoints)
        .num_spins(num_spins)
        .num_orbitals(num_orbitals)
        .max_bands(max_bands)
        .orbital_species(orbital_species)
        .orbital_ion(orbital_ion)
        .orbital_am(orbital_am)
        .build()
        .map_err(ParsingError::HeaderBuilderError)
}

/// Parse data for each k-point
fn parse_kpoint(input: &mut &[u8], header: &Header) -> Result<WeightsPerKPoint, ParsingError> {
    let kp_data = parse_record(input, 28)?;
    let index = u32::from_be_bytes(
        kp_data[0..4]
            .try_into()
            .map_err(HelperError::BytesIntoArray)?,
    );
    let kx = f64::from_be_bytes(
        kp_data[4..12]
            .try_into()
            .map_err(HelperError::BytesIntoArray)?,
    );
    let ky = f64::from_be_bytes(
        kp_data[12..20]
            .try_into()
            .map_err(HelperError::BytesIntoArray)?,
    );
    let kz = f64::from_be_bytes(
        kp_data[20..28]
            .try_into()
            .map_err(HelperError::BytesIntoArray)?,
    );
    let kpoint = [kx, ky, kz];
    let spins = (0..header.num_spins.spin_count())
        .map(|_| parse_weight_per_spin(input, header))
        .collect::<Result<Vec<WeightsPerSpin>, ParsingError>>()?;
    Ok(WeightsPerKPoint::new(index, kpoint, spins))
}

/// Parse weight for each spin inside the record of a k-point
fn parse_weight_per_spin(
    input: &mut &[u8],
    header: &Header,
) -> Result<WeightsPerSpin, ParsingError> {
    let index = parse_scalar::<u32, 4>(input)?;
    let spin_index = SpinIndex::try_from(index)?;
    let nbands_occ = parse_scalar::<u32, 4>(input)?;
    // Parse band weights
    let bands = (0..nbands_occ)
        .map(|_| {
            let weights = parse_vec::<f64, 8>(input, header.num_orbitals as usize)?;
            Ok(WeightsPerEigen::new(weights))
        })
        .collect::<Result<Vec<WeightsPerEigen>, ParsingError>>()?;
    Ok(WeightsPerSpin::new(spin_index, nbands_occ, bands))
}
#[cfg(test)]
mod test {
    use std::fs::read;

    use crate::pdos_weights_parser::parse_pdos_weight_file;

    use super::{ParsingError, parse_header};

    const TEST_PDOS_WEIGHT: &str = "/home/tony/Downloads/cosxmos2_DOS/cosxmos2_DOS.pdos_weights";
    const TEST_PDOS_WEIGHT_NO_SPIN: &str = "/home/tony/Downloads/Si/Si_DOS.pdos_weights";
    const TEST_PDOS_BIN: &str = "/home/tony/Downloads/Si/Si_DOS.pdos_bin";
    #[test]
    fn test_header() {
        let pdos_file = read(TEST_PDOS_WEIGHT).unwrap();
        let header = parse_header(&mut pdos_file.as_ref()).unwrap();
        dbg!(header.total_kpoints);
        dbg!(header.num_spins);
        dbg!(header.num_orbitals);
        dbg!(header.max_bands);
        dbg!(header.orbital_am.len());
    }
    #[test]
    fn test_spin_pdos_weight() -> Result<(), ParsingError> {
        let pdos_file = read(TEST_PDOS_WEIGHT).unwrap();
        let parsed_pdos = parse_pdos_weight_file(&mut &pdos_file[..])?;
        dbg!(parsed_pdos.orbital_states);
        Ok(())
    }
    #[test]
    fn test_pdos_weight() -> Result<(), ParsingError> {
        let pdos_file = read(TEST_PDOS_WEIGHT_NO_SPIN).unwrap();
        let parsed_pdos = parse_pdos_weight_file(&mut &pdos_file[..])?;
        dbg!(parsed_pdos.orbital_states);
        Ok(())
    }
    #[test]
    fn test_pdos_bin() -> Result<(), ParsingError> {
        let pdos_bin = read(TEST_PDOS_BIN).unwrap();
        let parsed_dos = parse_pdos_weight_file(&mut &pdos_bin[..]);
        assert!(parsed_dos.is_ok());
        Ok(())
    }
}
