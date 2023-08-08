//! Logsheet logic goes here
//!

#[cfg(test)]
mod tests {

    use chrono::NaiveTime;
    use g_forms::form::{Number, QuestionType};
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
}
