use reconstructor::dataset::{CSVConfigBuilder, domain_size, get_minimum, zeroable_bits};

fn main() {
    let column1 = 4;
    let column2 = 5;
    let builder = CSVConfigBuilder::default()
        .filter(column1, |x| x.len() > 2)
        .filter(column1, |x| x[1..2] == *"." || x[2..3] == *".")
        .filter(column2, |x| x.len() > 2)
        .filter(column2, |x| x[1..2] == *"." || x[2..3] == *".")
        ;
    let mut reader = builder.rb();
    let coll: Vec<(String, String)> = builder.read_iter(&mut reader).map(|x| (x.get(column1).unwrap().to_string(), x.get(column2).unwrap().to_string())).collect();
    let (mut coll1, mut coll2) = coll.into_iter().unzip();
    println!("max dev c1: {:?}", get_minimum(&mut coll1));
    println!("max dev c2: {:?}", get_minimum(&mut coll2));
    let z1 = zeroable_bits(&mut coll1);
    let z2 = zeroable_bits(&mut coll2);
    println!("{}, {}", z1, z2);
    let d1 = domain_size(&mut coll1);
    let d2 = domain_size(&mut coll2);
    println!("{:?}, {:?}", d1, d2);
}
