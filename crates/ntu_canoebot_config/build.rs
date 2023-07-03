#![allow(unused)]

const SETTINGS_TEMPLATE: &str = "botsettings.template.toml";
const SETTINGS_DEBUG: &str = "botsettings.template.debug.toml";
const SETTINGS_DEPLOY: &str = "botsettings.template.deploy.toml";

const CONFIGS_PATH: &str = "../../.configs/";

enum Setting {
    Template = 0,
    Debug = 1,
    Deploy = 2,
}

use std::{collections::HashMap, fs, path::Path, str::FromStr};
use toml;

fn main() {
    let settings_arr = vec![
        format!("{}{}", CONFIGS_PATH, SETTINGS_TEMPLATE),
        format!("{}{}", CONFIGS_PATH, SETTINGS_DEBUG),
        format!("{}{}", CONFIGS_PATH, SETTINGS_DEPLOY),
    ];

    // rerun this file if these files change
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
        (true, true) => file_to_use = Setting::Deploy as usize,
        (true, false) => file_to_use = Setting::Debug as usize,
        (false, true) => file_to_use = Setting::Deploy as usize,
        (false, false) => panic!(
            "debug/deploy file missing. At least one file required:\n-{}\n-{}",
            settings_arr[0], settings_arr[1]
        ),
    }

    let merged = merge_tables(
        &toml::Table::from_str(&settings_contents[Setting::Template as usize]).unwrap(),
        &toml::Table::from_str(&settings_contents[file_to_use]).unwrap(),
    );
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
fn table_to_hashmap(table: toml::Table) -> HashMap<String, String> {
    for (key, val) in table.iter() {}

    todo!()
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
