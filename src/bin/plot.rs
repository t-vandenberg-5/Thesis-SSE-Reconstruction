use plotters::style::full_palette::*;
use reconstructor::{dataset::CSVConfigBuilder, chart::triple::TripleChart as Chart};

fn main() -> Result<(), Box<dyn std::error::Error>>{
    type T = u16;
    const MIN: T = 10;
    const MAX: T = 100;
    const MAX3D: T = 50;

    let name = format!("plot-sqv-{}-{}.png", MIN, MAX);
    let range = MIN..MAX;
    let range3d = MIN..MAX3D;

    let c = CSVConfigBuilder::default().skip_first_line(false);
    let mut f = c.rb_file("2dlog-p50.csv".to_string())?;
    let mut ri50 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(0) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("2dlog-p90.csv".to_string())?;
    let mut ri90 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(0) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("2dlog-p99.csv".to_string())?;
    let mut ri99 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(0) { Some(y.to_string()) } else { None });

    let mut chart = Chart::new(&name, (MIN as f32)..(MAX as f32), 1e0f32..1e15f32, &"Amount of search queries needed to reconstruct a database with 1000 records.".to_string(), "Search Queries".to_string())?;
    let mut arr0 = (
        [0; (MAX-MIN) as usize], 
        [0; (MAX-MIN) as usize], 
        [0; (MAX-MIN) as usize], 
    );
    for i in 0..arr0.0.len() {
        arr0.0[i] = ri50.next().unwrap().parse().unwrap();
        arr0.1[i] = ri90.next().unwrap().parse().unwrap();
        arr0.2[i] = ri99.next().unwrap().parse().unwrap();
    }

    let mut f = c.rb_file("2dlog-p50.csv".to_string())?;
    let mut ri50 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(2) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("2dlog-p90.csv".to_string())?;
    let mut ri90 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(2) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("2dlog-p99.csv".to_string())?;
    let mut ri99 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(2) { Some(y.to_string()) } else { None });

    let mut arr1 = (
        [0; (MAX-MIN) as usize], 
        [0; (MAX-MIN) as usize], 
        [0; (MAX-MIN) as usize]
    );
    for i in 0..arr1.0.len() {
        arr1.0[i] = ri50.next().unwrap().parse().unwrap();
        arr1.1[i] = ri90.next().unwrap().parse().unwrap();
        arr1.2[i] = ri99.next().unwrap().parse().unwrap();
    }

    let mut arr2 = (
        [0; (MAX-MIN) as usize], 
        [0; (MAX-MIN) as usize], 
        [0; (MAX-MIN) as usize]
    );
    for i in 0..arr2.0.len() {
        let j = MIN as usize + i;
        let x = ((j * (j + 1)) as f32 / 2f32).powi(2);
        // arr2.0[i] = x as usize;
        arr2.0[i] = (x*(x.ln())) as usize;
        arr2.1[i] = arr2.0[i];
        arr2.2[i] = arr2.0[i];
    }

    let mut f = c.rb_file("3dlog-p50.csv".to_string())?;
    let mut ri503d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(0) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("3dlog-p90.csv".to_string())?;
    let mut ri903d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(0) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("3dlog-p99.csv".to_string())?;
    let mut ri993d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(0) { Some(y.to_string()) } else { None });
    let mut arr3 = (
        [0; (MAX3D-MIN) as usize], 
        [0; (MAX3D-MIN) as usize], 
        [0; (MAX3D-MIN) as usize]
    );
    for i in 0..arr3.0.len() {
        arr3.0[i] = ri503d.next().unwrap().parse().unwrap();
        arr3.1[i] = ri903d.next().unwrap().parse().unwrap();
        arr3.2[i] = ri993d.next().unwrap().parse().unwrap();
    }

    let mut f = c.rb_file("3dlog-p50.csv".to_string())?;
    let mut ri503d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(2) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("3dlog-p90.csv".to_string())?;
    let mut ri903d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(2) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("3dlog-p99.csv".to_string())?;
    let mut ri993d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(2) { Some(y.to_string()) } else { None });

    let mut arr4 = (
        [0; (MAX3D-MIN) as usize], 
        [0; (MAX3D-MIN) as usize], 
        [0; (MAX3D-MIN) as usize]
    );
    for i in 0..arr4.0.len() {
        arr4.0[i] = ri503d.next().unwrap().parse().unwrap();
        arr4.1[i] = ri903d.next().unwrap().parse().unwrap();
        arr4.2[i] = ri993d.next().unwrap().parse().unwrap();
    }

    let mut arr5 = (
        [0; (MAX-MIN) as usize], 
        [0; (MAX-MIN) as usize], 
        [0; (MAX-MIN) as usize]
    );
    for i in 0..arr5.0.len() {
        let j = MIN as usize + i;
        let x = ((j * (j + 1)) as f32 / 2f32).powi(3);
        arr5.0[i] = (x*(x.ln())) as usize;
        arr5.1[i] = arr5.0[i];
        arr5.2[i] = arr5.0[i];
    }

    chart.draw_line_complex(&range, &arr5, BLUE, "3D: Total response multiset".to_string(), true)?;
    chart.draw_line_complex(&range3d, &arr4, RED, "3D: Midline reconstruction".to_string(), true)?;
    chart.draw_line_complex(&range3d, &arr3, BLACK, "3D: Full reconstruction".to_string(), true)?;
    chart.draw_line(&range, &arr2, BLUE, "2D: Total response multiset".to_string())?;
    chart.draw_line(&range, &arr1, RED, "2D: Midline reconstruction".to_string())?;
    chart.draw_line(&range, &arr0, BLACK, "2D: Full reconstruction".to_string())?;
    chart.draw()?;

    let name = format!("plot-iqv-{}-{}.png", MIN, MAX);
    let range = MIN..MAX;

    let c = CSVConfigBuilder::default().skip_first_line(false);
    let mut f = c.rb_file("2dlog-p50.csv".to_string())?;
    let mut ri50 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(1) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("2dlog-p90.csv".to_string())?;
    let mut ri90 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(1) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("2dlog-p99.csv".to_string())?;
    let mut ri99 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(1) { Some(y.to_string()) } else { None });

    let mut chart2 = Chart::new(&name, (MIN as f32)..(MAX as f32), 1e1f32..1e6f32, &"Amount of insertion queries needed to reconstruct a database with 1000 records.".to_string(), "Search Queries".to_string())?;
    let mut arr0 = ([0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize]);
    for i in 0..arr0.0.len() {
        arr0.0[i] = ri50.next().unwrap().parse().unwrap();
        arr0.1[i] = ri90.next().unwrap().parse().unwrap();
        arr0.2[i] = ri99.next().unwrap().parse().unwrap();
    }

    let mut f = c.rb_file("2dlog-p50.csv".to_string())?;
    let mut ri50 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(3) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("2dlog-p90.csv".to_string())?;
    let mut ri90 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(3) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("2dlog-p99.csv".to_string())?;
    let mut ri99 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(3) { Some(y.to_string()) } else { None });

    let mut arr1 = ([0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize]);
    for i in 0..arr1.0.len() {
        arr1.0[i] = ri50.next().unwrap().parse().unwrap();
        arr1.1[i] = ri90.next().unwrap().parse().unwrap();
        arr1.2[i] = ri99.next().unwrap().parse().unwrap();
    }

    let mut arr2 = ([0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize]);
    for i in 0..arr2.0.len() {
        let j = MIN as usize + i;
        let x = j*2;
        arr2.0[i] = x;
        arr2.1[i] = arr2.0[i];
        arr2.2[i] = arr2.0[i];
    }

    let mut arr3 = ([0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize]);
    for i in 0..arr3.0.len() {
        let j = MIN as usize + i;
        let x = j.pow(2);
        arr3.0[i] = x;
        arr3.1[i] = arr3.0[i];
        arr3.2[i] = arr3.0[i];
    }
    println!("sf: {:?}", &arr0);
    let mut f = c.rb_file("3dlog-p50.csv".to_string())?;
    let mut ri503d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(1) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("3dlog-p90.csv".to_string())?;
    let mut ri903d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(1) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("3dlog-p99.csv".to_string())?;
    let mut ri993d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(1) { Some(y.to_string()) } else { None });

    let mut arr4 = ([0; (MAX3D-MIN) as usize], [0; (MAX3D-MIN) as usize], [0; (MAX3D-MIN) as usize]);
    for i in 0..arr4.0.len() {
        arr4.0[i] = ri503d.next().unwrap().parse().unwrap();
        arr4.1[i] = ri903d.next().unwrap().parse().unwrap();
        arr4.2[i] = ri993d.next().unwrap().parse().unwrap();
    }

    let mut f = c.rb_file("3dlog-p50.csv".to_string())?;
    let mut ri503d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(3) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("3dlog-p90.csv".to_string())?;
    let mut ri903d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(3) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("3dlog-p99.csv".to_string())?;
    let mut ri993d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(3) { Some(y.to_string()) } else { None });

    let mut arr5 = ([0; (MAX3D-MIN) as usize], [0; (MAX3D-MIN) as usize], [0; (MAX3D-MIN) as usize]);
    for i in 0..arr5.0.len() {
        arr5.0[i] = ri503d.next().unwrap().parse().unwrap();
        arr5.1[i] = ri903d.next().unwrap().parse().unwrap();
        arr5.2[i] = ri993d.next().unwrap().parse().unwrap();
    }

    let mut arr6 = ([0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize]);
    for i in 0..arr6.0.len() {
        let j = MIN as usize + i;
        let x = j*3;
        arr6.0[i] = x;
        arr6.1[i] = arr6.0[i];
        arr6.2[i] = arr6.0[i];
    }

    let mut arr7 = ([0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize]);
    for i in 0..arr7.0.len() {
        let j = MIN as usize + i;
        let x = j.pow(3);
        arr7.0[i] = x;
        arr7.1[i] = arr7.0[i];
        arr7.2[i] = arr7.0[i];
    }

    chart2.draw_line_complex(&range, &arr7, BLACK, "3D: Full insertion".to_string(), true)?;
    chart2.draw_line_complex(&range3d, &arr4, BLUE, "3D: Simple (full)".to_string(), true)?;
    chart2.draw_line_complex(&range3d, &arr5, GREEN, "3D: Simple (midline)".to_string(), true)?;
    chart2.draw_line_complex(&range, &arr6, RED, "3D: Midline insertion".to_string(), true)?;
    chart2.draw_line(&range, &arr3, BLACK, "2D: Full insertion".to_string())?;
    chart2.draw_line(&range, &arr0, BLUE, "2D: Simple (full)".to_string())?;
    chart2.draw_line(&range, &arr1, GREEN, "2D: Simple (midline)".to_string())?;
    chart2.draw_line(&range, &arr2, RED, "2D: Midline insertion".to_string())?;
    // chart.draw_line(&range, &result_inject, GREEN, "3D: Full reconstruction injected".to_string())?;
    // chart.draw_line(&range, &result_inject2, ORANGE, "3D: Full reconstruction injected 2".to_string())?;
    //
    // chart.draw_line(&range, &result_midline, RED, "3D: Midline reconstruction".to_string())?;
    // chart.draw_line(&range, &result_midline_inject, PURPLE, "3D: Midline reconstruction injected".to_string())?;
    // chart.draw_line(&range, &result_midline_inject2, YELLOW, "3D: Midline reconstruction injected 2".to_string())?;
    //
    // let v: [_; AMOUNT] = core::array::from_fn(|x| (((x + MIN as usize) as f32)*(((x + MIN as usize) as f32).ln())) as usize);
    // let all = (v, v, v);
    //
    // chart.draw_line(&range, &all, CYAN, "3D: All".to_string())?;

    chart2.draw()?;
    let c = CSVConfigBuilder::default().skip_first_line(false);

    let name = format!("plot-oqv-{}-{}.png", MIN, MAX);
    let mut chart3 = Chart::new(&name, (MIN as f32)..(MAX as f32), 1e1f32..1e6f32, &"Amount of insertion queries needed to reconstruct a database with 1000 records.".to_string(), "Search Queries".to_string())?;

    let mut f = c.rb_file("2dlog-p50.csv".to_string())?;
    let mut ri50 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(4) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("2dlog-p90.csv".to_string())?;
    let mut ri90 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(4) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("2dlog-p99.csv".to_string())?;
    let mut ri99 = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(4) { Some(y.to_string()) } else { None });

    let mut arr1 = ([0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize]);
    for i in 0..arr1.0.len() {
        arr1.0[i] = ri50.next().unwrap().parse().unwrap();
        arr1.1[i] = ri90.next().unwrap().parse().unwrap();
        arr1.2[i] = ri99.next().unwrap().parse().unwrap();
    }

    let mut arr2 = ([0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize]);
    for i in 0..arr2.0.len() {
        let j = MIN as usize + i;
        let x = j*2;
        arr2.0[i] = x;
        arr2.1[i] = arr2.0[i];
        arr2.2[i] = arr2.0[i];
    }

    let mut arr3 = ([0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize]);
    for i in 0..arr3.0.len() {
        let j = MIN as usize + i;
        let x = j.pow(2);
        arr3.0[i] = x;
        arr3.1[i] = arr3.0[i];
        arr3.2[i] = arr3.0[i];
    }
    println!("sf: {:?}", &arr0);

    let mut f = c.rb_file("3dlog-p50.csv".to_string())?;
    let mut ri503d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(4) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("3dlog-p90.csv".to_string())?;
    let mut ri903d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(4) { Some(y.to_string()) } else { None });
    let mut f = c.rb_file("3dlog-p99.csv".to_string())?;
    let mut ri993d = c.read_iter(&mut f).filter_map(|x| if let Some(y) = x.get(4) { Some(y.to_string()) } else { None });

    let mut arr5 = ([0; (MAX3D-MIN) as usize], [0; (MAX3D-MIN) as usize], [0; (MAX3D-MIN) as usize]);
    for i in 0..arr5.0.len() {
        arr5.0[i] = ri503d.next().unwrap().parse().unwrap();
        arr5.1[i] = ri903d.next().unwrap().parse().unwrap();
        arr5.2[i] = ri993d.next().unwrap().parse().unwrap();
    }

    let mut arr6 = ([0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize]);
    for i in 0..arr6.0.len() {
        let j = MIN as usize + i;
        let x = j*3;
        arr6.0[i] = x;
        arr6.1[i] = arr6.0[i];
        arr6.2[i] = arr6.0[i];
    }

    let mut arr7 = ([0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize], [0; (MAX-MIN) as usize]);
    for i in 0..arr7.0.len() {
        let j = MIN as usize + i;
        let x = j.pow(3);
        arr7.0[i] = x;
        arr7.1[i] = arr7.0[i];
        arr7.2[i] = arr7.0[i];
    }

    chart3.draw_line_complex(&range, &arr7, BLACK, "3D: Full insertion".to_string(), true)?;
    chart3.draw_line_complex(&range3d, &arr5, GREEN, "3D: Advanced (midline)".to_string(), true)?;
    chart3.draw_line_complex(&range, &arr6, RED, "3D: Midline insertion".to_string(), true)?;
    chart3.draw_line(&range, &arr3, BLACK, "2D: Full insertion".to_string())?;
    chart3.draw_line(&range, &arr1, GREEN, "2D: Advanced (midline)".to_string())?;
    chart3.draw_line(&range, &arr2, RED, "2D: Midline insertion".to_string())?;
    // chart.draw_line(&range, &result_inject, GREEN, "3D: Full reconstruction injected".to_string())?;
    // chart.draw_line(&range, &result_inject2, ORANGE, "3D: Full reconstruction injected 2".to_string())?;
    //
    // chart.draw_line(&range, &result_midline, RED, "3D: Midline reconstruction".to_string())?;
    // chart.draw_line(&range, &result_midline_inject, PURPLE, "3D: Midline reconstruction injected".to_string())?;
    // chart.draw_line(&range, &result_midline_inject2, YELLOW, "3D: Midline reconstruction injected 2".to_string())?;
    //
    // let v: [_; AMOUNT] = core::array::from_fn(|x| (((x + MIN as usize) as f32)*(((x + MIN as usize) as f32).ln())) as usize);
    // let all = (v, v, v);
    //
    // chart.draw_line(&range, &all, CYAN, "3D: All".to_string())?;

    Ok(chart3.draw()?)
}
