//! A crate for accessing variables inside configuration files.
//!
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

include!("../generated.rs");
