use acf_parser::prelude::*;

fn main() {
    let file = "./acfs/appmanifest_730.acf";
    let result = parse_acf(file);

    match result {
        Ok(val) => println!("{:#?}", val),
        Err(e) => println!("Parsing error: {e}"),
    }
}
