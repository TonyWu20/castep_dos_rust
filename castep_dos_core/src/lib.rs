#![warn(missing_docs)]
#![allow(dead_code)]
//! Parsing implementation for `.pdos_weight`
/// All the data struct corresponding to the data in `.pdos_weight`
pub mod fundamental;

/// Helper functions for parsing
pub mod helper;

/// Parsing logics and function routines
pub mod pdos_weights_parser;

pub mod bands;

/// Projector preprocess
pub mod projectors;

/// calculation of PDOS
pub mod pdos_compute {
    use std::f64::consts::PI;

    use ndarray::{Array1, Array2};
    use rayon::iter::{
        IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
    };

    use crate::fundamental::{
        AngularChannels, BandStructure, EigenvalueVec, KpointVec, KpointWeight, SpinData,
    };

    const HATREE_TO_EV: f64 = 27.211396641308;

    /// Angular momentum-resolved projected DOS result
    #[derive(Debug, Clone, PartialEq)]
    pub struct PDOSResult {
        /// Channel s,
        pub s: Vec<f64>,
        /// Channel p,
        pub p: Vec<f64>,
        /// Channel d,
        pub d: Vec<f64>,
        /// Channel f
        pub f: Vec<f64>,
    }

    impl PDOSResult {
        /// Write as csv
        pub fn csv_output(&self, energy_grid: &[f64]) -> String {
            let header = "E,DOS_s,DOS_p,DOS_d,DOS_f";
            let contents = energy_grid
                .iter()
                .zip(self.s.iter())
                .zip(self.p.iter())
                .zip(self.d.iter())
                .zip(self.f.iter())
                .map(|((((e, s), p), d), f)| {
                    format!("{:.16},{:.16},{:.16},{:.16},{:.16}", e, s, p, d, f)
                })
                .collect::<Vec<String>>()
                .join("\n");
            [header.to_string(), contents].join("\n")
        }

        /// Get the max PDOS value for y-axis limit in plotting
        pub fn max(&self) -> f64 {
            let s_max = self.s.iter().copied().reduce(f64::max).unwrap_or(20.0);
            let p_max = self.p.iter().copied().reduce(f64::max).unwrap_or(20.0);
            let d_max = self.d.iter().copied().reduce(f64::max).unwrap_or(20.0);
            let f_max = self.f.iter().copied().reduce(f64::max).unwrap_or(20.0);
            [s_max, p_max, d_max, f_max]
                .into_iter()
                .reduce(f64::max)
                .unwrap()
        }
    }

    /// Calculate projected DOS with band structure data and projected angular momentum
    /// resolved weights
    /// # Returns
    /// The result will inherently keep the spin-polarization settings:
    /// - `SpinData::NonPolarized(PDOSResult { s, p, d, f, })'
    /// - `SpinData::SpinPolarized([PDOSResult { s_up, p_up, d_up, f_up, }, PDOSResult {s_down, p_down, d_down,f_down}])
    pub fn calculate_pdos(
        band_structure: &BandStructure,
        projected_weights: &SpinData<KpointVec<EigenvalueVec<AngularChannels>>>,
        energy_grid: &[f64],
        smearing: f64,
    ) -> SpinData<PDOSResult> {
        let band_eigenvalues = &band_structure.eigenvalues;
        let kpoint_weights = &band_structure.kpoint_weights;
        let fermi_energys = &band_structure.fermi_energy;
        band_eigenvalues
            .map_pair(fermi_energys, |kpts, &e_fermi| {
                kpts.iter()
                    .map(|eigens| {
                        eigens
                            .iter()
                            // Precompute the fermi energy shift
                            .map(|eigenvalue| (eigenvalue - e_fermi) * HATREE_TO_EV)
                            .collect()
                    })
                    .collect::<KpointVec<EigenvalueVec<f64>>>()
            })
            .map_pair(projected_weights, |eigens, pdos_weights| {
                calc_spin_pdos(eigens, pdos_weights, kpoint_weights, energy_grid, smearing)
            })
    }

