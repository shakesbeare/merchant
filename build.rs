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
    probabilities: HashMap<String, f32>,
    provisions_kinds: Vec<String>,
    supplies_kinds: Vec<String>,
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
    let sum: f32 = data.probabilities.values().sum();
    if sum != 1.0 {
        return Err("Probabilities must add to precisely 1.0".to_string());
    }

    Ok(())
}

fn codegen(data: &Data) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join(PathBuf::from(OUT_PATH));

    let mut kinds = String::new();
    let mut probs = String::new();
    for key in data.probabilities.keys() {
        let value = data.probabilities.get(key).unwrap();
        let line = format!("(ItemKind::{}, {}),\n", key, value);
        let enum_entry = format!("{},\n", key);
        probs.push_str(&line);
        kinds.push_str(&enum_entry);
    }

    let mut prov_kinds = String::new();
    for kind in data.provisions_kinds.iter() {
        let line = format!("{},\n", kind);
        prov_kinds.push_str(&line);
    }

    let mut supp_kinds = String::new();
    let mut supp_strs = String::new();
    for supp in data.supplies_kinds.iter() {
        let kind = supp.chars().filter(|c| c.is_alphanumeric()).collect::<String>();
        let enum_variant = format!("{},\n", kind);
        let string_rep = format!("Self::{} => write!(f, \"{}\"),\n", kind, supp);
        supp_kinds.push_str(&enum_variant);
        supp_strs.push_str(&string_rep);
    }

    let out_string = format!(
        "
    use std::collections::HashMap;

    lazy_static::lazy_static! {{
        static ref PROBABILITIES: HashMap<ItemKind, f32> = HashMap::from([
            {}
        ]);
    }}


    #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, enum_derived::Rand)]
    pub enum ProvisionsKind {{
        {}
    }}

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, enum_derived::Rand)]
    pub enum ItemKind {{
        {}
    }}

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, enum_derived::Rand)]
    pub enum SuppliesKind {{
        {}
    }}

    impl std::fmt::Display for SuppliesKind {{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
            match self {{
                {}
            }}
        }}
    }}

    ",
        probs, prov_kinds, kinds, supp_kinds, supp_strs,
    );

    std::fs::write(&out_path, out_string).unwrap();
}
