use clap::{Parser, ValueEnum};

// enum Database { Grid }

#[derive(ValueEnum, Clone, Debug)]
enum QueryStrategy {
    Random,
    BetterRandom,
    Known,
}

impl ToString for QueryStrategy {
    fn to_string(&self) -> String {
        match self {
            QueryStrategy::Random => "random".to_string(),
            QueryStrategy::BetterRandom => "better-random".to_string(),
            QueryStrategy::Known => "known".to_string(),
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
enum InsertionStrategy {
    None,
    Full,
    Midline,
    Naive,
    Iterative,
}

impl ToString for InsertionStrategy {
    fn to_string(&self) -> String {
        match self {
            InsertionStrategy::None => "none".to_string(),
            InsertionStrategy::Full => "full".to_string(),
            InsertionStrategy::Midline => "midline".to_string(),
            InsertionStrategy::Naive => "naive".to_string(),
            InsertionStrategy::Iterative => "iterative".to_string(),
        }
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Amount of dimensions of the database
    #[arg(short, long, default_value_t = 3)]
    dimensions: u8,

    /// Domain of all dimensions, currently only supports hypercube shaped databases
    #[arg(short, long, default_value_t = 100)]
    domain: u128,

    /// Maximum number of worker threads to use
    #[arg(short, long, default_value_t = 4)]
    threads: u16,

    /// Number of times to attempt reconstruction
    #[arg(short, long, default_value_t = 100)]
    attempts: u16,

    /// Strategy used to determine insertions
    #[arg(short, long, default_value_t = InsertionStrategy::None)]
    insertion: InsertionStrategy,
}

fn main() {
    let args = Args::parse();

    println!("Hello {}!", args.dimensions);
}
