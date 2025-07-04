use castep_dos_core::pdos_compute::PDOSResult;
use plotters::{
    chart::{ChartBuilder, LabelAreaPosition},
    prelude::{DrawingAreaErrorKind, IntoDrawingArea, IntoLinspace, PathElement, SVGBackend},
    series::LineSeries,
    style::{AsRelative, Color, FontDesc, FontStyle, IntoTextStyle, RGBColor, register_font},
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
/// #7c7f93
const OVERLAY: RGBColor = RGBColor(124, 127, 147);

pub fn plot(
    energy_grid: &[f64],
    pdos: &PDOSResult,
    plotname: &str,
) -> Result<(), DrawingAreaErrorKind<std::io::Error>> {
    let plot_name = format!("{plotname}.svg");
    let source_sans_pro_black = include_bytes!("SourceSans3-Black.otf");
    let source_sans_pro_semibold = include_bytes!("SourceSans3-Semibold.otf");
    let source_sans_pro_bold = include_bytes!("SourceSans3-Bold.otf");
    register_font(
        "source sans pro,black",
        plotters::style::FontStyle::Bold,
        source_sans_pro_black,
    )
    .unwrap_or_else(|_| panic!("Register Source Sans Pro Black failed"));
    register_font(
        "source sans pro,semibold",
        plotters::style::FontStyle::Normal,
        source_sans_pro_semibold,
    )
    .unwrap_or_else(|_| panic!("Register Source Sans Pro Semibold failed"));
    register_font(
        "source sans pro,bold",
        plotters::style::FontStyle::Normal,
        source_sans_pro_bold,
    )
    .unwrap_or_else(|_| panic!("Register Source Sans Pro Bold failed"));
    let caption_style = FontDesc::new(
        plotters::style::FontFamily::Name("source sans pro,black"),
        32.0,
        FontStyle::Bold,
    )
    .with_color(TEXT);

    let root = SVGBackend::new(&plot_name, (1600, 900)).into_drawing_area();
    root.fill(&MANTLE)?;
    let root = root.margin(10, 40, 60, 10);
    let y_max = pdos.max() + 2.0;
    let x_min = *energy_grid.first().unwrap();
    let x_max = *energy_grid.last().unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption("Projected Density of States", caption_style)
        .set_label_area_size(LabelAreaPosition::Left, (8).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, (10).percent())
        .margin((1).percent())
        .build_cartesian_2d(x_min..x_max, (0.0..y_max).step(2.0))
        .unwrap();
    chart
        .configure_mesh()
        .disable_x_mesh()
        .y_desc("DOS (electrons/eV)")
        .x_desc("Energy (eV)")
        .axis_desc_style(
            FontDesc::new(
                plotters::style::FontFamily::Name("source sans pro,semibold"),
                22.0,
                FontStyle::Normal,
            )
            .with_color(TEXT),
        )
        // .y_labels((y_max - 0.0).ceil() as usize / 2)
        // .y_max_light_lines((y_max - 0.0).ceil() as usize / 2)
        .x_labels((x_max - x_min).ceil() as usize / 5)
        // .light_line_style(MANTLE)
        .y_label_style(
            FontDesc::new(
                plotters::style::FontFamily::Name("source sans pro,semibold"),
                22.0,
                FontStyle::Normal,
            )
            .with_color(TEXT),
        )
        .x_label_style(
            FontDesc::new(
                plotters::style::FontFamily::Name("source sans pro,semibold"),
                22.0,
                FontStyle::Normal,
            )
            .with_color(TEXT),
        )
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
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 60, y)], LAVENDER.stroke_width(3)));
    chart
        .draw_series(LineSeries::new(p_series, TEAL.stroke_width(2)))
        .unwrap()
        .label("P")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 60, y)], TEAL.stroke_width(3)));
    chart
        .draw_series(LineSeries::new(d_series, FLAMINGO.stroke_width(2)))
        .unwrap()
        .label("D")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 60, y)], FLAMINGO.stroke_width(3)));
    chart
        .draw_series(LineSeries::new(f_series, YELLOW.stroke_width(2)))
        .unwrap()
        .label("F")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 60, y)], YELLOW.stroke_width(3)));
    chart
        .draw_series(LineSeries::new(
            [(0.0, 0.0), (0.0, y_max)],
            OVERLAY.stroke_width(2),
        ))
        .unwrap();

    chart
        .configure_series_labels()
        .legend_area_size(80)
        .label_font(
            FontDesc::new(
                plotters::style::FontFamily::Name("source sans pro,bold"),
                32.0,
                FontStyle::Bold,
            )
            .with_color(TEXT),
        )
        .background_style(MANTLE.mix(0.8))
        .border_style(BLACK)
        .draw()?;
    root.present()
}
