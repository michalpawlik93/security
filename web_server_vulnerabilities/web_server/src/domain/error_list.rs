use std::collections::HashMap;

pub struct ErrorList {
    errors: HashMap<i32, Vec<String>>,
}
impl ErrorList {
    pub fn new() -> Self {
        ErrorList {
            errors: HashMap::new(),
        }
    }
    pub fn add_error(&mut self, id: i32, error: String) {
        self.errors.entry(id).or_insert_with(Vec::new).push(error);
    }

    pub fn get_errors(&self, id: i32) -> Option<&Vec<String>> {
        self.errors.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorList;

    #[test]
    fn test_add_and_get_errors() {
        let mut error_list = ErrorList::new();

        error_list.add_error(1, "First error".to_string());
        error_list.add_error(1, "Second error".to_string());

        let errors = error_list.get_errors(1).unwrap();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0], "First error");
        assert_eq!(errors[1], "Second error");

        assert!(error_list.get_errors(2).is_none());
    }

    #[test]
    fn test_get_empty_error_list() {
        let error_list = ErrorList::new();

        assert!(error_list.get_errors(1).is_none());
    }
}
