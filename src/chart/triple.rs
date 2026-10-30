use plotters::prelude::BitMapBackend;

use super::*;

pub struct TripleChart<'a> {
    root_area: DrawingArea<BitMapBackend<'a>, Shift>,
    p99: Chart<'a>,
    p90: Chart<'a>,
    p50: Chart<'a>,
}

impl<'a> TripleChart<'a> {
    pub fn new(
        name: &'a String,
        x_range: Range<f32>,
        y_range: Range<f32>,
        title: &String,
        y_desc: String,
    ) -> PlottersResult<Self> {
        let root_area = BitMapBackend::new(name, (8000, 2000)).into_drawing_area();
        root_area.fill(&WHITE)?;
        let root_area = root_area.titled(title, ("sans-serif", 150).into_font())?;
        let mut v = root_area.split_evenly((1, 3));
        let p99 = v.pop().unwrap();
        let p90 = v.pop().unwrap();
        let p50 = v.pop().unwrap();
        Ok(Self {
            root_area,
            p99: to_chart_complex(
                p99,
                "99th percentile".to_string(),
                130,
                y_desc.clone(),
                x_range.clone(),
                y_range.clone(),
            )?,
            p90: to_chart_complex(
                p90,
                "90th percentile".to_string(),
                130,
                y_desc.clone(),
                x_range.clone(),
                y_range.clone(),
            )?,
            p50: to_chart_complex(
                p50,
                "50th percentile".to_string(),
                130,
                y_desc.clone(),
                x_range.clone(),
                y_range.clone(),
            )?,
        })
    }

    pub fn draw(&mut self) -> PlottersResult<()> {
        let _ = draw_subchart_complex(&mut self.p50, 80);
        let _ = draw_subchart_complex(&mut self.p90, 80);
        let _ = draw_subchart_complex(&mut self.p99, 80);

        self.root_area.present()
    }

    pub fn draw_line<const AMOUNT: usize>(
        &mut self,
        range: &Range<T>,
        result: &([usize; AMOUNT], [usize; AMOUNT], [usize; AMOUNT]),
        color: RGBColor,
        label: String,
    ) -> PlottersResult<()> {
        draw_line_single_chart_complex(
            &mut self.p50,
            range,
            &result.0[..],
            5,
            color,
            label.clone(),
        )?;
        draw_line_single_chart_complex(
            &mut self.p90,
            range,
            &result.1[..],
            5,
            color,
            label.clone(),
        )?;
        draw_line_single_chart_complex(
            &mut self.p99,
            range,
            &result.2[..],
            5,
            color,
            label.clone(),
        )?;
        Ok(())
    }

    pub fn draw_line_complex<const AMOUNT: usize>(
        &mut self,
        range: &Range<T>,
        result: &([usize; AMOUNT], [usize; AMOUNT], [usize; AMOUNT]),
        color: RGBColor,
        label: String,
        dashed: bool,
    ) -> PlottersResult<()> {
        if !dashed {
            draw_line_single_chart_complex(
                &mut self.p50,
                range,
                &result.0[..],
                5,
                color,
                label.clone(),
            )?;
            draw_line_single_chart_complex(
                &mut self.p90,
                range,
                &result.1[..],
                5,
                color,
                label.clone(),
            )?;
            draw_line_single_chart_complex(
                &mut self.p99,
                range,
                &result.2[..],
                5,
                color,
                label.clone(),
            )?;
        } else {
            draw_dashed_line_single_chart_complex(
                &mut self.p50,
                range,
                &result.0[..],
                5,
                color,
                label.clone(),
            )?;
            draw_dashed_line_single_chart_complex(
                &mut self.p90,
                range,
                &result.1[..],
                5,
                color,
                label.clone(),
            )?;
            draw_dashed_line_single_chart_complex(
                &mut self.p99,
                range,
                &result.2[..],
                5,
                color,
                label.clone(),
            )?;
        }
        Ok(())
    }
}
