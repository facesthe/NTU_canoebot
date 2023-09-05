//! Library for common utility functions that are used by other crates.

mod macros;

use std::{
    error::Error,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use serde::{Serialize, Deserialize};
use veil::Redact;

/// String with contents hidden from the [Debug] trait
#[derive(Clone, Redact, Serialize, Deserialize)]
#[redact(all, fixed = 8)]
pub struct HiddenString {
    inner: String,
}

impl Deref for HiddenString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for HiddenString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: AsRef<str>> From<T> for HiddenString {
    fn from(value: T) -> Self {
        Self {
            inner: value.as_ref().to_string(),
        }
    }
}

impl FromStr for HiddenString {
    type Err = Box<dyn Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_redaction() {
        let x = HiddenString::from("asd");
        println!("{:?}", x);
    }
}
