//! Boat deconflict module

use std::collections::{BTreeMap, HashMap, HashSet};

use ntu_canoebot_util::{debug_print, debug_println};

use crate::{get_config_type, Config, NameList, BOAT_ALLOCATIONS};

/// This struct contains the boat allocation result.
/// If lock is set to true, the boat assigned must no longer be changed.
/// If fail is set to true, all other options have been used.
#[derive(Clone, Debug, Default)]
struct AllocResult {
    boat: String,
    /// Boat allocated must not be changed
    lock: bool,
    /// Allocation has failed
    fail: bool,
    /// Name does not have any boats to allocate
    absent: bool,
}

impl AllocResult {
    /// Pass in a boat name
    fn from_boat(boat: String) -> Self {
        Self {
            boat,
            lock: false,
            fail: false,
            absent: false,
        }
    }
    /// Create a alloc where no boat has been
    /// specified for a person
    fn from_absent() -> Self {
        Self {
            boat: String::new(),
            lock: false,
            fail: false,
            absent: true,
        }
    }

    /// Create an alloc where no boat can be assigned
    /// to a person without creating conflicts
    fn from_fail(boat: String) -> Self {
        Self {
            boat,
            lock: true,
            fail: true,
            absent: false,
        }
    }

    /// Create an alloc where the boat assigned is locked
    fn from_lock(boat: String) -> Self {
        Self {
            boat,
            lock: true,
            fail: false,
            absent: false,
        }
    }
}

