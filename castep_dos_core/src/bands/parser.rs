use thiserror::Error;
use winnow::{
    ModalResult, Parser,
    ascii::{digit1, float, line_ending, space1},
    combinator::{
        alt, delimited, dispatch, empty, fail, preceded, repeat, separated_pair, terminated,
    },
    error::{ContextError, StrContext},
    token::one_of,
};

use crate::bands::{
    BandsFile, BandsFileBuilder, BandsFileBuilderError, Eigenvalues, ElectronCount, FermiEnergy,
    KPoint, SpinPolarized,
};

/// Band data per k-point in `.bands`
#[derive(Debug, Clone, PartialEq)]
pub struct BandPerKPoint {
    /// k-point of this band
    pub kpoint: KPoint,
    /// Eigenvalues of this k-point
    pub eigen_band: EigenvaluesPerBand,
}

impl BandPerKPoint {
    /// Constructor
    pub fn new(kpoint: KPoint, eigen_band: EigenvaluesPerBand) -> Self {
        Self { kpoint, eigen_band }
    }
}

/// Type-safe Eigenvalues per band repr
#[derive(Debug, Clone, PartialEq)]
pub enum EigenvaluesPerBand {
    /// Non-spin-polarized: 1 entry
    NonPolarized(Vec<f64>),
    /// Spin-polarized: 2 entries
    SpinPolarized(Vec<f64>, Vec<f64>),
}

impl EigenvaluesPerBand {
    /// Consume `self` as `NonPolarized` variant
    pub fn as_non_polarized(&self) -> Option<&Vec<f64>> {
        if let Self::NonPolarized(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Consume `self` as `SpinPolarized` variant
    pub fn as_spin_polarized(&self) -> Option<(&Vec<f64>, &Vec<f64>)> {
        if let Self::SpinPolarized(v1, v2) = self {
            Some((v1, v2))
        } else {
            None
        }
    }
}

#[derive(Debug, Error)]
/// Possible errors in parsing `.bands`
pub enum BandsParsingError {
    #[error("During parsing with `winnow`:{0}")]
    /// Error from `winnow`
    ContextError(winnow::error::ErrMode<ContextError>),
    #[error("Builder error: {0}")]
    /// Error from builder
    BuilderError(#[from] BandsFileBuilderError),
}

/// Parser of `.bands`, holds the slice of file content.
#[derive(Debug, Clone)]
pub struct BandsParser<'a> {
    input: &'a str,
    /// Spin polarized settings
    pub spin_polarized: Option<SpinPolarized>,
    /// Number of electrons in system
    pub electron_count: Option<ElectronCount>,
    /// Fermi energy/energies in Hartree
    pub fermi_energy: Option<FermiEnergy>,
    /// Lattice vectors in Angstroms (row-major)
    pub lattice_vectors: Option<[[f64; 3]; 3]>,
    /// Eigen values for each k-point
    pub eigens_bands: Option<Vec<BandPerKPoint>>,
}

impl<'a> BandsParser<'a> {
    /// Constructor
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            spin_polarized: None,
            electron_count: None,
            fermi_energy: None,
            lattice_vectors: None,
            eigens_bands: None,
        }
    }
    /// Parse the first line for k-point count
    fn parse_kpoint_count(&mut self) -> ModalResult<&mut Self> {
        delimited(("Number of k-points", space1), digit1, line_ending)
            .context(StrContext::Label("Number of k-points"))
            .parse_next(self.input_mut())?;
        Ok(self)
    }

    fn parse_spin_config(&mut self) -> ModalResult<&mut Self> {
        let spin_polarized = delimited(
            ("Number of spin components", space1),
            dispatch! {one_of(('1', '2'));
                '1' => empty.map(|_|SpinPolarized::False) ,
                '2' => empty.map(|_|SpinPolarized::True) ,
                _ => fail::<_,SpinPolarized,_>.context(winnow::error::StrContext::Expected(
                winnow::error::StrContextValue::Description("Spin value 1 or 2"),
            ))

            },
            line_ending,
        )
        .parse_next(self.input_mut())?;
        self.spin_polarized = Some(spin_polarized);
        Ok(self)
    }

