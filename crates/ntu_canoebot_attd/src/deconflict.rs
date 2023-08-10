//! Boat deconflict module
#![allow(unused)]

use std::{collections::BTreeMap, hash::Hash};

use lazy_static::__Deref;
use ntu_canoebot_util::debug_println;
use polars::export::ahash::{HashMap, HashSet};

use crate::{get_config_type, Config, NameList, BOAT_ALLOCATIONS};

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

impl NameList {
    /// Assign everyone their primary boats.
    /// If deconflict is set to true, perform deconflict.
    ///
    /// Returns false when deconflict fails.
    pub async fn assign_boats(&mut self, deconflict: bool) -> bool {
        let config = get_config_type(self.date);

        match deconflict {
            false => {
                let allo_lock = BOAT_ALLOCATIONS[config as usize].read().await;

                let assigned: Vec<Option<String>> = self
                    .names
                    .iter()
                    .map(|n| {
                        if allo_lock.contains_key(n) {
                            allo_lock.get(n).unwrap().0.to_owned()
                        } else {
                            None
                        }
                    })
                    .collect();

                self.boats = Some(assigned);
                return true;
            }
            // generate a frequency count
            true => {
                let potential_matches = Self::find_matching(&self.names, config);

                todo!()
            }
        }
    }

    /// Group names that might potentially share the same boat
    async fn find_matching(names: &[String], config: Config) -> Vec<Vec<String>> {
        // vec of names and if a name has been used (true => used, false => not used)
        let mut remaining_names: Vec<(&str, bool)> =
            names.iter().map(|n| (n.as_str(), false)).collect();

        // names set for current group
        let mut names_set = HashSet::<&str>::default();
        let mut allo_set = HashSet::<&str>::default();

        // groups
        let mut groups: Vec<Vec<String>> = Vec::new();
        let read_lock = BOAT_ALLOCATIONS[config as usize].read().await;

        // each iteration of the main loop must create a new list
        while remaining_names.len() != 0 {
            // debug_println!("remaining names: {}", remaining_names.len());
            let mut group: Vec<String> = Vec::new();
            names_set.clear();
            allo_set.clear();

            for (name, used) in remaining_names.iter_mut() {
                let (pri, alt) = {
                    let boats = read_lock.get(*name).unwrap();
                    let p = boats.0.as_deref();
                    let a = boats.1.as_deref();

                    (p, a)
                };

                // empty case
                if names_set.len() == 0 {
                    group.push(name.to_owned());
                    names_set.insert(name);
                    if let Some(_p) = pri {
                        allo_set.insert(_p);
                    }
                    if let Some(_a) = alt {
                        allo_set.insert(_a);
                    }
                }

                match names_set.contains(name) {
                    // name exists, pri and alt boats added to set
                    true => {
                        *used = true;

                        if let Some(primary) = pri {
                            allo_set.insert(primary);
                        }
                        if let Some(alternate) = alt {
                            allo_set.insert(alternate);
                        }
                    }
                    // check if boats exist in allo set
                    false => {
                        let to_add = match (pri, alt) {
                            (None, None) => false,
                            (None, Some(a)) => {
                                if allo_set.contains(a) {
                                    true
                                } else {
                                    false
                                }
                            }
                            (Some(p), None) => {
                                if allo_set.contains(p) {
                                    true
                                } else {
                                    false
                                }
                            }
                            (Some(p), Some(a)) => {
                                if allo_set.contains(p) || allo_set.contains(a) {
                                    allo_set.insert(p);
                                    allo_set.insert(a);
                                    true
                                } else {
                                    false
                                }
                            }
                        };

                        if to_add {
                            *used = true;
                            names_set.insert(name);
                            group.push(name.to_owned());
                        }
                    }
                }
            }

            // remove used names
            remaining_names = remaining_names
                .into_iter()
                .filter(|(_, used)| !used)
                .collect();
            // debug_println!("removed {} names", group.len());

            debug_println!("names: {:?}", names_set);
            debug_println!("potential conflicts: {:?}", allo_set);

            groups.push(group);
        }

        // todo!();
        groups
    }

    /// Internal deconflict method.
    ///
    /// Pass in a list of names and confg,
    /// Returns a (hopefully) deconflicted list of boats and if the operation
    /// is successful.
    ///
    async fn deconflict(names: &Vec<String>, config: Config) -> (Vec<Option<&str>>, bool) {
        // identify names with the same boat

        todo!()
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{Config, NameList, BOAT_ALLOCATIONS, NAMES_CERTS};

    #[tokio::test]
    async fn assign_no_deconflict() {
        crate::init().await;

        let mut name_list = crate::namelist(NaiveDate::from_ymd_opt(2023, 1, 14).unwrap(), false)
            .await
            .unwrap();
        name_list.assign_boats(false).await;

        println!("namelist with directly assigned boats:\n{}", name_list);
    }

    /// Find matching groups against the whole config
    #[tokio::test]
    async fn test_find_matching() {
        crate::init().await;

        let config = Config::New;

        let read_lock: tokio::sync::RwLockReadGuard<
            '_,
            std::collections::HashMap<String, (Option<String>, Option<String>)>,
        > = BOAT_ALLOCATIONS[config as usize].read().await;
        let names: Vec<String> = read_lock.iter().map(|(k, _)| k.to_owned()).collect();

        let groups = NameList::find_matching(&names, config).await;

        println!("boat allocations:\n{:?}\n", read_lock);
        println!("potential conflicting groups:\n{:?}\n", groups);
    }
}
