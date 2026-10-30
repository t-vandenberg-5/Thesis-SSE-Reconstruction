use std::{
    fs::File,
    io::{Read, Stdin, stdin},
};

use csv::{Reader, ReaderBuilder, StringRecord};

#[derive(Clone)]
pub struct CSVFilter {
    pub column: usize,
    pub predicate: fn(&str) -> bool,
}

impl CSVFilter {}

#[derive(Clone)]
pub struct CSVConfig {
    filters: Vec<CSVFilter>,
    skip_first_line: bool,
    flexible: bool,
}

impl CSVConfig {
    fn skip(&self) -> usize {
        if self.skip_first_line { 1 } else { 0 }
    }

    fn matches_filters(&self, record: &StringRecord) -> bool {
        for filter in &self.filters {
            if let Some(s) = record.get(filter.column) {
                if !(filter.predicate)(s) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

impl From<CSVConfigBuilder> for CSVConfig {
    fn from(value: CSVConfigBuilder) -> Self {
        value.config
    }
}

#[derive(Clone)]
pub struct CSVConfigBuilder {
    config: CSVConfig,
}

impl Default for CSVConfigBuilder {
    fn default() -> Self {
        Self {
            config: CSVConfig {
                filters: Vec::new(),
                skip_first_line: true,
                flexible: true,
            },
        }
    }
}

impl CSVConfigBuilder {
    pub fn filter(mut self, column: usize, predicate: fn(&str) -> bool) -> Self {
        self.config.filters.push(CSVFilter { column, predicate });
        self
    }

    pub fn skip_first_line(mut self, skip: bool) -> Self {
        self.config.skip_first_line = skip;
        self
    }

    pub fn rb_file(&self, path: String) -> Result<Reader<File>, std::io::Error> {
        Ok(ReaderBuilder::new()
            .flexible(self.config.flexible)
            .from_reader(File::open(path)?))
    }

    pub fn rb(&self) -> Reader<Stdin> {
        ReaderBuilder::new()
            .flexible(self.config.flexible)
            .from_reader(stdin())
    }

    pub fn read_iter<'a, T: Read>(
        &self,
        reader: &'a mut Reader<T>,
    ) -> impl Iterator<Item = StringRecord> {
        reader
            .records()
            .skip(self.config.skip())
            .filter_map(move |result| {
                let r = result.ok()?;
                if self.config.matches_filters(&r) {
                    Some(r)
                } else {
                    None
                }
            })
    }
}

pub fn zeroable_bits(coll: &mut Vec<String>) -> u32 {
    coll.sort();
    coll.dedup();
    println!("Unique record count: {}", coll.len());
    let mut zeroed_bits = 0;
    // let mut mantissa_and_sign_check: Vec<f32> = coll.iter().map(|x| {
    //     let mut f: f32 = x.parse().unwrap();
    //     f32::from_bits(f.to_bits() & !((1 << (zeroed_bits + 1)) - 1))
    // }).collect();
    // mantissa_and_sign_check.sort_by(|a, b| a.total_cmp(b));
    // mantissa_and_sign_check.dedup();
    // let mantissa_mask = !((1 << 9) - 1);
    // let mantissa_and_sign = mantissa_and_sign_check[0].to_bits() & mantissa_mask;
    // for v in mantissa_and_sign_check {
    //     assert_eq!(v.to_bits() & mantissa_mask, mantissa_and_sign);
    // }
    loop {
        let mut c: Vec<f32> = coll
            .iter()
            .map(|x| {
                let f: f32 = x.parse().unwrap();
                f32::from_bits(f.to_bits() & !((1 << (zeroed_bits + 1)) - 1))
            })
            .collect();
        c.sort_by(|a, b| a.total_cmp(b));
        println!("min {}, max {}", c.first().unwrap(), c.last().unwrap());
        let mut smallest_diff = 1000f32;
        for i in 1..c.len() {
            if c[i] - c[i - 1] < smallest_diff && c[i] - c[i - 1] > 0f32 {
                smallest_diff = c[i] - c[i - 1]
            }
        }
        println!("Smallest diff: {}", smallest_diff);
        c.dedup();
        println!("Unique record count: {}", c.len());
        if c.len() == coll.len() {
            zeroed_bits += 1
        } else {
            break;
        }
    }
    zeroed_bits
}

pub fn domain_size(coll: &mut Vec<String>) -> (u32, (u32, u32)) {
    let zeroable = zeroable_bits(coll);
    let mut c: Vec<f32> = coll
        .iter()
        .map(|x| {
            let f: f32 = x.parse().unwrap();
            f32::from_bits(f.to_bits() & !((1 << (zeroable)) - 1))
        })
        .collect();
    c.sort_by(|a, b| a.total_cmp(b));
    let full_diff = c.last().unwrap().to_bits() - c.first().unwrap().to_bits();
    (
        full_diff >> zeroable,
        (
            c.first().unwrap().to_bits() >> zeroable,
            c.last().unwrap().to_bits() >> zeroable,
        ),
    )
}

pub fn parse(record: &String, minimum: u64) -> u16 {
    let after_decimal = if record[1..2] == *"." { 2 } else { 3 };
    let mut naive = (record[after_decimal..].parse::<u64>().unwrap()
        * 10u64.pow(10 - (record.len() as u32 - 1)))
        + 1;
    naive +=
        record[..after_decimal - 1].parse::<u64>().unwrap() * 10u64.pow(11 - after_decimal as u32);
    let raw = (naive * 9) / if after_decimal == 2 { 2500000 } else { 250000 };
    (raw - minimum) as u16
}

pub fn get_minimum(coll: &mut Vec<String>) -> u64 {
    coll.sort();
    coll.dedup();
    println!("{}", coll.len());
    let mut deviation: Vec<u64> = coll
        .iter()
        .map(|x| {
            let after_decimal = if x[1..2] == *"." { 2 } else { 3 };
            let mut naive = (x[after_decimal..].parse::<u64>().unwrap()
                * 10u64.pow(10 - (x.len() as u32 - 1)))
                + 1;
            naive += x[..after_decimal - 1].parse::<u64>().unwrap()
                * 10u64.pow(11 - after_decimal as u32);
            (naive * 9) / if after_decimal == 2 { 2500000 } else { 250000 }
        })
        .collect();
    deviation.sort();
    deviation.dedup();
    println!("{}", deviation.len());
    println!("max: {}", deviation[deviation.len() - 1] - deviation[0]);
    deviation[0]
}