    fn calc_spin_pdos(
        eigenvalues: &KpointVec<EigenvalueVec<f64>>,
        pdos_weights: &KpointVec<EigenvalueVec<AngularChannels>>,
        kpoint_weights: &KpointVec<KpointWeight>,
        energy_grid: &[f64],
        smearing: f64,
    ) -> PDOSResult {
        // precompute Gaussian smearing
        let norm = 1.0 / (smearing * ((2.0 * PI).sqrt()));
        let two_sigma_sq = 2.0 * smearing.powi(2);
        // Create energy grid as `Array1` for vectorized operations
        let energy_arr = Array1::from_vec(energy_grid.to_vec());

        // Initialize result array [channels x energy_points]
        let mut result = Array2::zeros((4, energy_grid.len()));

        // Process each k-point
        // Since the number of k-points is very limited, we don't use parallel
        // iterator here
        eigenvalues
            .iter()
            .zip(pdos_weights.iter())
            .zip(kpoint_weights.iter())
            .for_each(|((eigen_k, weights_k), kw)| {
                let kw_val = kw.value();

                // Precompute Gaussian factors for all eigenvalues at this k-point
                let factors = eigen_k
                    .par_iter()
                    .map(|&eigen| {
                        let e_delta = &energy_arr - eigen;
                        // kw * 1 / (ω * sqrt(2π)) * exp(-1/2 * (-d / ω)^2)
                        e_delta.mapv(|d| {
                            let gaussian_smearing = (-0.5 * ((-d).powi(2) / two_sigma_sq)).exp();
                            kw_val * norm * gaussian_smearing
                        })
                    })
                    .collect::<Vec<Array1<f64>>>();

                // Process eigenvalues in parallel
                let kpoint_result = factors
                    .into_par_iter()
                    .zip(weights_k.par_iter())
                    .map(|(factor_arr, am_weights)| {
                        let mut local = Array2::zeros((4, energy_grid.len()));
                        local.row_mut(0).assign(&(&factor_arr * am_weights.s));
                        local.row_mut(1).assign(&(&factor_arr * am_weights.p));
                        local.row_mut(2).assign(&(&factor_arr * am_weights.d));
                        local.row_mut(3).assign(&(&factor_arr * am_weights.f));

                        local
                    })
                    // Reduce all results of eigenvalues
                    .reduce(|| Array2::zeros((4, energy_grid.len())), |acc, e| acc + e);
                // Update to the result array
                result += &kpoint_result;
            });
        PDOSResult {
            s: result.row(0).to_vec(),
            p: result.row(1).to_vec(),
            d: result.row(2).to_vec(),
            f: result.row(3).to_vec(),
        }
    }

    #[cfg(test)]
    mod test {
        use std::fs::{read, read_to_string, write};

        use plotters::{
            chart::ChartBuilder,
            prelude::{DrawingAreaErrorKind, IntoDrawingArea, PathElement, SVGBackend},
            series::LineSeries,
            style::{Color, IntoFont, IntoTextStyle, RGBColor},
        };

        use crate::{
            bands::BandsParser, pdos_weights_parser::parse_pdos_weight_file, projectors::PDOSConfig,
        };

        use super::{PDOSResult, calculate_pdos};

