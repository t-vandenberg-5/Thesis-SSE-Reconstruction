use super::*;

pub struct QuadChart<'a> {
    root_area: DrawingArea<BitMapBackend<'a>, Shift>,
    p99: Chart<'a>,
    p90: Chart<'a>,
    p50: Chart<'a>,
    p50_l: Chart<'a>,
}

impl<'a> QuadChart<'a> {
    pub fn new(name: &'a String, range: Range<f32>) -> PlottersResult<Self> {
        let root_area = BitMapBackend::new(name, (2000, 2000)).into_drawing_area();
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
            p99: to_chart(
                p99,
                "p99".to_string(),
                range.clone(),
                1e0f32..1e8f32,
            )?,
            p90: to_chart(
                p90,
                "p90".to_string(),
                range.clone(),
                1e0f32..1e8f32,
            )?,
            p50: to_chart(
                p50,
                "p50".to_string(),
                range.clone(),
                1e0f32..1e8f32,
            )?,
            p50_l: to_chart(
                p50_l,
                "p50_l".to_string(),
                range,
                1f32..1e13f32,
            )?,
        })
    }

    pub fn draw(&mut self) -> PlottersResult<()> {
        let _ = draw_subchart(&mut self.p50_l);
        let _ = draw_subchart(&mut self.p50);
        let _ = draw_subchart(&mut self.p90);
        let _ = draw_subchart(&mut self.p99);

        self.root_area.present()
    }


    pub fn draw_line<const AMOUNT: usize>(
        &mut self,
        range: &Range<T>,
        result: &([usize; AMOUNT], [usize; AMOUNT], [usize; AMOUNT]),
        color: RGBColor,
        label: String,
    ) -> PlottersResult<()> {
        draw_line_single_chart(&mut self.p50_l, range, &result.0[..], color, label.clone())?;
        draw_line_single_chart(&mut self.p50, range, &result.0[..], color, label.clone())?;
        draw_line_single_chart(&mut self.p90, range, &result.1[..], color, label.clone())?;
        draw_line_single_chart(&mut self.p99, range, &result.2[..], color, label.clone())?;
        Ok(())
    }
}