    fn parse_electron_count(&mut self) -> ModalResult<&mut Self> {
        let electron_count = match self.spin_polarized {
            Some(SpinPolarized::True) => delimited(
                ("Number of electrons", space1),
                (terminated(float, space1), terminated(float, space1)),
                line_ending,
            )
            .map(|(count_1, count_2)| ElectronCount::Polarized(count_1, count_2))
            .parse_next(self.input_mut()),
            Some(SpinPolarized::False) => delimited(
                ("Number of electrons", space1),
                terminated(float, space1)
                    .context(StrContext::Label("Non spin polarized, number of electrons")),
                line_ending,
            )
            .map(ElectronCount::NonPolarized)
            .parse_next(self.input_mut()),
            _ => fail::<_, ElectronCount, _>
                .context(StrContext::Expected(
                    winnow::error::StrContextValue::StringLiteral(
                        "Spin components have not been parsed",
                    ),
                ))
                .parse_next(self.input_mut()),
        }?;
        self.electron_count = Some(electron_count);
        Ok(self)
    }

    fn parse_eigenvalue_count(&mut self) -> ModalResult<&mut Self> {
        match self.spin_polarized {
            Some(SpinPolarized::True) => delimited(
                ("Number of eigenvalues", space1),
                separated_pair(digit1, space1, digit1),
                line_ending,
            )
            .map(|_| ())
            .parse_next(self.input_mut()),
            Some(SpinPolarized::False) => {
                delimited(("Number of eigenvalues", space1), digit1, line_ending)
                    .map(|_| ())
                    .context(StrContext::Label("Skips number of eigenvalues"))
                    .parse_next(self.input_mut())
            }
            _ => fail::<_, (), _>
                .context(StrContext::Expected(
                    winnow::error::StrContextValue::StringLiteral(
                        "Spin components have not been parsed",
                    ),
                ))
                .parse_next(self.input_mut()),
        }?;
        Ok(self)
    }

