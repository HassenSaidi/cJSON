use std::rc::Rc;
use std::cell::RefCell;

const CJSON_VERSION_MAJOR: u32 = 1;
const CJSON_VERSION_MINOR: u32 = 7;
const CJSON_VERSION_PATCH: u32 = 15;

pub fn cjson_version() -> String {
    format!("{}.{}.{}", CJSON_VERSION_MAJOR, CJSON_VERSION_MINOR, CJSON_VERSION_PATCH)
}

struct PrintBuffer<'a> {
    buffer: &'a mut String,
    length: usize,
    offset: usize,
    noalloc: bool,
    format: bool,
}

// cJSON Types
const CJSON_INVALID: u32 = 0;
const CJSON_FALSE: u32 = 1 << 0;
const CJSON_TRUE: u32 = 1 << 1;
const CJSON_NULL: u32 = 1 << 2;
const CJSON_NUMBER: u32 = 1 << 3;
const CJSON_STRING: u32 = 1 << 4;
const CJSON_ARRAY: u32 = 1 << 5;
const CJSON_OBJECT: u32 = 1 << 6;
const CJSON_RAW: u32 = 1 << 7; // Raw JSON

// cJSON Flags
const CJSON_IS_REFERENCE: u32 = 256;
const CJSON_STRING_IS_CONST: u32 = 512;


#[derive(Debug)]
pub struct CJSON {
    pub next: Option<Rc<RefCell<CJSON>>>,
    pub prev: Option<Rc<RefCell<CJSON>>>,
    pub child: Option<Rc<RefCell<CJSON>>>,
    pub item_type: u32,
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
        item_type: CJSON_NULL,
        valuestring: None,
        valueint: 0,
        valuedouble: 0.0,
        string: None,
    }))
}

pub fn cjson_create_null() -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().item_type = CJSON_NULL;
    item
}

pub fn cjson_create_true() -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().item_type = CJSON_TRUE;
    item
}

pub fn cjson_create_false() -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().item_type = CJSON_FALSE;
    item
}

pub fn cjson_create_bool(boolean: bool) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().item_type = if boolean { CJSON_TRUE } else { CJSON_FALSE };
    item
}

pub fn cjson_create_number(num: f64) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.item_type = CJSON_NUMBER;
        item_mut.valuedouble = num;
        item_mut.valueint = num as i32; // cast to integer for backward compatibility
    }
    item
}

pub fn cjson_create_string_reference(string: &str) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.item_type = CJSON_STRING | CJSON_IS_REFERENCE;
        item_mut.valuestring = Some(string.to_string()); // Store reference flag without ownership
    }
    item
}

pub fn cjson_create_object_reference(child: Rc<RefCell<CJSON>>) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.item_type = CJSON_OBJECT | CJSON_IS_REFERENCE;
        item_mut.child = Some(child); // Reference to existing object
    }
    item
}

pub fn cjson_create_array_reference(child: Rc<RefCell<CJSON>>) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.item_type = CJSON_ARRAY | CJSON_IS_REFERENCE;
        item_mut.child = Some(child); // Reference to existing array
    }
    item
}

pub fn cjson_create_raw(raw: &str) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.item_type = CJSON_RAW;
        item_mut.valuestring = Some(raw.to_string()); // Store raw JSON string
    }
    item
}

pub fn cjson_create_array() -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().item_type = CJSON_ARRAY;
    item
}

pub fn cjson_create_object() -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    item.borrow_mut().item_type = CJSON_OBJECT;
    item
}


