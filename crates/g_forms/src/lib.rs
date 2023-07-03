/// Google form interface
pub struct GoogleForm {
    form_id: String,
}

impl GoogleForm {
    /// Link to a new form
    pub fn from_url<T: ToString>(url: T) -> GoogleForm {
        todo!()
    }

    /// Link to a new form by it's id
    pub fn from_id<T: ToString>(id: T) -> GoogleForm {
        GoogleForm {
            form_id: id.to_string(),
        }
    }
}

#[cfg(test)]
#[allow(unused)]
mod test {
    use super::*;
}