    fn parse_fermi_energy(&mut self) -> ModalResult<&mut Self> {
        let spin_heading = |input: &mut &'a str| -> ModalResult<(&'a str, &'a str)> {
            ("Fermi energies (in atomic units)", space1).parse_next(input)
        };
        let heading = |input: &mut &'a str| -> ModalResult<(&'a str, &'a str)> {
            ("Fermi energy (in atomic units)", space1).parse_next(input)
        };
        let fermi_energy = match self.spin_polarized {
            Some(SpinPolarized::True) => delimited(
                spin_heading,
                separated_pair(float, space1, float),
                line_ending,
            )
            .map(|(fermi_1, fermi_2)| FermiEnergy::Polarized(fermi_1, fermi_2))
            .parse_next(self.input_mut()),
            Some(SpinPolarized::False) => delimited(heading, float, line_ending)
                .map(FermiEnergy::NonPolarized)
                .parse_next(self.input_mut()),
            _ => fail::<_, FermiEnergy, _>
                .context(StrContext::Expected(
                    winnow::error::StrContextValue::StringLiteral(
                        "Spin components have not been parsed",
                    ),
                ))
                .parse_next(self.input_mut()),
        }?;
        self.fermi_energy = Some(fermi_energy);
        Ok(self)
    }
    fn parse_unit_cell_vectors(&mut self) -> ModalResult<&mut Self> {
        let vector_line = |input: &mut &'a str| -> ModalResult<[f64; 3]> {
            terminated(repeat(3, preceded(space1, float::<_, f64, _>)), line_ending)
                .verify_map(|v: Vec<f64>| v.try_into().ok())
                .parse_next(input)
        };
        self.lattice_vectors = Some(
            preceded(("Unit cell vectors", line_ending), repeat(3, vector_line))
                .verify_map(|vectors: Vec<[f64; 3]>| vectors.try_into().ok())
                .parse_next(self.input_mut())?,
        );
        Ok(self)
    }

    fn parse_band_per_kpoint(&mut self) -> ModalResult<&mut Self> {
        let kpoint = |input: &mut &str| -> ModalResult<KPoint> {
            delimited(
                ("K-point", space1),
                (
                    digit1.parse_to::<u32>(), // index
                    // kx,
                    preceded(space1, float),
                    // ky,
                    preceded(space1, float),
                    // kz
                    preceded(space1, float),
                    // weight
                    preceded(space1, float),
                ),
                line_ending,
            )
            .map(|(id, kx, ky, kz, weight)| KPoint::new(id as usize, [kx, ky, kz], weight))
            .context(StrContext::Label("Kpoint"))
            .parse_next(input)
        };
        let eigenvalues = |input: &mut &str| -> ModalResult<Vec<f64>> {
            repeat(
                1..,
                delimited(
                    space1.context(StrContext::Label("leading spaces")),
                    float::<_, f64, _>.context(StrContext::Label("eigenvalue")),
                    line_ending,
                ),
            )
            .context(StrContext::Label("eigenvalue per line"))
            .parse_next(input)
        };
        let spin_polarized = self.spin_polarized.unwrap();
        let eigens_bands: Vec<BandPerKPoint> = repeat(
            1..,
            (
                kpoint,
                repeat(
                    1..=2,
                    preceded(
                        terminated(alt(("Spin component 1", "Spin component 2")), line_ending),
                        eigenvalues,
                    )
                    .context(StrContext::Label("Eigenvalues")),
                ),
            )
                .context(StrContext::Label("Eigenvalues per kpoint"))
                .map(|(kpoint, eigens): (KPoint, Vec<Vec<f64>>)| {
                    let eigen_band = match spin_polarized {
                        SpinPolarized::True => EigenvaluesPerBand::SpinPolarized(
                            eigens[0].to_vec(),
                            eigens[1].to_vec(),
                        ),
                        SpinPolarized::False => {
                            EigenvaluesPerBand::NonPolarized(eigens[0].to_vec())
                        }
                    };
                    BandPerKPoint { kpoint, eigen_band }
                }),
        )
        .parse_next(self.input_mut())?;
        self.eigens_bands = Some(eigens_bands);
        Ok(self)
    }

    /// mut getter of `input`, for usage in parsing
    pub fn input_mut(&mut self) -> &mut &'a str {
        &mut self.input
    }

    /// Main usage
    pub fn parse_bands_file(mut self) -> Result<BandsFile, BandsParsingError> {
        self.parse_kpoint_count()
            .and_then(|parser| parser.parse_spin_config())
            .and_then(|parser| parser.parse_electron_count())
            .and_then(|parser| parser.parse_eigenvalue_count())
            .and_then(|parser| parser.parse_fermi_energy())
            .and_then(|parser| parser.parse_unit_cell_vectors())
            .and_then(|parser| parser.parse_band_per_kpoint())
            .map_err(BandsParsingError::ContextError)?;
        // Reorganize data into spin-major format
        let kpoints = self
            .eigens_bands
            .as_ref()
            .map(|bands| {
                bands
                    .iter()
                    .map(|band_per_kpoint| band_per_kpoint.kpoint)
                    .collect::<Vec<KPoint>>()
            })
            .unwrap();
        let eigenvalues = match self.spin_polarized.unwrap() {
            SpinPolarized::True => {
                let (spin_1, spin_2): (Vec<Vec<f64>>, Vec<Vec<f64>>) = self
                    .eigens_bands
                    .unwrap()
                    .into_iter()
                    .map(|bands| -> (Vec<f64>, Vec<f64>) {
                        let (v1, v2) = bands.eigen_band.as_spin_polarized().unwrap();
                        (v1.to_vec(), v2.to_vec())
                    })
                    .unzip();
                Eigenvalues::SpinPolarized([spin_1, spin_2])
            }
            SpinPolarized::False => Eigenvalues::NonPolarized(
                self.eigens_bands
                    .unwrap()
                    .into_iter()
                    .map(|bands| bands.eigen_band.as_non_polarized().unwrap().to_vec())
                    .collect::<Vec<Vec<f64>>>(),
            ),
        };
        let band_structure = BandsFileBuilder::default()
            .spin_polarized(self.spin_polarized.unwrap())
            .lattice_vectors(self.lattice_vectors.unwrap())
            .fermi_energy(self.fermi_energy.unwrap())
            .kpoints(kpoints)
            .electron_count(self.electron_count.unwrap())
            .eigenvalues(eigenvalues)
            .build()?;
        Ok(band_structure)
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use super::BandsParser;

    // const BANDS_FILE: &str = "/home/tony/Downloads/cosxmos2_DOS/cosxmos2_DOS.bands";
    const BANDS_FILE: &str =
        "/home/tony/Downloads/Mg2SiO4_Dy_Bandstr_edft/Mg2SiO4_Dy_BandStr.bands";
    #[test]
    fn test_bands_parser() {
        let bands_content = read_to_string(BANDS_FILE).unwrap();
        let parser = BandsParser::new(&bands_content);
        let band_structure = parser.parse_bands_file().unwrap();
        dbg!(band_structure);
    }
}