        const BANDS_FILE: &str = "/home/tony/Downloads/cosxmos2_DOS/cosxmos2_DOS.bands";
        const PDOS_FILE: &str = "/home/tony/Downloads/cosxmos2_DOS/cosxmos2_DOS.pdos_bin";
        const CONFIG: &str = r#"
mapping=[{species="Mo", rank=2}, {species="S", rank=1}]
[[projector]]
[[projector.selections]]
species = "Mo"
#atoms = [1,]
[[projector.selections]]
species = "S"
#atoms = [1,]
"#;
        const CBA_BANDS: &str =
            "/home/tony/Downloads/Mg2SiO4_Dy_Bandstr_edft/Mg2SiO4_Dy_BandStr.bands";
        const CBA_PDOS: &str =
            "/home/tony/Downloads/Mg2SiO4_Dy_Bandstr_edft/Mg2SiO4_Dy_BandStr.pdos_weights";
        const CBA_CONFIG: &str = r#"
mapping=[{species="Dy", rank=4}]
[[projector]]
[[projector.selections]]
species = "Dy"
atoms = [1,]
#[[projector.selections]]
#species = "S"
#atoms = [1,]
"#;
        /// #e6e9ef
        const MANTLE: RGBColor = RGBColor(230, 233, 239);
        /// #1e2030
        // const MANTLE: RGBColor = RGBColor(49, 50, 68);
        const TEXT: RGBColor = RGBColor(76, 79, 105);
        /// #dd7878
        const FLAMINGO: RGBColor = RGBColor(221, 120, 120);
        /// #7287fd
        const LAVENDER: RGBColor = RGBColor(114, 135, 253);
        /// #df8e1d
        const YELLOW: RGBColor = RGBColor(223, 142, 29);
        /// #179299
        const TEAL: RGBColor = RGBColor(23, 146, 153);
        /// #4c4f69
        const BLACK: RGBColor = RGBColor(76, 79, 105);
        fn plot(
            energy_grid: &[f64],
            pdos: &PDOSResult,
            plotname: &str,
        ) -> Result<(), DrawingAreaErrorKind<std::io::Error>> {
            let plot_name = format!("{plotname}.svg");
            let root = SVGBackend::new(&plot_name, (1600, 900)).into_drawing_area();
            root.fill(&MANTLE)?;
            let root = root.margin(10, 10, 20, 10);
            let y_max = pdos.max();
            let x_min = energy_grid.first().unwrap() - 2.0;
            let x_max = energy_grid.last().unwrap() + 2.0;
            let mut chart = ChartBuilder::on(&root)
                .caption(
                    "Projected Density of States",
                    ("source sans pro, bold", 32).into_font().color(&TEXT),
                )
                .x_label_area_size(20)
                .y_label_area_size(40)
                .build_cartesian_2d(x_min..x_max, -1f64..y_max)
                .unwrap();
            chart
                .configure_mesh()
                .disable_mesh()
                // .light_line_style(MANTLE)
                .y_label_style(("source sans pro,bold", 28).with_color(TEXT))
                .x_label_style(("source sans pro,bold", 28).with_color(TEXT))
                .draw()
                .unwrap();
            let s_series = energy_grid
                .iter()
                .copied()
                .zip(pdos.s.iter().copied())
                .collect::<Vec<(f64, f64)>>();
            let p_series = energy_grid
                .iter()
                .copied()
                .zip(pdos.p.iter().copied())
                .collect::<Vec<(f64, f64)>>();
            let d_series = energy_grid
                .iter()
                .copied()
                .zip(pdos.d.iter().copied())
                .collect::<Vec<(f64, f64)>>();
            let f_series = energy_grid
                .iter()
                .copied()
                .zip(pdos.f.iter().copied())
                .collect::<Vec<(f64, f64)>>();
            chart
                .draw_series(LineSeries::new(s_series, LAVENDER.stroke_width(3)))
                .unwrap()
                .label("S")
                .legend(|(x, y)| {
                    PathElement::new(vec![(x, y), (x + 60, y)], LAVENDER.stroke_width(3))
                });
            chart
                .draw_series(LineSeries::new(p_series, TEAL.stroke_width(3)))
                .unwrap()
                .label("P")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 60, y)], TEAL.stroke_width(3)));
            chart
                .draw_series(LineSeries::new(d_series, FLAMINGO.stroke_width(3)))
                .unwrap()
                .label("D")
                .legend(|(x, y)| {
                    PathElement::new(vec![(x, y), (x + 60, y)], FLAMINGO.stroke_width(3))
                });
            chart
                .draw_series(LineSeries::new(f_series, YELLOW.stroke_width(3)))
                .unwrap()
                .label("F")
                .legend(|(x, y)| {
                    PathElement::new(vec![(x, y), (x + 60, y)], YELLOW.stroke_width(3))
                });

            chart
                .configure_series_labels()
                .legend_area_size(80)
                .label_font(("source sans pro, medium", 28).with_color(TEXT))
                .background_style(MANTLE.mix(0.8))
                .border_style(BLACK)
                .draw()?;
            root.present()
        }

        #[test]
        fn test_pdos_calc() {
            let pdos_file = read(PDOS_FILE).unwrap();
            let pdos_weights = parse_pdos_weight_file(&mut &pdos_file[..]).unwrap();
            let bands_file = read_to_string(BANDS_FILE).unwrap();
            let bands_parser = BandsParser::new(&bands_file);
            let band_structure = bands_parser.parse_bands_file().unwrap().to_band_structure();
            let config = toml::from_str::<PDOSConfig>(CONFIG).unwrap();
            let (min, max) = match band_structure.energy_range() {
                crate::fundamental::SpinData::NonPolarized((min, max)) => (min, max),
                crate::fundamental::SpinData::SpinPolarized([up, down]) => {
                    (up.0.min(down.0), up.1.max(down.1))
                }
            };
            let total_points = (max - min * 100.0_f64).ceil() as usize + 1;
            let energy_grid = (0..total_points)
                .map(|i| {
                    let fraction = i as f64 / (total_points - 1) as f64;
                    min + fraction * (max - min)
                })
                .collect::<Vec<f64>>();
            config
                .projectors
                .iter()
                .map(|proj_conf| {
                    proj_conf.project_pdos_from_config(&config.species_mapping(), &pdos_weights)
                })
                .for_each(|projected_weights| {
                    let result =
                        calculate_pdos(&band_structure, &projected_weights, &energy_grid, 0.1);
                    match result {
                        crate::fundamental::SpinData::NonPolarized(_item) => todo!(),
                        crate::fundamental::SpinData::SpinPolarized([up, down]) => {
                            let filename = "cosxmos2_DOS";
                            write(format!("{}_up.csv", filename), up.csv_output(&energy_grid))
                                .unwrap();
                            write(
                                format!("{}_down.csv", filename),
                                down.csv_output(&energy_grid),
                            )
                            .unwrap();
                            plot(&energy_grid, &up, &format!("{}_up", filename)).unwrap();
                            plot(&energy_grid, &down, &format!("{}_down", filename)).unwrap();
                        }
                    }
                });
        }
        #[test]
        fn test_cba() {
            let pdos_file = read(CBA_PDOS).unwrap();
            let pdos_weights = parse_pdos_weight_file(&mut &pdos_file[..]).unwrap();
            let bands_file = read_to_string(CBA_BANDS).unwrap();
            let bands_parser = BandsParser::new(&bands_file);
            let band_structure = bands_parser.parse_bands_file().unwrap().to_band_structure();
            let config = toml::from_str::<PDOSConfig>(CBA_CONFIG).unwrap();
            let (min, max) = match band_structure.energy_range() {
                crate::fundamental::SpinData::NonPolarized((min, max)) => (min, max),
                crate::fundamental::SpinData::SpinPolarized([up, down]) => {
                    (up.0.min(down.0), up.1.max(down.1))
                }
            };
            let total_points = (max - min * 100.0_f64).ceil() as usize + 1;
            let energy_grid = (0..total_points)
                .map(|i| {
                    let fraction = i as f64 / (total_points - 1) as f64;
                    min + fraction * (max - min)
                })
                .collect::<Vec<f64>>();
            config
                .projectors
                .iter()
                .map(|proj_conf| {
                    proj_conf.project_pdos_from_config(&config.species_mapping(), &pdos_weights)
                })
                .for_each(|projected_weights| {
                    let result =
                        calculate_pdos(&band_structure, &projected_weights, &energy_grid, 0.1);
                    match result {
                        crate::fundamental::SpinData::NonPolarized(res) => {
                            let csv_path = "Mg2SiO4_Dy_Bandstr_edft_Dy_pdos.csv";
                            let toml_path = "Mg2SiO4_Dy_Bandstr_edft_Dy_pdos.toml";
                            write(csv_path, res.csv_output(&energy_grid)).unwrap();
                            write(toml_path, toml::to_string_pretty(&config).unwrap()).unwrap();
                            plot(&energy_grid, &res, "Mg2SiO4_Dy_Bandstr_edft_Dy_pdos").unwrap();
                        }
                        crate::fundamental::SpinData::SpinPolarized(_) => todo!(),
                    }
                });
        }
    }
}
