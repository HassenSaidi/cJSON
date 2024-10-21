use std::rc::Rc;
use std::cell::RefCell;

pub const CJSON_NULL: i32 = 0;
pub const CJSON_FALSE: i32 = 1;
pub const CJSON_TRUE: i32 = 2;
pub const CJSON_NUMBER: i32 = 3;
pub const CJSON_STRING: i32 = 4;
pub const CJSON_ARRAY: i32 = 5;
pub const CJSON_OBJECT: i32 = 6;
pub const CJSON_RAW: i32 = 7;

#[derive(Debug)]
pub struct CJSON {
    pub next: Option<Rc<RefCell<CJSON>>>,
    pub prev: Option<Rc<RefCell<CJSON>>>,
    pub child: Option<Rc<RefCell<CJSON>>>,
    pub type_: i32,
    pub valuestring: Option<String>,
    pub valueint: i32,
    pub valuedouble: f64,
    pub string: Option<String>,
}

/// Initializes a new `CJSON` instance with default values.
pub fn cjson_new() -> Rc<RefCell<CJSON>> {
    Rc::new(RefCell::new(CJSON {
        next: None,
        prev: None,
        child: None,
        type_: CJSON_NULL,
        valuestring: None,
        valueint: 0,
        valuedouble: 0.0,
        string: None,
    }))
}

/// Creates a `CJSON` instance representing a JSON string.
pub fn cjson_create_string(s: &str) -> Rc<RefCell<CJSON>> {
    let item = cjson_new();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.type_ = CJSON_STRING;
        item_mut.valuestring = Some(s.to_string());
    }
    item
}

/// Creates a `CJSON` instance representing a JSON array of strings.
pub fn cjson_create_string_array(strings: &[&str]) -> Option<Rc<RefCell<CJSON>>> {
    if strings.is_empty() {
        return None;
    }

    let array = cjson_new();
    array.borrow_mut().type_ = CJSON_ARRAY;

    let mut prev_node: Option<Rc<RefCell<CJSON>>> = None;
    let mut first_child: Option<Rc<RefCell<CJSON>>> = None;

    for (i, &s) in strings.iter().enumerate() {
        let string_cjson = cjson_create_string(s);

        // Set the prev and next pointers
        if let Some(ref prev) = prev_node {
            prev.borrow_mut().next = Some(Rc::clone(&string_cjson));
            string_cjson.borrow_mut().prev = Some(Rc::clone(prev));
        }

        // Set the first child
        if i == 0 {
            first_child = Some(Rc::clone(&string_cjson));
        }

        prev_node = Some(Rc::clone(&string_cjson));
    }

    // Set the array's child to the first node
    array.borrow_mut().child = first_child;

    // Set the prev pointer of the first child to the last node (as in the original C code)
    if let (Some(ref child), Some(ref prev)) = (&array.borrow().child, &prev_node) {
        child.borrow_mut().prev = Some(Rc::clone(prev));
    }

    Some(array)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cjson_create_string_array() {
    let strings = ["Hello", "world", "Rust"];
    let array = cjson_create_string_array(&strings).unwrap();

    // Check that the type is CJSON_ARRAY
    assert_eq!(array.borrow().type_, CJSON_ARRAY);

    // Check the first child
    let child = array.borrow_mut().child.clone().expect("Array should have a child");
    assert_eq!(child.borrow().type_, CJSON_STRING);
    assert_eq!(child.borrow().valuestring, Some("Hello".to_string()));

    // Move to the next child
    child = child.borrow_mut().next.clone().expect("First child should have a next");
    assert_eq!(child.borrow().type_, CJSON_STRING);
    assert_eq!(child.borrow().valuestring, Some("world".to_string()));

    // Move to the next child
    child = child.borrow_mut().next.clone().expect("Second child should have a next");
    assert_eq!(child.borrow().type_, CJSON_STRING);
    assert_eq!(child.borrow().valuestring, Some("Rust".to_string()));

    // Ensure that there are no more children
    assert!(child.borrow().next.is_none(), "There should be no more children");
    }
}
