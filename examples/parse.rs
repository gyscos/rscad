use std::io::Read;

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();

    // println!("{}", &input[66..]);

    let res = rscad::parse(&input).unwrap();
    println!("{:#?}", res);
}
