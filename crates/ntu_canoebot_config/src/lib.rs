//! A crate for accessing variables inside configuration files.
//!
//! ## Configuration sources
//! Configuration settings are read in from these 3 sources,
//! in the `.configs` directory (project root):
//! - `botsettings.template.toml` (already exists)
//! - `botsettings.template.debug.toml` (optional)
//! - `botsettings.template.deploy.toml` (optional)
//!
//! Either `debug` or `deploy` should exist.
//! If both do not exist, the template file will be used.
//! This **will** result in runtime panics!
//!
//! If both files exist, the first that contains the key-value pair
//! `use = true` will be used.
//! `deploy` will be checked first, followed by `debug`.
//!
//! If neither file has the `use = true` pair set, the build process will exit.
//!
//! ## Usage
//! - All nested tables are destructured to become absolute keys.
//! - Each inner table (namespace) is separated by an underscore `_`
//! - Absolute keys can be any toml type except for tables
//! - Absolute keys are stored directly as their toml data type
//! - Absolute keys that point to arrays need to have each array element to be the same type
//! - Last level tables (tables that do not contain tables) are also stored as a const hashmap. All hashmap values are strings.
//! - All keys are stored in uppercase.
//!
//! ```
//! use ntu_canoebot_config as config;
//!
//! /// retrieve the api key by absolute key
//! let key: &str = *config::CANOEBOT_APIKEY;
//!
//! /// retrieve a config by it's absolute key
//! let is_enabled: bool = *config::EVENTS_DAILY_LOGSHEET_PROMPT_ENABLE;
//!
//! /// retrieve the same config from a lookup table as a string
//! let is_enabled: &str = config::EVENTS_DAILY_LOGSHEET_PROMPT.get("ENABLE").unwrap();
//!
//! ```

include!("../generated.rs");

#[cfg(test)]
mod tests {
    // included here so that build.rs can unit test as well
    #![allow(unused)]
    include!("../build.rs");
}