impl NameList {
    /// Assign everyone their primary boats.
    /// If deconflict is set to true, perform deconflict.
    ///
    /// Returns false when operation fails.
    pub async fn assign_boats(&mut self, deconflict: bool) -> bool {
        let config = get_config_type(self.date);

        let allo_lock = BOAT_ALLOCATIONS[config as usize].read().await;

        let assigned: Vec<Option<String>> = self
            .names
            .iter()
            .map(|n| {
                if allo_lock.contains_key(n) {
                    let allo = allo_lock.get(n).unwrap();
                    if let Some(pri) = allo.0.as_deref() {
                        return Some(pri.to_owned());
                    }
                    if let Some(alt) = allo.1.as_deref() {
                        return Some(alt.to_owned());
                    }

                    None
                } else {
                    None
                }
            })
            .collect();

        self.boats = Some(assigned);

        let res = match deconflict {
            false => {
                return true;
            }
            true => {
                let potential_matches = Self::find_matching(&self.names, config).await;

                let mut lookup: HashMap<&str, Option<String>> = Default::default();

                let mut deconf_result: bool = true;
                for matches in potential_matches.iter() {
                    let (deconf_lookup, success) = Self::deconflict(matches, config).await;
                    if !success {
                        deconf_result = false;
                    }
                    lookup.extend(deconf_lookup);
                }

                debug_println!("deconf lookup: {:?}", lookup);

                for idx in 0..self.names.len() {
                    let name = &self.names[idx];
                    if lookup.contains_key(name.as_str()) {
                        let boat = lookup.get(name.as_str()).unwrap().clone();
                        match &mut self.boats {
                            Some(boatlist) => boatlist[idx] = boat,
                            None => (),
                        }
                    }
                }

                deconf_result
            }
        };
        match &mut self.boats {
            Some(boats) => {
                Self::mark_matching(boats);
            }
            None => (),
        }

        res
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

            for idx in 0..2 {
                debug_println!("repetition {} of 2", idx);
                for (name, used) in remaining_names.iter_mut() {
                    let (pri, alt) = {
                        match read_lock.get(*name) {
                            Some(boats) => {
                                let p = boats.0.as_deref();
                                let a = boats.1.as_deref();

                                (p, a)
                            }
                            None => (None, None),
                        }
                    };

                    // empty case
                    if names_set.len() == 0 {
                        *used = true;

                        group.push(name.to_owned());
                        names_set.insert(name);
                        if let Some(_p) = pri {
                            allo_set.insert(_p);
                        }
                        if let Some(_a) = alt {
                            allo_set.insert(_a);
                        }

                        continue;
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
                                // if !names_set.contains(name) {
                                group.push(name.to_owned());
                                // }
                            }
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

            debug_print!("names: {:?} ", names_set);
            debug_println!("conflicts: {:?}", allo_set);

            // at least 1 is in each group
            if group.len() > 1 {
                groups.push(group);
            }
        }

        // todo!();
        groups
    }

    /// Internal deconflict method.
    ///
    /// Pass in a list of names known to have a conflict and config,
    /// Returns a (hopefully) deconflicted list of boats and if the operation
    /// is successful.
    ///
    async fn deconflict(
        names: &Vec<String>,
        config: Config,
    ) -> (HashMap<&str, Option<String>>, bool) {
        // identify names with the same boat

        let read_lock = BOAT_ALLOCATIONS[config as usize].read().await;

        let allo_set: HashSet<&str> = names
            .iter()
            .map(|n| {
                let allo = read_lock.get(n).unwrap();
                let (pri, alt) = { (allo.0.as_deref(), allo.1.as_deref()) };
                match (pri, alt) {
                    (None, None) => vec![],
                    (None, Some(a)) => vec![a],
                    (Some(p), None) => vec![p],
                    (Some(p), Some(a)) => vec![p, a],
                }
            })
            .collect::<Vec<Vec<&str>>>()
            .concat()
            .into_iter()
            .collect();

        // actual allocated set of boats.
        // the bool marks if the allocation is fixed: true => fixed, false => not fixed
        let mut curr_allo: HashSet<&str> = Default::default();

        // initialize empty map
        // this map contains the lookup table of a name to a boat and some others.
        // all values must be the Option::Some variant for
        // deconflict to complete.
        let mut res: BTreeMap<&str, Option<AllocResult>> =
            names.iter().map(|n| (n.as_str(), None)).collect();

        // deconflict possible (sort of)
        if names.len() <= allo_set.len() {
            // break when all have been allocated
            while !res.iter().all(|(_, v)| v.is_some()) {
                let unallo_name = res
                    .iter()
                    .find_map(|(n, allo)| match allo {
                        Some(_) => None,
                        None => Some(n),
                    })
                    .cloned()
                    .unwrap();

                debug_println!("performing allocation for {}", unallo_name);

                let avail_opts = read_lock.get(unallo_name).unwrap();

                // first allocation
                if curr_allo.len() == 0 {
                    let x;
                    match (avail_opts.0.as_deref(), avail_opts.1.as_deref()) {
                        (None, None) => {
                            x = None;
                            res.insert(&unallo_name, Some(AllocResult::from_absent()));
                            // temp
                        }
                        (Some(pri), opt) => {
                            x = Some(pri);
                            curr_allo.insert(pri);
                            let mut allo = AllocResult::from_boat(pri.to_owned());

                            if let None = opt {
                                allo.lock = true;
                            }

                            res.insert(&unallo_name, Some(allo));
                        }
                        (None, Some(alt)) => {
                            x = Some(alt);
                            curr_allo.insert(alt);

                            let mut allo = AllocResult::from_boat(alt.to_owned());
                            allo.lock = true;
                            res.insert(&unallo_name, Some(allo));
                        }
                    }
                    debug_println!("allocating {} to: {:?}", unallo_name, x);
                    continue;
                }

                // deconf logic here
                match (avail_opts.0.as_deref(), avail_opts.1.as_deref()) {
                    // no boat allocation given in config
                    (None, None) => {
                        debug_println!("no allocaions for {}", unallo_name);
                        res.insert(&unallo_name, Some(AllocResult::from_absent()));
                    }
                    // primary boat only, dibs
                    (Some(pri), None) => {
                        debug_println!("primary boat available for {}", unallo_name);
                        if !curr_allo.contains(pri) {
                            curr_allo.insert(pri);
                            res.insert(&unallo_name, Some(AllocResult::from_lock(pri.to_owned())));
                        } else {
                            // find the offending boat and replace with this
                            let conflict = res
                                .iter()
                                .find_map(|(k, v)| match v {
                                    Some(allocation) => {
                                        if allocation.boat == pri {
                                            Some((k.to_string(), allocation.clone()))
                                        } else {
                                            None
                                        }
                                    }
                                    None => None,
                                })
                                .unwrap();

                            let other = conflict.1;
                            let mut allo = AllocResult::from_boat(pri.to_owned());

                            // check if the conflicting person has dibs
                            match other.lock {
                                true => {
                                    allo.fail = true;
                                    allo.lock = true;
                                }
                                false => {
                                    // kick the other guy out
                                    let other_allo = res.get_mut(conflict.0.as_str()).unwrap();
                                    *other_allo = None;
                                    allo.lock = true
                                }
                            }

                            res.insert(&unallo_name, Some(allo));
                        }
                    }
                    // alternate boat only
                    // this branch should not be taken
                    // pri boats should exist before alt
                    // people assigned only alt will be given the least priority
                    (None, Some(alt)) => {
                        debug_println!("alternate boat available for {}", unallo_name);
                        if !curr_allo.contains(alt) {
                            curr_allo.insert(alt);
                            res.insert(&unallo_name, Some(AllocResult::from_boat(alt.to_owned())));
                        } else {
                            res.insert(&unallo_name, Some(AllocResult::from_fail(alt.to_owned())));
                        }
                    }
                    // both options available
                    (Some(pri), Some(alt)) => {
                        debug_println!("both boats available for {}", unallo_name);
                        // check conflicts
                        match (curr_allo.contains(pri), curr_allo.contains(alt)) {
                            // need to kick people out
                            (true, true) => {
                                let pri_conflict = res
                                    .iter()
                                    .find_map(|(k, v)| match v {
                                        Some(allocation) => {
                                            if allocation.boat == pri {
                                                Some((k.to_string(), allocation.clone()))
                                            } else {
                                                None
                                            }
                                        }
                                        None => None,
                                    })
                                    .unwrap();

                                let alt_conflict = res
                                    .iter()
                                    .find_map(|(k, v)| match v {
                                        Some(allocation) => {
                                            if allocation.boat == alt {
                                                Some((k.to_string(), allocation.clone()))
                                            } else {
                                                None
                                            }
                                        }
                                        None => None,
                                    })
                                    .unwrap();

                                // which person do we kick out?
                                let kicked;
                                debug_println!("{} has conflicts: {:?}, {:?}", unallo_name, pri_conflict, alt_conflict);
                                match (pri_conflict.1.lock, alt_conflict.1.lock) {
                                    (true, true) => {
                                        debug_println!("no options left for {}", unallo_name);
                                        // all locked
                                        res.insert(
                                            unallo_name,
                                            Some(AllocResult::from_fail(pri.to_owned())),
                                        );

                                        kicked = None;
                                    }
                                    (true, false) => {
                                        debug_println!("locking {} to {}", unallo_name, alt);
                                        res.insert(
                                            unallo_name,
                                            Some(AllocResult::from_lock(alt.to_owned())),
                                        );

                                        kicked = Some(alt_conflict.0);
                                    }
                                    (false, true) => {
                                        debug_println!("locking {} to {}", unallo_name, pri);
                                        // curr_al
                                        res.insert(
                                            unallo_name,
                                            Some(AllocResult::from_lock(pri.to_owned())),
                                        );

                                        kicked = Some(pri_conflict.0);
                                    }
                                    (false, false) => {
                                        debug_println!("assigning {} to {} without locking", unallo_name, pri);
                                        // this branch is causing problems
                                        res.insert(
                                            unallo_name,
                                            Some(AllocResult::from_boat(pri.to_owned())),
                                        );

                                        kicked = Some(pri_conflict.0);
                                    }
                                }

                                if let Some(to_kick) = kicked {
                                    debug_println!(
                                        "kicking out {}, taking in {}",
                                        to_kick,
                                        unallo_name
                                    );
                                    let other_allo = res.get_mut(to_kick.as_str()).unwrap();
                                    *other_allo = None;
                                }
                            }

                            (true, false) => {
                                // res
                                curr_allo.insert(alt);
                                res.insert(
                                    unallo_name,
                                    Some(AllocResult::from_lock(alt.to_owned())),
                                );
                            }
                            (false, true) => {
                                curr_allo.insert(pri);
                                res.insert(
                                    unallo_name,
                                    Some(AllocResult::from_lock(pri.to_owned())),
                                );
                            }
                            (false, false) => {
                                curr_allo.insert(pri);
                                res.insert(
                                    unallo_name,
                                    Some(AllocResult::from_boat(pri.to_owned())),
                                );
                            }
                        }
                    }
                }

                debug_println!("curr iteration:\n{:#?}", res);
            }

            let successful = res.iter().all(|(_, v)| match v {
                Some(allo) => match (allo.absent, allo.fail) {
                    (false, false) => true,
                    _ => false,
                },
                None => false,
            });

            let actual_res = res
                .into_iter()
                .map(|(k, v)| {
                    let allo = match v {
                        Some(allocation) => {
                            if allocation.absent {
                                None
                            } else {
                                Some(allocation.boat)
                            }
                        }
                        None => None,
                    };

                    (k, allo)
                })
                .collect();

            return (actual_res, successful);
        } else {
            // deconflict defo not possible
            debug_println!(
                "deconflict definitely not possible: num names = {}, possible boats = {}",
                names.len(),
                allo_set.len()
            );
            let allocated: HashMap<&str, Option<String>> = names
                .iter()
                .map(|n| {
                    let (pri, alt) = read_lock.get(n).unwrap();
                    let allocated = match (pri, alt) {
                        (None, None) => None,
                        (Some(p), _) => Some(p.to_owned()),
                        (None, Some(a)) => Some(a.to_owned()),
                    };

                    (n.as_str(), allocated)
                })
                .collect();

            return (allocated, false);
        }
    }

    /// Mark matching elements in the boat list with an exclamation mark (!)
    fn mark_matching(boat_list: &mut Vec<Option<String>>) {
        const MARK: &str = "!";

        let mut boat_set: HashSet<String> = Default::default();
        let mut match_set: HashSet<String> = Default::default();

        for boat in boat_list.iter() {
            match boat {
                Some(b) => {
                    if boat_set.contains(b.as_str()) {
                        match_set.insert(b.clone());
                    } else {
                        boat_set.insert(b.clone());
                    }
                }
                None => (),
            }
        }

        for boat in boat_list.iter_mut() {
            match boat {
                Some(b) => {
                    if match_set.contains(b.as_str()) {
                        *b = format!("{}{}", MARK, b);
                    }
                }
                None => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{get_config_type, Config, NameList, BOAT_ALLOCATIONS};

    #[cfg(notset)]
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
    async fn test_find_matching_all() {
        crate::init().await;

        let config = Config::New;

        let read_lock: tokio::sync::RwLockReadGuard<
            '_,
            std::collections::HashMap<String, (Option<String>, Option<String>)>,
        > = BOAT_ALLOCATIONS[config as usize].read().await;
        let names: Vec<String> = read_lock.iter().map(|(k, _)| k.to_owned()).collect();

        let groups = NameList::find_matching(&names, config).await;

        // println!("boat allocations:\n{:?}\n", read_lock);
        println!("potential conflicting groups:\n{:?}\n", groups);

        println!("performing deconf");

        for group in groups.iter() {
            println!("deconflicting group: {:?}", group);
            let res = NameList::deconflict(group, config).await;
            println!("deconf result: {:?}", res);
        }
    }

    #[tokio::test]
    async fn test_find_matching_today() {
        crate::init().await;

        // let date = NaiveDate::from_ymd_opt(2023, 1, 14).unwrap();
        // let date = NaiveDate::from_ymd_opt(2023, 7, 13).unwrap();
        let date = chrono::Local::now().date_naive();
        let config = get_config_type(date);
        let mut name_list = crate::namelist(date, false).await.unwrap();
        let deconf_res = name_list.assign_boats(true).await;
        name_list.paddling().await.unwrap();
        let groups = NameList::find_matching(&name_list.names, config).await;

        println!("allocation success: {}", deconf_res);
        println!("potential conflicting groups: {:?}", groups);
        println!("deconf boat allocation: {}", name_list);

        name_list.assign_boats(false).await;
        name_list.paddling().await.unwrap();

        println!("no deconf boat allocation: {}", name_list);
    }
}
