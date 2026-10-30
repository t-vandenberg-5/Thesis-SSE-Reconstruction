use std::ops::Range;

use crate::T;
use plotters::{
    backend::SVGBackend,
    chart::{ChartBuilder, ChartContext},
    coord::{
        Shift, cartesian::Cartesian2d, combinators::{IntoLogRange, LogCoord}, types::RangedCoordf32
    },
    drawing::{DrawingArea, DrawingAreaErrorKind, IntoDrawingArea},
    element::PathElement,
    series::LineSeries,
    style::{Color, IntoFont, RGBColor, full_palette::{BLACK, WHITE}},
};

type PlottersResult<T> = Result<T, DrawingAreaErrorKind<std::io::Error>>;
type Chart<'a> = ChartContext<'a, SVGBackend<'a>, Cartesian2d<RangedCoordf32, LogCoord<f32>>>;

pub struct QuadChart<'a> {
    root_area: DrawingArea<SVGBackend<'a>, Shift>,
    p99: Chart<'a>,
    p90: Chart<'a>,
    p50: Chart<'a>,
    p50_l: Chart<'a>,
}

impl<'a> QuadChart<'a> {
    pub fn new(name: &'a String, range: Range<f32>) -> PlottersResult<Self> {
        let root_area = SVGBackend::new(name, (2000, 2000)).into_drawing_area();
        root_area.fill(&WHITE)?;
        let root_area = root_area.titled(
            "Amount of queries needed to reconstruct a database with 10^d records.",
            ("sans-serif", 30).into_font(),
        )?;
        let mut v = root_area.split_evenly((2, 2));
        let p99 = v.pop().unwrap();
        let p90 = v.pop().unwrap();
        let p50_l = v.pop().unwrap();
        let p50 = v.pop().unwrap();
        Ok(Self {
            root_area,
            p99: Self::to_chart(
                p99,
                "p99".to_string(),
                range.clone(),
                1e0f32..1e8f32,
            )?,
            p90: Self::to_chart(
                p90,
                "p90".to_string(),
                range.clone(),
                1e0f32..1e8f32,
            )?,
            p50: Self::to_chart(
                p50,
                "p50".to_string(),
                range.clone(),
                1e0f32..1e8f32,
            )?,
            p50_l: Self::to_chart(
                p50_l,
                "p50_l".to_string(),
                range,
                1f32..1e13f32,
            )?,
        })
    }

    fn draw_subchart(chart: &mut Chart<'a>) -> Result<(), DrawingAreaErrorKind<std::io::Error>>{
        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
        Ok(())
    }

    pub fn draw(&mut self) -> Result<(), DrawingAreaErrorKind<std::io::Error>> {
        let _ = Self::draw_subchart(&mut self.p50_l);
        let _ = Self::draw_subchart(&mut self.p50);
        let _ = Self::draw_subchart(&mut self.p90);
        let _ = Self::draw_subchart(&mut self.p99);

        self.root_area.present()
    }

    fn to_chart(
        area: DrawingArea<SVGBackend<'a>, Shift>,
        caption: String,
        x_range: Range<f32>,
        y_range: Range<f32>,
    ) -> PlottersResult<Chart<'a>> {
        let mut chart = ChartBuilder::on(&area)
            .caption(caption, ("sans-serif", 20).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(80)
            .build_cartesian_2d(x_range, y_range.log_scale())?;

        chart
            .configure_mesh()
            .x_desc("Domain size in each dimension")
            .y_desc("Uniform random non-unique queries")
            .y_label_formatter(&|x| format!("{:e}", x))
            .draw()?;

        Ok(chart)
    }

    fn draw_line_single_chart(
        chart: &mut Chart<'a>,
        range: &Range<T>,
        result: &[usize],
        color: RGBColor,
        label: String,
    ) -> PlottersResult<()> {
        chart
            .draw_series(LineSeries::new(
                (range.clone()).map(|x| {
                    (
                        x as f32,
                        result[(x - range.clone().min().unwrap()) as usize] as f32,
                    )
                }),
                &color,
            ))?
            .label("TEST".clone())
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        Ok(())
    }

    pub fn draw_line<const AMOUNT: usize>(
        &mut self,
        range: &Range<T>,
        result: &([usize; AMOUNT], [usize; AMOUNT], [usize; AMOUNT]),
        color: RGBColor,
        label: String,
    ) -> PlottersResult<()> {
        println!("Drawing");
        Self::draw_line_single_chart(&mut self.p50_l, range, &result.0[..], color, label.clone())?;
        Self::draw_line_single_chart(&mut self.p50, range, &result.0[..], color, label.clone())?;
        Self::draw_line_single_chart(&mut self.p90, range, &result.1[..], color, label.clone())?;
        Self::draw_line_single_chart(&mut self.p99, range, &result.2[..], color, label.clone())?;
        Ok(())
    }
}
