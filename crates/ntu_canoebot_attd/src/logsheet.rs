//! Logsheet logic goes here
//!

use std::collections::HashMap;

use chrono::{Duration, NaiveDate, NaiveTime};
use g_forms::form::Response;
use lazy_static::lazy_static;
use ntu_canoebot_util::debug_println;
use tokio::sync::RwLock;

use ntu_canoebot_config as config;

use crate::{get_config_type, NAMES_CERTS};

lazy_static! {
    /// Logsheet lock. Prevents multiple submissions. Keeps track of
    /// each session's most recent logsheet submissions.
    ///
    /// Element 0 is for AM sessions,
    /// Element 1 is for PM sessions.
    pub static ref SUBMIT_LOCK: RwLock<(NaiveDate, NaiveDate)> = {
        let yesterday = chrono::Local::now().date_naive() - Duration::days(1);

        RwLock::new((yesterday, yesterday))
    };

    static ref LOOPING_COUNTER: RwLock<LoopingCounter> = {
        let particulars: &HashMap<&'static str, String> = &*config::FORMFILLER_PARTICULARS;

        RwLock::new(LoopingCounter::from_size(particulars.len()))
    };
}

/// Looping counter for particulars
/// Iterates infinitely
struct LoopingCounter {
    size: i64,
    curr: usize,
}

impl Iterator for LoopingCounter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr;

        if (self.curr as i64) < self.size {
            self.curr += 1
        } else {
            self.curr = 0
        }

        Some(curr)
    }
}

impl LoopingCounter {
    /// Creates a looping counter with a given size.
    ///
    /// The counter starts from 0.
    pub fn from_size(size: usize) -> Self {
        Self {
            size: size as i64 - 1,
            curr: 0,
        }
    }
}

/// Sends a logsheet for a date and time.
#[rustfmt::skip]
pub async fn send(date: NaiveDate, session: bool) -> Result<Response, ()> {

    let logsheet_id = *ntu_canoebot_config::FORMFILLER_FORM_ID;

    let mut form = g_forms::GoogleForm::from_id(logsheet_id).await.unwrap();

    let name_list = crate::namelist(date, session).await.unwrap();
    let total_paddlers = name_list.names.len();

    let config = get_config_type(date);
    let cert_lock = NAMES_CERTS[config as usize].read().await;

    let certified: usize = name_list.names.iter().map(|name| {
        if cert_lock.contains_key(name) {
            match cert_lock.get(name).unwrap() {
                true => 1,
                false => 0,
            }
        } else {
            0
        }
    }).sum();

    let not_certified = total_paddlers - certified;

    debug_println!("total: {}\ncertified: {}\nnon-certified: {}", total_paddlers, certified, not_certified);
    debug_println!("namelist struct: {:?}", name_list);

    let particulars: &HashMap<&'static str, String> = &*config::FORMFILLER_PARTICULARS;
    let part_idx = LOOPING_COUNTER.write().await.next().unwrap();
    let (exco_name, exco_number) = particulars.iter().skip(part_idx).next().unwrap();


    let start_time = {
        let time = match session {
            true => *config::FORMFILLER_TIMES_PM_START,
            false => *config::FORMFILLER_TIMES_AM_START,
        };
        let time = time.time.unwrap();
        NaiveTime::from_hms_opt(time.hour.into(), time.minute.into(), time.second.into()).unwrap()
    };
    let end_time = {
        let time = match session {
            true => *config::FORMFILLER_TIMES_PM_END,
            false => *config::FORMFILLER_TIMES_AM_END,
        };
        let time = time.time.unwrap();
        NaiveTime::from_hms_opt(time.hour.into(), time.minute.into(), time.second.into()).unwrap()
    };

    // TODO: extract out all consts under this comment to the config file.
    form.question(0).unwrap().fill_str(&exco_name).unwrap(); // name
    form.question(1).unwrap().fill_str(&exco_number).unwrap(); // hp number
    form.question(2).unwrap().fill_str("Nanyang Technological University").unwrap(); // organization
    form.question(3).unwrap().fill_option(1).unwrap(); // type of activity
    form.question(4).unwrap().fill_number(certified.into()).unwrap(); // number of certified
    form.question(5).unwrap().fill_number(not_certified.into()).unwrap(); // number of non certified
    form.question(6).unwrap().fill_option(1).unwrap(); // paddling location
    form.question(7).unwrap().fill_date(date.and_time(chrono::Local::now().time())).unwrap(); // date of training
    form.question(8).unwrap().fill_time(start_time).unwrap(); // start time
    form.question(9).unwrap().fill_time(end_time).unwrap(); // end time
    form.question(10).unwrap().fill_option(0).unwrap(); // disclaimer agree


    debug_println!("form response: {:#?}", form);


    form.submit().await
    // Ok(Default::default())
    // Err(())
}

