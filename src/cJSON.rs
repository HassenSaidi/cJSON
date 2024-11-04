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

const CJSON_IS_REFERENCE: u32 = 256;
const CJSON_STRING_IS_CONST: u32 = 512;

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
pub fn cJSON_New_Item() -> Rc<RefCell<CJSON>> {
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

pub fn cjson_create_null() -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().type_ = CJSON_NULL;
    item
}

pub fn cjson_create_true() -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().type_ = CJSON_TRUE;
    item
}

pub fn cjson_create_false() -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().type_ = CJSON_FALSE;
    item
}

pub fn cjson_create_bool(boolean: bool) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().type_ = if boolean { CJSON_TRUE } else { CJSON_FALSE };
    item
}

pub fn cjson_create_number(num: f64) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.type_ = CJSON_NUMBER;
        item_mut.valuedouble = num;
        item_mut.valueint = num as i32; // cast to integer for backward compatibility
    }
    item
}

pub fn cjson_create_string_reference(string: &str) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.type_ = CJSON_STRING | CJSON_IS_REFERENCE;
        item_mut.valuestring = Some(string.to_string()); // Store reference flag without ownership
    }
    item
}

pub fn cjson_create_object_reference(child: Rc<RefCell<CJSON>>) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.type_ = CJSON_OBJECT | CJSON_IS_REFERENCE;
        item_mut.child = Some(child); // Reference to existing object
    }
    item
}

pub fn cjson_create_array_reference(child: Rc<RefCell<CJSON>>) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.type_ = CJSON_ARRAY | CJSON_IS_REFERENCE;
        item_mut.child = Some(child); // Reference to existing array
    }
    item
}

pub fn cjson_create_raw(raw: &str) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.type_ = CJSON_RAW;
        item_mut.valuestring = Some(raw.to_string()); // Store raw JSON string
    }
    item
}

pub fn cjson_create_array() -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().type_ = CJSON_ARRAY;
    item
}

pub fn cjson_create_object() -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().type_ = CJSON_OBJECT;
    item
}


/// Creates a `CJSON` instance representing a JSON string.
pub fn cjson_create_string(s: &str) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.type_ = CJSON_STRING;
        item_mut.valuestring = Some(s.to_string());
    }
    item
}

/// Creates a `CJSON` instance representing a JSON array of strings.
pub fn cJSON_CreateStringArray(strings: &[&str]) -> Option<Rc<RefCell<CJSON>>> {
    if strings.is_empty() {
        return None;
    }

    let array = cJSON_New_Item();
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
    fn test_cJSON_CreateStringArray() {
    let strings = ["Hello", "world", "Rust"];
    let array = cJSON_CreateStringArray(&strings).unwrap();

    // Check that the type is CJSON_ARRAY
    assert_eq!(array.borrow().type_, CJSON_ARRAY);
    
    // Check the first child
    let childv = array.borrow_mut().child.clone().expect("Array should have a child");
    assert_eq!(childv.borrow().type_, CJSON_STRING);
    assert_eq!(childv.borrow().valuestring, Some("Hello".to_string()));
    
    // Move to the next child
    let childv = childv.borrow_mut().next.clone().expect("First child should have a next");
    assert_eq!(childv.borrow().type_, CJSON_STRING);
    assert_eq!(childv.borrow().valuestring, Some("world".to_string()));
        
    // Move to the next child
    let childv = childv.borrow_mut().next.clone().expect("Second child should have a next");
    assert_eq!(childv.borrow().type_, CJSON_STRING);
    assert_eq!(childv.borrow().valuestring, Some("Rust".to_string()));

    // Ensure that there are no more children
    assert!(childv.borrow().next.is_none(), "There should be no more children");

    }
}
