//! This build script parses the config files and generates compile-time constants
//! for use in the program.

const SETTINGS_TEMPLATE: &str = "botsettings.template.toml";
const SETTINGS_DEBUG: &str = "botsettings.template.debug.toml";
const SETTINGS_DEPLOY: &str = "botsettings.template.deploy.toml";

const CONFIGS_PATH: &str = "../../.configs/";

enum Setting {
    Template = 0,
    Debug = 1,
    Deploy = 2,
}

use core::panic;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    process::exit,
    str::FromStr,
};
use toml::{self, Value};

fn main() {
    let settings_arr = vec![
        format!("{}{}", CONFIGS_PATH, SETTINGS_TEMPLATE),
        format!("{}{}", CONFIGS_PATH, SETTINGS_DEBUG),
        format!("{}{}", CONFIGS_PATH, SETTINGS_DEPLOY),
    ];

    // rerun this file if these files change
    println!("cargo:rerun-if-changed=build.rs");
    let _ = settings_arr
        .iter()
        .map(|setting| println!("cargo:rerun-if-changed={}", setting));

    let mut settings_contents = Vec::new();

    let template_result = read_append_to_vec(&mut settings_contents, &settings_arr[0]);
    if !template_result {
        panic!("file should exist: {}", settings_arr[0]);
    }

    let deploy_file: bool;
    let debug_file: bool;

    debug_file = read_append_to_vec(&mut settings_contents, &settings_arr[1]);
    deploy_file = read_append_to_vec(&mut settings_contents, &settings_arr[2]);

    let file_to_use: usize; // indexes into settings_arr
    match (debug_file, deploy_file) {
        (true, true) => {
            let debug = toml::Table::from_str(&settings_contents[Setting::Debug as usize]).unwrap();
            let deploy =
                toml::Table::from_str(&settings_contents[Setting::Deploy as usize]).unwrap();

            let debug_use = debug
                .get("use")
                .and_then(|val| match val {
                    Value::Boolean(_b) => Some(_b.to_owned()),
                    _ => None,
                })
                .unwrap_or(false);

            let deploy_use = deploy
                .get("use")
                .and_then(|val| match val {
                    Value::Boolean(_b) => Some(_b.to_owned()),
                    _ => None,
                })
                .unwrap_or(false);

            match (debug_use, deploy_use) {
                (true, true) => file_to_use = Setting::Deploy as usize,
                (true, false) => file_to_use = Setting::Debug as usize,
                (false, true) => file_to_use = Setting::Deploy as usize,
                (false, false) => {
                    println!("cargo:warning=\"use = true\" pair not set for both files. Set this key-value pair inside one configuration file.");
                    exit(0)
                    // file_to_use = Setting::Deploy as usize;
                }
            }
        }
        (true, false) => file_to_use = Setting::Debug as usize,
        (false, true) => file_to_use = Setting::Deploy as usize,
        (false, false) => {
            file_to_use = Setting::Template as usize; // merge into self, effectively doing nothing

            println!("cargo:warning=debug/deploy file missing. At least one file required:");
            println!("cargo:warning=- {}", settings_arr[Setting::Debug as usize]);
            println!("cargo:warning=- {}", settings_arr[Setting::Deploy as usize]);
            println!("cargo:warning=Default settings may cause panics on runtime.");
        }
    }

    let merged = merge_tables(
        &toml::Table::from_str(&settings_contents[Setting::Template as usize]).unwrap(),
        &toml::Table::from_str(&settings_contents[file_to_use]).unwrap(),
    );

    let hash_table = table_to_hashmap(&merged, None);
    generate_code_file(hash_table);
}

/// ChatGPT generated
/// Compares base against other.
/// Merges any matching keys from other to base.
fn merge_tables(template: &toml::Table, changes: &toml::Table) -> toml::Table {
    let mut merged_table = template.clone();

    for (key, value) in changes.iter() {
        if let Some(existing_value) = merged_table.get_mut(key) {
            if let Some(existing_table) = existing_value.as_table_mut() {
                if let Some(changes_table) = value.as_table() {
                    // Recursively merge the tables
                    let merged_subtable = merge_tables(existing_table, changes_table);
                    *existing_value = toml::Value::Table(merged_subtable);
                    continue;
                }
            }
        }

        // Update the value directly if it doesn't exist in the template or cannot be merged
        merged_table.insert(key.clone(), value.clone());
    }

    merged_table
}

/// Checks if file exists, and appends to vec.
/// Returns true and appends to vec if file exists,
/// returns false and appends an empty string if file does not exist.
fn read_append_to_vec(vec: &mut Vec<String>, file_path: &str) -> bool {
    if Path::new(file_path).exists() {
        vec.push(fs::read_to_string(file_path).unwrap());
        true
    } else {
        vec.push(format!(""));
        false
    }
}

/// Convert a toml table to a hashmap by flattening
fn table_to_hashmap(table: &toml::Table, prefix: Option<&str>) -> HashMap<String, Value> {
    let mut map = HashMap::<String, Value>::new();

    for (key, val) in table.iter() {
        let mut _key = key.to_owned().to_uppercase().replace("-", "_");
        if let Some(pre) = prefix {
            _key = format!("{}_{}", pre, _key);
        }

        if let Value::Table(t) = val {
            let sub_map = table_to_hashmap(t, Some(_key.as_str()));
            map.extend(sub_map);
        } else {
            map.insert(_key, val.to_owned());
        }
    }

    map
}

/// Create and populate the generated file.
/// This file will reside in this crate's root alongside `build.rs`.
fn generate_code_file(variables: HashMap<String, Value>) {
    let mut gen_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("generated.rs")
        .unwrap();

    gen_file
        .write_all("/// The contents of this file are automatically generated.\n\n".as_bytes())
        .unwrap();

    for (key, val) in variables.iter() {
        let (value_literal, value_type) = match val {
            Value::String(s) => (format!("\"{}\"", s), "&str"),
            Value::Integer(i) => (i.to_string(), "i64"),
            Value::Float(f) => (f.to_string(), "f64"),
            Value::Boolean(b) => (b.to_string(), "bool"),
            Value::Datetime(dt) => (format!("\"{}\"", dt), "&str"),
            // Value::Array(_) => todo!(), // never taken
            // Value::Table(_) => todo!(), // never taken
            _ => todo!(),
        };

        let generated_line = format!(
            r#"pub const {}: {} = {};
"#,
            key, value_type, value_literal
        );

        gen_file.write_all(generated_line.as_bytes()).unwrap();
    }
}

#[cfg(test)]
mod test {
    use std::{fmt::format, fs, str::FromStr};

    use toml::de;

    use crate::{merge_tables, CONFIGS_PATH, SETTINGS_DEBUG, SETTINGS_TEMPLATE};

    #[test]
    fn test_merge_table() {
        let template_file = fs::read_to_string(format!("{}{}", CONFIGS_PATH, SETTINGS_TEMPLATE))
            .expect("template file should exist");

        let debug_file = fs::read_to_string(format!("{}{}", CONFIGS_PATH, SETTINGS_DEBUG))
            .expect("debug file should exist");

        let template = toml::Table::from_str(&template_file).expect("should be a valid toml file");
        let debug = toml::Table::from_str(&debug_file).expect("debug file should exist");

        let merged = merge_tables(&template, &debug);

        println!("{}\n\n{}\n\n{}", template, debug, merged);
    }
}