#[cfg(test)]
mod tests {

    use chrono::NaiveTime;
    use g_forms::form::{Number, QuestionType};

    use super::LoopingCounter;
    // use ntu_canoebot_config as config;

    /// Test if g_forms can deserialize form data
    #[tokio::test]
    async fn test_logsheet_valid() {
        let logsheet_id = *ntu_canoebot_config::FORMFILLER_FORM_ID;

        let mut form = g_forms::GoogleForm::from_id(logsheet_id).await.unwrap();

        // println!("{:#?}", form);

        for qn in form.iter() {
            println!("{:#?}", qn.question_type)
        }

        // this is the long way of filling up a question
        if let QuestionType::ShortAnswer(q) = &mut form.get_mut(0).unwrap().question_type {
            q.fill_str("osas").unwrap()
        } else {
            panic!();
        };
        if let QuestionType::ShortAnswer(q) = &mut form.get_mut(1).unwrap().question_type {
            q.fill_str("912345678").unwrap()
        } else {
            panic!();
        };
        if let QuestionType::ShortAnswer(q) = &mut form.get_mut(2).unwrap().question_type {
            q.fill_str("NTU").unwrap()
        } else {
            panic!();
        };
        if let QuestionType::MultipleChoice(q) = &mut form.get_mut(3).unwrap().question_type {
            q.fill_option(1).unwrap();
        } else {
            panic!();
        };
        if let QuestionType::ShortAnswer(q) = &mut form.get_mut(4).unwrap().question_type {
            q.fill_number(Number::from(10)).unwrap()
        } else {
            panic!();
        };
        if let QuestionType::ShortAnswer(q) = &mut form.get_mut(5).unwrap().question_type {
            q.fill_number(Number::from(1)).unwrap()
        } else {
            panic!();
        };

        if let QuestionType::MultipleChoice(q) = &mut form.get_mut(6).unwrap().question_type {
            q.fill_option(1).unwrap()
        } else {
            panic!();
        };
        if let QuestionType::Date(q) = &mut form.get_mut(7).unwrap().question_type {
            q.fill_date(chrono::Local::now().naive_local()).unwrap()
        } else {
            panic!();
        };
        if let QuestionType::Time(q) = &mut form.get_mut(8).unwrap().question_type {
            // q.fill_number(Number::from(10)).unwrap()
            q.fill_time(NaiveTime::from_hms_opt(7, 0, 0).unwrap())
                .unwrap()
        } else {
            panic!();
        };
        if let QuestionType::Time(q) = &mut form.get_mut(9).unwrap().question_type {
            // q.fill_number(Number::from(10)).unwrap()
            q.fill_time(NaiveTime::from_hms_opt(10, 0, 0).unwrap())
                .unwrap()
        } else {
            panic!();
        };
        if let QuestionType::CheckBox(q) = &mut form.get_mut(10).unwrap().question_type {
            // q.fill_number(Number::from(10)).unwrap()
            q.fill_option(0).unwrap();
        } else {
            panic!();
        };

        // for qn in form.iter() {
        //     let qn_id = format!("entry.{}", qn.id);
        //     let qn_resp = qn.response().unwrap();

        //     form.response.insert(qn_id, qn_resp).unwrap();
        // }

        form.generate_map();

        println!("{:#?}", form);
    }

    /// Temp, remove after testing!
    #[cfg(notset)]
    #[tokio::test]
    async fn test_logsheet() {
        crate::init().await;

        let res = super::send(chrono::Local::now().date_naive(), false).await;

        println!("{:#?}", res);
    }

    #[test]
    fn test_looping_counter() {
        let mut counter = LoopingCounter::from_size(5);

        assert_eq!(counter.next(), Some(0));
        assert_eq!(counter.next(), Some(1));
        assert_eq!(counter.next(), Some(2));
        assert_eq!(counter.next(), Some(3));
        assert_eq!(counter.next(), Some(4));
        assert_eq!(counter.next(), Some(0));
        assert_eq!(counter.next(), Some(1));

        let mut counter = LoopingCounter::from_size(1);
        assert_eq!(counter.next(), Some(0));
        assert_eq!(counter.next(), Some(0));
        assert_eq!(counter.next(), Some(0));
        assert_eq!(counter.next(), Some(0));
    }
}
