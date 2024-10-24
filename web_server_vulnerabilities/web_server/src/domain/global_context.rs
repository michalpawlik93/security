use std::sync::RwLock;

use once_cell::sync::Lazy;

use super::error_list::ErrorList;

pub struct GlobalContext;

static GLOBAL_ERROR_LIST: Lazy<RwLock<ErrorList>> = Lazy::new(|| RwLock::new(ErrorList::new()));

impl GlobalContext {
    pub fn add_global_error(id: i32, error: String) {
        let mut error_list = GLOBAL_ERROR_LIST.write().unwrap();
        error_list.add_error(id, error);
    }
    pub fn get_global_errors(id: i32) -> Option<Vec<String>> {
        let error_list = GLOBAL_ERROR_LIST.read().unwrap();
        error_list.get_errors(id).cloned()
    }
}

#[cfg(test)]
mod global_tests {
    use super::GlobalContext;
    use super::GLOBAL_ERROR_LIST;

    fn clear_global_error_list() {
        let mut error_list = GLOBAL_ERROR_LIST.write().unwrap();
        *error_list = super::ErrorList::new();
    }

    #[test]
    fn test_add_and_get_global_errors() {
        clear_global_error_list();

        GlobalContext::add_global_error(1, "First global error".to_string());
        GlobalContext::add_global_error(1, "Second global error".to_string());

        let errors = GlobalContext::get_global_errors(1).unwrap();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0], "First global error");
        assert_eq!(errors[1], "Second global error");
    }

    #[test]
    fn test_get_empty_global_error_list() {
        clear_global_error_list();

        assert!(GlobalContext::get_global_errors(1).is_none());
    }
}
