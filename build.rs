use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const DATA_PATH: &str = "assets/data.ron";
const OUT_PATH: &str = "data.rs";

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    probabilities: HashMap<String, Decimal>,
}

fn main() {
    let data = get_or_create_data();
    if let Err(e) = verify_data(&data) {
        panic!("{:?}", e);
    }
    codegen(&data);
    println!("cargo::rerun-if-changed=build.rs,assets/data.ron");
}

fn get_or_create_data() -> Data {
    let data_path = PathBuf::from(DATA_PATH);

    if !data_path.exists() {
        // generate default file
        todo!();
    }

    let contents = fs::read_to_string(data_path).unwrap();
    ron::from_str(&contents).expect("Data file invalid")
}

fn verify_data(data: &Data) -> Result<(), String> {
    let sum: Decimal = data.probabilities.values().sum();
    if sum != dec!(1.0) {
        return Err("Probabilities must add to precisely 1.0".to_string());
    }

    Ok(())
}

fn codegen(data: &Data) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join(PathBuf::from(OUT_PATH));

    let mut insertions = String::new();
    for key in data.probabilities.keys() {
        let value = data.probabilities.get(key).unwrap();
        let line = format!("\"{}\" => dec!({}),\n", key, value);
        insertions.push_str(&line);
    }

    let out_string = format!(
        "
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use phf::phf_map;
    
    static PROBABILITIES: phf::Map<&str, Decimal> = phf_map! {{
        {}
    }};
    ",
        insertions
    );

    std::fs::write(&out_path, out_string).unwrap();
}