/// Creates a `CJSON` instance representing a JSON string.
pub fn cjson_create_string(s: &str) -> Rc<RefCell<CJSON>> {
    let item = cJSON_New_Item();
    {
        let mut item_mut = item.borrow_mut();
        item_mut.item_type = CJSON_STRING;
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
    array.borrow_mut().item_type = CJSON_ARRAY;

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


pub fn cjson_create_int_array(numbers: &[i32]) -> Option<Rc<RefCell<CJSON>>> {
    if numbers.is_empty() {
        return None;
    }

    let array = cjson_create_array();
    let mut prev:Option<Rc<RefCell<CJSON>>>  = None;

    for &num in numbers {
        let number_item = cjson_create_number(num as f64);
        if prev.is_none() {
            // Set the first item as the child of the array
            array.borrow_mut().child = Some(Rc::clone(&number_item));
        } else {
            // Append to the previous item
            prev.as_ref().unwrap().borrow_mut().next = Some(Rc::clone(&number_item));
            number_item.borrow_mut().prev = Some(Rc::clone(prev.as_ref().unwrap()));
        }
        prev = Some(number_item);
    }

    // Link last and first elements if necessary
    if let Some(first_child) = &array.borrow().child {
        first_child.borrow_mut().prev = prev;
    }

    Some(array)
}

pub fn cjson_create_float_array(numbers: &[f32]) -> Option<Rc<RefCell<CJSON>>> {
    if numbers.is_empty() {
        return None;
    }

    let array = cjson_create_array();
    let mut prev:Option<Rc<RefCell<CJSON>>>  = None;

    for &num in numbers {
        let number_item = cjson_create_number(num as f64); // Convert f32 to f64 for storage
        if prev.is_none() {
            // Set the first item as the child of the array
            array.borrow_mut().child = Some(Rc::clone(&number_item));
        } else {
            // Append to the previous item
            prev.as_ref().unwrap().borrow_mut().next = Some(Rc::clone(&number_item));
            number_item.borrow_mut().prev = Some(Rc::clone(prev.as_ref().unwrap()));
        }
        prev = Some(number_item);
    }

    // Link last and first elements if necessary
    if let Some(first_child) = &array.borrow().child {
        first_child.borrow_mut().prev = prev;
    }

    Some(array)
}


pub fn cjson_create_double_array(numbers: &[f64]) -> Option<Rc<RefCell<CJSON>>> {
    if numbers.is_empty() {
        return None;
    }

    let array = cjson_create_array();
    let mut prev: Option<Rc<RefCell<CJSON>>> = None;

    for &num in numbers {
        let number_item = cjson_create_number(num);
        if prev.is_none() {
            // Set the first item as the child of the array
            array.borrow_mut().child = Some(Rc::clone(&number_item));
        } else {
            // Append to the previous item
            prev.as_ref().unwrap().borrow_mut().next = Some(Rc::clone(&number_item));
            number_item.borrow_mut().prev = Some(Rc::clone(prev.as_ref().unwrap()));
        }
        prev = Some(number_item);
    }

    // Link last and first elements if necessary
    if let Some(first_child) = &array.borrow().child {
        first_child.borrow_mut().prev = prev;
    }

    Some(array)
}

pub fn cjson_create_string_array(strings: &[&str]) -> Option<Rc<RefCell<CJSON>>> {
    if strings.is_empty() {
        return None;
    }

    let array = cjson_create_array();
    let mut prev: Option<Rc<RefCell<CJSON>>>  = None;

    for &string in strings {
        let string_item = cjson_create_string(string);
        if prev.is_none() {
            // Set the first item as the child of the array
            array.borrow_mut().child = Some(Rc::clone(&string_item));
        } else {
            // Append to the previous item
            prev.as_ref().unwrap().borrow_mut().next = Some(Rc::clone(&string_item));
            string_item.borrow_mut().prev = Some(Rc::clone(prev.as_ref().unwrap()));
        }
        prev = Some(string_item);
    }

    // Link last and first elements if necessary
    if let Some(first_child) = &array.borrow().child {
        first_child.borrow_mut().prev = prev;
    }

    Some(array)
}

pub fn cjson_get_array_size(array: &Rc<RefCell<CJSON>>) -> usize {
    let mut size = 0;
    let mut child = array.borrow().child.clone();

    while let Some(current) = child {
        size += 1;
        child = current.borrow().next.clone();
    }

    size
}

fn get_array_item(array: &Rc<RefCell<CJSON>>, index: usize) -> Option<Rc<RefCell<CJSON>>> {
    let mut current_child = array.borrow().child.clone();
    let mut current_index = index;

    while let Some(child) = current_child {
        if current_index == 0 {
            return Some(child);
        }
        current_index -= 1;
        current_child = child.borrow().next.clone();
    }

    None
}

pub fn cjson_get_array_item(array: &Rc<RefCell<CJSON>>, index: i32) -> Option<Rc<RefCell<CJSON>>> {
    if index < 0 {
        return None;
    }

    get_array_item(array, index as usize)
}

fn add_item_to_array(array: &Rc<RefCell<CJSON>>, item: Rc<RefCell<CJSON>>) -> bool {
    if Rc::ptr_eq(&array, &item) || array.borrow().item_type != CJSON_ARRAY {
        return false;
    }

    let mut array_mut = array.borrow_mut();
    let child = array_mut.child.clone();

    if child.is_none() {
        // List is empty, start a new one
        array_mut.child = Some(Rc::clone(&item));
        item.borrow_mut().prev = Some(Rc::clone(&item));
        item.borrow_mut().next = None;
    } else {
        // Append to the end of the list
        let last = child.as_ref().unwrap().borrow().prev.clone();
        if let Some(last_item) = last {
            last_item.borrow_mut().next = Some(Rc::clone(&item));
            item.borrow_mut().prev = Some(Rc::clone(&last_item));
            array_mut.child.as_ref().unwrap().borrow_mut().prev = Some(Rc::clone(&item));
        }
    }

    true
}

pub fn cjson_add_item_to_array(array: &Rc<RefCell<CJSON>>, item: Rc<RefCell<CJSON>>) -> bool {
    add_item_to_array(array, item)
}



fn add_item_to_object(
    object: &Rc<RefCell<CJSON>>,
    key: &str,
    item: Rc<RefCell<CJSON>>,
    constant_key: bool,
) -> bool {
    if Rc::ptr_eq(&object, &item) || key.is_empty() || object.borrow().item_type != CJSON_OBJECT {
        return false;
    }

    let new_key = if constant_key {
        key.to_string()
    } else {
        key.to_owned()
    };

    {
        let mut item_mut = item.borrow_mut();
        let new_type = if constant_key {
            item_mut.item_type | CJSON_STRING_IS_CONST
        } else {
            item_mut.item_type & !CJSON_STRING_IS_CONST
        };

        if (item_mut.item_type & CJSON_STRING_IS_CONST) == 0 {
            item_mut.string = None;
        }

        item_mut.string = Some(new_key);
        item_mut.item_type = new_type;
    }

    {
        let mut object_mut = object.borrow_mut();

        if object_mut.child.is_none() {
            object_mut.child = Some(Rc::clone(&item));
        } else {
            let mut last = Rc::clone(object_mut.child.as_ref().unwrap());
            loop {
                let next = last.borrow().next.clone();
                if let Some(next_child) = next {
                    last = next_child;
                } else {
                    break;
                }
            }
            last.borrow_mut().next = Some(Rc::clone(&item));
            item.borrow_mut().prev = Some(last);
        }
    }

    true
}



pub fn cjson_add_item_to_object(
    object: &Rc<RefCell<CJSON>>,
    key: &str,
    item: Rc<RefCell<CJSON>>,
) -> bool {
    add_item_to_object(object, key, item, false)
}

pub fn cjson_add_true_to_object(object: &Rc<RefCell<CJSON>>, name: &str) -> Option<Rc<RefCell<CJSON>>> {
    let true_item = cjson_create_true();
    if add_item_to_object(object, name, Rc::clone(&true_item), false) {
        Some(true_item)
    } else {
        cjson_delete(Some(true_item));
        None
    }
}

pub fn cjson_add_false_to_object(object: &Rc<RefCell<CJSON>>, name: &str) -> Option<Rc<RefCell<CJSON>>> {
    let false_item = cjson_create_false();
    if add_item_to_object(object, name, Rc::clone(&false_item), false) {
        Some(false_item)
    } else {
        cjson_delete(Some(false_item));
        None
    }
}

pub fn cjson_add_number_to_object(
    object: &Rc<RefCell<CJSON>>,
    name: &str,
    number: f64,
) -> Option<Rc<RefCell<CJSON>>> {
    let number_item = cjson_create_number(number);
    if add_item_to_object(object, name, Rc::clone(&number_item), false) {
        Some(number_item)
    } else {
        cjson_delete(Some(number_item));
        None
    }
}

pub fn cjson_add_string_to_object(
    object: &Rc<RefCell<CJSON>>,
    name: &str,
    string: &str,
) -> Option<Rc<RefCell<CJSON>>> {
    let string_item = cjson_create_string(string);
    if add_item_to_object(object, name, Rc::clone(&string_item), false) {
        Some(string_item)
    } else {
        cjson_delete(Some(string_item));
        None
    }
}

pub fn cjson_print(item: &Rc<RefCell<CJSON>>) -> Option<String> {
    let item_borrow = item.borrow();

    match item_borrow.item_type {
        CJSON_NULL => Some("null".to_string()),
        CJSON_TRUE => Some("true".to_string()),
        CJSON_FALSE => Some("false".to_string()),
        CJSON_NUMBER => Some(format!("{}", item_borrow.valuedouble)),
        //CJSON_STRING => item_borrow.valuestring.clone(),
        CJSON_STRING => Some(format!("\"{}\"", item_borrow.valuestring.as_deref().unwrap_or(""))),
        CJSON_ARRAY => {
            let mut result = String::from("[");
            let mut child = item_borrow.child.clone();
            while let Some(current) = child {
                if let Some(rendered) = cjson_print(&current) {
                    result.push_str(&rendered);
                    child = current.borrow().next.clone();
                    if child.is_some() {
                        result.push_str(", ");
                    }
                } else {
                    return None;
                }
            }
            result.push(']');
            Some(result)
        }
        CJSON_OBJECT => {
            let mut result = String::from("{");
            let mut child = item_borrow.child.clone();
            let mut first = true;
            while let Some(current) = child {
                let current_borrow = current.borrow();
                if let Some(key) = &current_borrow.string {
                    if let Some(rendered) = cjson_print(&current) {
                        if !first {
                            result.push_str(", ");
                        }
                        result.push_str(&format!("\"{}\": {}", key, rendered));
                        first = false;
                    }
                }
                child = current_borrow.next.clone();
            }
        result.push('}');
        Some(result)
        }
        _ => None,
    }
}

pub fn cjson_print_preallocated(
    item: &Rc<RefCell<CJSON>>,
    buffer: &mut String,
    length: usize,
    format: bool,
) -> bool {
    if length == 0 || buffer.is_empty() {
        return false;
    }

    // Ensure the buffer capacity matches the specified length
    if buffer.capacity() < length {
        buffer.reserve(length - buffer.capacity());
    }

    let mut p = PrintBuffer {
        buffer,
        length,
        offset: 0,
        noalloc: true,
        format,
    };

    print_value(item, &mut p)
}

fn ensure_capacity(output_buffer: &mut PrintBuffer, required: usize) -> bool {
    if output_buffer.buffer.capacity() < output_buffer.offset + required {
        output_buffer.buffer.reserve(required);
    }
    true
}

fn print_value(item: &Rc<RefCell<CJSON>>, output_buffer: &mut PrintBuffer) -> bool {
    if item.borrow().item_type == CJSON_INVALID || output_buffer.buffer.is_empty() {
        return false;
    }

    match item.borrow().item_type & 0xFF {
        CJSON_NULL => {
            if ensure_capacity(output_buffer, 5) {
                output_buffer.buffer.push_str("null");
                true
            } else {
                false
            }
        }
        CJSON_FALSE => {
            if ensure_capacity(output_buffer, 6) {
                output_buffer.buffer.push_str("false");
                true
            } else {
                false
            }
        }
        CJSON_TRUE => {
            if ensure_capacity(output_buffer, 5) {
                output_buffer.buffer.push_str("true");
                true
            } else {
                false
            }
        }
        CJSON_NUMBER => print_number(item, output_buffer),
        CJSON_RAW => {
            let item_borrow = item.borrow();
            if let Some(raw_string) = &item_borrow.valuestring {
                let raw_length = raw_string.len();
                if ensure_capacity(output_buffer, raw_length) {
                    output_buffer.buffer.push_str(raw_string);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        CJSON_STRING => print_string(item, output_buffer),
        CJSON_ARRAY => print_array(item, output_buffer),
        CJSON_OBJECT => print_object(item, output_buffer),
        _ => false,
    }
}


pub fn cjson_delete(item: Option<Rc<RefCell<CJSON>>>) {
    let mut current = item;

    while let Some(node) = current {
        let mut node_mut = node.borrow_mut();

        // Save the next pointer before we drop the current node
        let next = node_mut.next.clone();

        // Recursively delete child if it's not a reference
        if (node_mut.item_type & CJSON_IS_REFERENCE) == 0 {
            if let Some(child) = node_mut.child.take() {
                cjson_delete(Some(child));
            }
        }

        // Clear the valuestring if it's not a reference
        if (node_mut.item_type & CJSON_IS_REFERENCE) == 0 {
            node_mut.valuestring = None;
        }

        // Clear the string if it's not marked as const
        if (node_mut.item_type & CJSON_STRING_IS_CONST) == 0 {
            node_mut.string = None;
        }

        // Move to the next item in the list
        current = next;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cJSON_CreateStringArray() {
    let strings = ["Hello", "world", "Rust"];
    let array = cjson_create_string_array(&strings).unwrap();

    // Check that the type is CJSON_ARRAY
    assert_eq!(array.borrow().item_type, CJSON_ARRAY);
    
    // Check the first child
    let childv = array.borrow_mut().child.clone().expect("Array should have a child");
    assert_eq!(childv.borrow().item_type, CJSON_STRING);
    assert_eq!(childv.borrow().valuestring, Some("Hello".to_string()));
    
    // Move to the next child
    let childv = childv.borrow_mut().next.clone().expect("First child should have a next");
    assert_eq!(childv.borrow().item_type, CJSON_STRING);
    assert_eq!(childv.borrow().valuestring, Some("world".to_string()));
        
    // Move to the next child
    let childv = childv.borrow_mut().next.clone().expect("Second child should have a next");
    assert_eq!(childv.borrow().item_type, CJSON_STRING);
    assert_eq!(childv.borrow().valuestring, Some("Rust".to_string()));

    // Ensure that there are no more children
    assert!(childv.borrow().next.is_none(), "There should be no more children");

    }

    #[test]
    fn test_create_string_array_and_get_size() {
        let strings = ["Hello", "world", "Rust"];
        let array = cjson_create_string_array(&strings).unwrap();

        // Check that the type is CJSON_ARRAY
        assert_eq!(array.borrow().item_type, CJSON_ARRAY);

        // Check the size of the array
        let size = cjson_get_array_size(&array);
        assert_eq!(size, (strings.len() as i32).try_into().unwrap());
    }

    #[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn test_print_null() {
        let item = cjson_create_null();
        assert_eq!(cjson_print(&item), Some("null".to_string()));
    }

    #[test]
    fn test_print_true() {
        let item = cjson_create_true();
        assert_eq!(cjson_print(&item), Some("true".to_string()));
    }

    #[test]
    fn test_print_false() {
        let item = cjson_create_false();
        assert_eq!(cjson_print(&item), Some("false".to_string()));
    }

    #[test]
    fn test_print_number() {
        let item = cjson_create_number(42.0);
        assert_eq!(cjson_print(&item), Some("42".to_string()));
    }

    #[test]
    fn test_print_string() {
        let item = cjson_create_string("Hello, world!");
        assert_eq!(cjson_print(&item), Some("\"Hello, world!\"".to_string()));
    }

    #[test]
    fn test_print_array() {
        let array = cjson_create_array();
        cjson_add_item_to_array(&array, cjson_create_number(1.0));
        cjson_add_item_to_array(&array, cjson_create_number(2.0));
        cjson_add_item_to_array(&array, cjson_create_number(3.0));
        assert_eq!(cjson_print(&array), Some("[1, 2, 3]".to_string()));
    }

    #[test]
    fn test_print_object() {
        let object = cjson_create_object();
        cjson_add_string_to_object(&object, "name", "John");
        cjson_add_number_to_object(&object, "age", 30.0);
        cjson_add_true_to_object(&object, "is_student");
        assert_eq!(
            cjson_print(&object),
            Some("{\"name\": \"John\", \"age\": 30, \"is_student\": true}".to_string())
        );
    }

    #[test]
    fn test_print_nested_structure() {
        let object = cjson_create_object();
        let nested_array = cjson_create_array();
        cjson_add_item_to_array(&nested_array, cjson_create_string("nested"));
        cjson_add_item_to_array(&nested_array, cjson_create_number(99.0));

        cjson_add_string_to_object(&object, "title", "Example");
        cjson_add_item_to_object(&object, "details", nested_array);

        assert_eq!(
            cjson_print(&object),
            Some("{\"title\": \"Example\", \"details\": [\"nested\", 99]}".to_string())
        );
    }
}

}
