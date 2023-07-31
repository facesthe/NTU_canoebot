//! Boat deconflict module
#![allow(unused)]

use std::{collections::BTreeMap, hash::Hash};

use lazy_static::__Deref;
use polars::export::ahash::{HashMap, HashSet};

/// Boat allocation type
pub struct BoatAllocations {
    inner: BTreeMap<String, (Option<String>, Option<String>)>,
}

impl __Deref for BoatAllocations {
    type Target = BTreeMap<String, (Option<String>, Option<String>)>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl BoatAllocations {
    /// Perform a deconflict run
    pub fn deconflict(&self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::<String, String>::default();

        for (name, (pri, alt)) in self.iter() {
            map.insert(name.to_owned(), pri.to_owned().unwrap());

            let conflicts = Self::find_matching_values(&map);

            if conflicts.len() == 0 {
                continue;
            } else {
                // deconflict here

                // iterating over previous, deconflicted data
                // means that current data only has 1 conflict
                let (conf_val, conf_keys) = conflicts.iter().next().unwrap();
                let conf_a = &conf_keys[0];
                let conf_b = &conf_keys[1];
            }
        }

        map
    }

    /// Directly assign each person their primary boat,
    /// ignoring any conflicts.
    pub fn assign(&self) -> BTreeMap<String, Option<String>> {
        self.iter()
            .map(|(k, v)| (k.to_owned(), v.0.to_owned()))
            .collect::<BTreeMap<String, Option<String>>>()
    }

    /// Looks for matching values and returns the value as key,
    /// with keys as a vector of values
    fn find_matching_values(map: &BTreeMap<String, String>) -> HashMap<String, Vec<String>> {
        let mut set = HashSet::<String>::default();
        let mut matching_map = HashMap::<String, Vec<String>>::default();

        for (key, val) in map.iter() {
            if set.contains(key) {
                if matching_map.contains_key(val) {
                    let v = matching_map.get_mut(val).unwrap();
                    v.push(key.to_owned());
                } else {
                    matching_map.insert(val.to_owned(), vec![key.to_owned()]);
                }
            } else {
                set.insert(key.to_owned());
            }
        }

        matching_map
    }
}

/// Checks if an iterator contains unique elements
fn has_unique_elements<T, Element>(iter: T) -> bool
where
    T: IntoIterator<Item = Element>,

    Element: Eq + Hash,
{
    let vec: Vec<Element> = iter.into_iter().collect();
    let num_elements = vec.len();
    let set: HashSet<Element> = vec.into_iter().collect();

    if set.len() == num_elements {
        true
    } else {
        false
    }
}
