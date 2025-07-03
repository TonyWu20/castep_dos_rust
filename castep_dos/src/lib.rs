pub mod config;
pub mod plot {
    use castep_dos_core::pdos_compute::PDOSResult;
    use plotters::{
        chart::ChartBuilder,
        prelude::{DrawingAreaErrorKind, IntoDrawingArea, PathElement, SVGBackend},
        series::LineSeries,
        style::{Color, IntoFont, IntoTextStyle, RGBColor},
    };

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

    pub fn plot(
        energy_grid: &[f64],
        pdos: &PDOSResult,
        plotname: &str,
    ) -> Result<(), DrawingAreaErrorKind<std::io::Error>> {
        let plot_name = format!("{plotname}.svg");
        let root = SVGBackend::new(&plot_name, (1600, 900)).into_drawing_area();
        root.fill(&MANTLE)?;
        let root = root.margin(10, 10, 40, 10);
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
            .draw_series(LineSeries::new(s_series, LAVENDER.stroke_width(2)))
            .unwrap()
            .label("S")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 60, y)], LAVENDER.stroke_width(2)));
        chart
            .draw_series(LineSeries::new(p_series, TEAL.stroke_width(2)))
            .unwrap()
            .label("P")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 60, y)], TEAL.stroke_width(2)));
        chart
            .draw_series(LineSeries::new(d_series, FLAMINGO.stroke_width(2)))
            .unwrap()
            .label("D")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 60, y)], FLAMINGO.stroke_width(2)));
        chart
            .draw_series(LineSeries::new(f_series, YELLOW.stroke_width(2)))
            .unwrap()
            .label("F")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 60, y)], YELLOW.stroke_width(3)));
        chart
            .draw_series(LineSeries::new(
                [(0.0, 0.0), (0.0, y_max)],
                TEXT.stroke_width(2),
            ))
            .unwrap();

        chart
            .configure_series_labels()
            .legend_area_size(80)
            .label_font(("source sans pro, medium", 28).with_color(TEXT))
            .background_style(MANTLE.mix(0.8))
            .border_style(BLACK)
            .draw()?;
        root.present()
    }
}
