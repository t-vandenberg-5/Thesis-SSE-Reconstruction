use std::{ops::Range};

use crate::T;
use plotters::{
    chart::{ChartBuilder, ChartContext}, coord::{
        Shift, cartesian::Cartesian2d, combinators::{IntoLogRange, LogCoord}, types::RangedCoordf32
    }, drawing::{DrawingArea, DrawingAreaErrorKind, IntoDrawingArea}, element::{DashedPathElement, PathElement}, prelude::BitMapBackend, series::{DashedLineSeries, LineSeries}, style::{Color, IntoFont, RGBColor, ShapeStyle, full_palette::{BLACK, WHITE}}
};
use plotters_bitmap::BitMapBackendError;

type PlottersResult<T> = Result<T, DrawingAreaErrorKind<BitMapBackendError>>;
type Chart<'a> = ChartContext<'a, BitMapBackend<'a>, Cartesian2d<RangedCoordf32, LogCoord<f32>>>;

pub mod triple;
pub mod quad;

fn to_chart<'a>(
    area: DrawingArea<BitMapBackend<'a>, Shift>,
    caption: String,
    x_range: Range<f32>,
    y_range: Range<f32>,
) -> PlottersResult<Chart<'a>> {
    to_chart_complex(area, caption, 20, "Uniform random non-unique queries".to_string(), x_range, y_range)
}

fn to_chart_complex<'a>(
    area: DrawingArea<BitMapBackend<'a>, Shift>,
    caption: String,
    caption_size: u32,
    y_desc: String,
    x_range: Range<f32>,
    y_range: Range<f32>,
) -> PlottersResult<Chart<'a>> {
    let mut chart = ChartBuilder::on(&area)
        .caption(caption, ("sans-serif", caption_size).into_font())
        .margin(caption_size / 2)
        .x_label_area_size(caption_size / 2 * 3)
        .y_label_area_size(caption_size * 2)
        .build_cartesian_2d(x_range, y_range.log_scale())?;

    chart
        .configure_mesh()
        .x_desc("Domain size in each dimension")
        .y_desc(y_desc)
        .x_label_style(("sans-serif", caption_size / 3 * 2).into_font())
        .y_label_style(("sans-serif", caption_size / 3 * 2).into_font())
        .x_label_formatter(&|x| format!("{:.0}", x))
        .y_label_formatter(&|x| format!("{:e}", x))
        .draw()?;

    Ok(chart)
}

fn draw_subchart<'a>(chart: &mut Chart<'a>) -> PlottersResult<()>{
    chart
        .configure_series_labels()
        .position(plotters::chart::SeriesLabelPosition::UpperRight)
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    Ok(())
}

fn draw_subchart_complex<'a>(chart: &mut Chart<'a>, label_size: u32) -> PlottersResult<()> {
    chart
        .configure_series_labels()
        .position(plotters::chart::SeriesLabelPosition::UpperRight)
        .label_font(("sans-serif", label_size).into_font())
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    Ok(())
}

fn draw_line_single_chart<'a>(
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
        .label(label)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
    Ok(())
}

fn draw_line_single_chart_complex<'a>(
    chart: &mut Chart<'a>,
    range: &Range<T>,
    result: &[usize],
    width: u32,
    color: RGBColor,
    label: String,
) -> PlottersResult<()> {
    // chart
    //     .draw_series(LineSeries::new(
    //             (range.clone()).map(|x| {
    //                 (
    //                     x as f32,
    //                     result[(x - range.clone().min().unwrap()) as usize] as f32,
    //                 )
    //             }),
    //             Into::<ShapeStyle>::into(&BLACK).stroke_width(width),
    //     ))?;
    chart
        .draw_series(LineSeries::new(
                (range.clone()).map(|x| {
                    (
                        x as f32,
                        result[(x - range.clone().min().unwrap()) as usize] as f32,
                    )
                }),
                Into::<ShapeStyle>::into(&color).stroke_width(width),
        ))?
        .label(label)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20 as i32, y)], color.stroke_width(width)));
    Ok(())
}

fn draw_dashed_line_single_chart_complex<'a>(
    chart: &mut Chart<'a>,
    range: &Range<T>,
    result: &[usize],
    width: u32,
    color: RGBColor,
    label: String,
) -> PlottersResult<()> {
    // chart
    //     .draw_series(LineSeries::new(
    //             (range.clone()).map(|x| {
    //                 (
    //                     x as f32,
    //                     result[(x - range.clone().min().unwrap()) as usize] as f32,
    //                 )
    //             }),
    //             Into::<ShapeStyle>::into(&BLACK).stroke_width(width),
    //     ))?;
    chart
        .draw_series(DashedLineSeries::new(
                (range.clone()).map(|x| {
                    (
                        x as f32,
                        result[(x - range.clone().min().unwrap()) as usize] as f32,
                    )
                }),
                5,
                10,
                ShapeStyle { color: color.into(), filled: false, stroke_width: width },
        ))?
        .label(label)
        .legend(move |(x, y)| DashedPathElement::new(vec![(x, y), (x + 20 as i32, y)], 5, 10, color.stroke_width(width)));
    Ok(())
}
