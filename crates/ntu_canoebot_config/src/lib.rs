//! A crate for accessing variables inside configuration files.
//!

/// Retrieve a config param.
///
/// ```no_run
/// use ntu_canoebot_config::config;
///
/// let value = config!("nested::config::in::file");
/// // use your config value
/// ```
#[macro_export]
macro_rules! config {
    ($param: literal) => {{
        let param: &str = $param;

        let keys = param.split("::");

        let constructed_var: Vec<String> = keys.into_iter().map(|k| k.to_uppercase()).collect();

        // constructed_var.join("_")
        let env_var = constructed_var.join("_");
        std::env::var(&env_var)
            .expect(format!("failed to get environment variable: {}", &env_var).as_str())
    }};
}

/// Module for build.rs code
mod build {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_lookup() {
        std::env::set_var("THIS_VARIABLE", "set");
        let config = config!("this::variable");
        println!("config param: {}", config);
    }

    #[test]
    // #[should_panic]
    fn test_unsuccessful_lookup() {
        // std::env::var("").expect("asd");
        let result = std::panic::catch_unwind(|| config!("invalid::config::path"));
        assert!(result.is_err());
    }
}
