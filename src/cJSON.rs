use std::rc::Rc;
use std::cell::RefCell;
use std::f64;
use std::i32;
use std::str::FromStr;
use std::sync::Mutex;
use lazy_static::lazy_static;


// Error handling 
#[derive(Default, Debug)]
pub struct Error {
    pub json: Option<Vec<u8>>, // Use `Option<Vec<u8>>` to represent a nullable byte slice
    pub position: usize,
}

lazy_static! {
    // Define a global mutable error state using `Mutex` for thread safety
    static ref GLOBAL_ERROR: Mutex<Error> = Mutex::new(Error::default());
}

pub fn cjson_get_error_ptr() -> Option<String> {
    let error = GLOBAL_ERROR.lock().unwrap();

    if let Some(ref json) = error.json {
        if error.position < json.len() {
            // Return an owned `String` instead of a reference
            return String::from_utf8(json[error.position..].to_vec()).ok();
        }
    }

    None
}
/*
pub fn cjson_get_error_ptr() -> Option<&'static str> {
    let error = GLOBAL_ERROR.lock().unwrap();

    if let Some(ref json) = error.json {
        // Calculate the error position and return a slice from the JSON input
        if error.position < json.len() {
            return std::str::from_utf8(&json[error.position..]).ok();
        }
    }

    None
}
*/
fn reset_global_error() {
    let mut error = GLOBAL_ERROR.lock().unwrap();
    error.json = None;
    error.position = 0;
}

fn set_global_error(value: &[u8], position: usize) {
    let mut error = GLOBAL_ERROR.lock().unwrap();
    error.json = Some(value.to_vec());
    error.position = position;
}

// End Error handling 

const CJSON_VERSION_MAJOR: u32 = 1;
const CJSON_VERSION_MINOR: u32 = 7;
const CJSON_VERSION_PATCH: u32 = 15;

const CJSON_NESTING_LIMIT: usize = 1000;

pub fn cjson_version() -> String {
    format!("{}.{}.{}", CJSON_VERSION_MAJOR, CJSON_VERSION_MINOR, CJSON_VERSION_PATCH)
}

pub struct ParseBuffer {
    pub content: Vec<u8>, // The input JSON content as bytes
    pub offset: usize,    // Current parsing offset
    pub depth: usize,
    pub length: usize,
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
    // Check for invalid length or an empty buffer
    if length == 0 || buffer.capacity() < length {
        return false;
    }

    // Initialize the print buffer
    let mut p = PrintBuffer {
        buffer,
        length,
        offset: 0,
        noalloc: true,
        format,
    };

    // Attempt to print the value into the buffer
    print_value(item, &mut p)
}

/*
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
*/
    
fn ensure_capacity(output_buffer: &mut PrintBuffer, required: usize) -> bool {
    let current_capacity = output_buffer.buffer.capacity();
    let needed_capacity = output_buffer.offset + required;

    // If the current capacity is less than needed, reserve more space
    if current_capacity < needed_capacity {
        output_buffer.buffer.reserve(needed_capacity - current_capacity);
        println!(
            "Reserving capacity: current = {}, needed = {}, new capacity = {}",
            current_capacity,
            needed_capacity,
            output_buffer.buffer.capacity()
        );
    }

    true
}


fn print_array(item: &Rc<RefCell<CJSON>>, output_buffer: &mut PrintBuffer) -> bool {
    let item_borrow = item.borrow();

    // Start the array with an opening bracket
    if !ensure_capacity(output_buffer, 1) {
        return false;
    }
    output_buffer.buffer.push('[');

    // Traverse the array elements
    let mut child = item_borrow.child.clone();
    let mut first = true;

    while let Some(current) = child {
        // Add a comma separator if this is not the first element
        if !first {
            if !ensure_capacity(output_buffer, 2) {
                return false;
            }
            output_buffer.buffer.push_str(", ");
        }

        // Print the current element
        if !print_value(&current, output_buffer) {
            return false;
        }

        first = false;
        // Move to the next element in the array
        child = current.borrow().next.clone();
    }

    // Close the array with a closing bracket
    if !ensure_capacity(output_buffer, 1) {
        return false;
    }
    output_buffer.buffer.push(']');

    true
}


fn print_number(item: &Rc<RefCell<CJSON>>, output_buffer: &mut PrintBuffer) -> bool {
    let item_borrow = item.borrow();
    let number = item_borrow.valuedouble;

    // Determine if the number is an integer or a floating-point value
    let output = if number.fract() == 0.0 {
        // Print as an integer if there is no fractional part
        format!("{}", number as i64)
    } else {
        // Print as a floating-point number
        format!("{:.17}", number)
    };

    // Ensure there is enough capacity in the buffer
    if ensure_capacity(output_buffer, output.len()) {
        output_buffer.buffer.push_str(&output);
        true
    } else {
        false
    }
}

fn print_string_ptr(input: &str, output_buffer: &mut PrintBuffer) -> bool {
    // Calculate the required length for the escaped string, including surrounding quotes
    let mut escaped_string = String::with_capacity(input.len() + 2);
    escaped_string.push('"');

    for c in input.chars() {
        match c {
            '"' => escaped_string.push_str("\\\""),
            '\\' => escaped_string.push_str("\\\\"),
          //  '\b' => escaped_string.push_str("\\b"),
           // '\f' => escaped_string.push_str("\\f"),
            '\n' => escaped_string.push_str("\\n"),
            '\r' => escaped_string.push_str("\\r"),
            '\t' => escaped_string.push_str("\\t"),
            // Escape non-printable ASCII characters
            c if c.is_control() => escaped_string.push_str(&format!("\\u{:04x}", c as u32)),
            // Regular character
            _ => escaped_string.push(c),
        }
    }

    escaped_string.push('"');

    // Ensure capacity in the output buffer and append the escaped string
    if ensure_capacity(output_buffer, escaped_string.len()) {
        output_buffer.buffer.push_str(&escaped_string);
        true
    } else {
        false
    }
}

fn print_object(item: &Rc<RefCell<CJSON>>, output_buffer: &mut PrintBuffer) -> bool {
    let item_borrow = item.borrow();

    // Start the object with an opening brace
    if !ensure_capacity(output_buffer, 1) {
        return false;
    }
    output_buffer.buffer.push('{');

    // Traverse the child list
    let mut child = item_borrow.child.clone();
    let mut first = true;

    while let Some(current) = child {
        let current_borrow = current.borrow();

        // Ensure that the current item has a string key
        if let Some(key) = &current_borrow.string {
            // Add a comma separator if this is not the first item
            if !first {
                if !ensure_capacity(output_buffer, 2) {
                    return false;
                }
                output_buffer.buffer.push_str(", ");
            }

            // Print the key as a string
            if !print_string_ptr(key, output_buffer) {
                return false;
            }

            // Add the key-value separator
            if !ensure_capacity(output_buffer, 2) {
                return false;
            }
            output_buffer.buffer.push_str(": ");

            // Print the value of the current item
            if !print_value(&current, output_buffer) {
                return false;
            }

            first = false;
        }

        // Move to the next item in the list
        child = current_borrow.next.clone();
    }

    // Close the object with a closing brace
    if !ensure_capacity(output_buffer, 1) {
        return false;
    }
    output_buffer.buffer.push('}');

    true
}

fn print_string(item: &Rc<RefCell<CJSON>>, output_buffer: &mut PrintBuffer) -> bool {
    let item_borrow = item.borrow();

    // Check if the valuestring is present
    if let Some(valuestring) = &item_borrow.valuestring {
        print_string_ptr(valuestring, output_buffer)
    } else {
        false
    }
}

fn print_value(item: &Rc<RefCell<CJSON>>, output_buffer: &mut PrintBuffer) -> bool {
    let item_borrow = item.borrow();

    match item_borrow.item_type & 0xFF {
        CJSON_NULL => {
            if ensure_capacity(output_buffer, 5) {
                output_buffer.buffer.push_str("null");
                println!("Added 'null' to buffer");
                true
            } else {
                false
            }
        }
        CJSON_FALSE => {
            if ensure_capacity(output_buffer, 6) {
                output_buffer.buffer.push_str("false");
                println!("Added 'false' to buffer");
                true
            } else {
                false
            }
        }
        CJSON_TRUE => {
            if ensure_capacity(output_buffer, 5) {
                output_buffer.buffer.push_str("true");
                println!("Added 'true' to buffer");
                true
            } else {
                false
            }
        }
        CJSON_NUMBER => {
            let number = item_borrow.valuedouble;
            let formatted_number = format!("{}", number);
            if ensure_capacity(output_buffer, formatted_number.len()) {
                output_buffer.buffer.push_str(&formatted_number);
                println!("Added number '{}' to buffer", formatted_number);
                true
            } else {
                false
            }
        }
        CJSON_STRING => {
            if let Some(valuestring) = &item_borrow.valuestring {
                if ensure_capacity(output_buffer, valuestring.len() + 2) {
                    output_buffer.buffer.push('"');
                    output_buffer.buffer.push_str(valuestring);
                    output_buffer.buffer.push('"');
                    println!("Added string '{}' to buffer", valuestring);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        CJSON_ARRAY => {
            println!("Printing array");
            print_array(item, output_buffer)
        }
        CJSON_OBJECT => {
            println!("Printing object");
            print_object(item, output_buffer)
        }
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

/* 

Parse

*/

fn get_decimal_point() -> char {
    '.' // Placeholder: Use locale-specific logic if needed
}

impl ParseBuffer {
    pub fn cannot_access_at_index(&self, index: usize) -> bool {
        self.offset + index >= self.content.len()
    }

    pub fn can_access_at_index(&self, index: usize) -> bool {
        self.offset + index < self.content.len()
    }

    pub fn buffer_at_offset(&self) -> &[u8] {
        &self.content[self.offset..]
    }

    pub fn can_read(&self, length: usize) -> bool {
        self.offset + length <= self.content.len()
    }

    pub fn skip_whitespace(&mut self) {
        while self.offset < self.length && self.content[self.offset].is_ascii_whitespace() {
            self.offset += 1;
        }
    }

}

pub fn parse_number(item: &mut CJSON, input_buffer: &mut ParseBuffer) -> bool {
    let mut number_c_string = String::with_capacity(64);
    let decimal_point = get_decimal_point();
    let mut i = 0;

    // Check if the input buffer is valid
    if input_buffer.content.is_empty() {
        return false;
    }

    // Copy the number into a temporary buffer, replacing '.' with the locale-specific decimal point
    while i < 63 && input_buffer.can_access_at_index(i) {
        let current_char = input_buffer.buffer_at_offset()[i];
        match current_char {
            b'0'..=b'9' | b'+' | b'-' | b'e' | b'E' => {
                number_c_string.push(current_char as char);
            }
            b'.' => {
                number_c_string.push(decimal_point);
            }
            _ => break,
        }
        i += 1;
    }

    // Attempt to parse the number from the string
    let number = match f64::from_str(&number_c_string) {
        Ok(num) => num,
        Err(_) => return false, // parse_error
    };

    item.valuedouble = number;

    // Handle integer overflow and underflow with saturation
    item.valueint = if number >= i32::MAX as f64 {
        i32::MAX
    } else if number <= i32::MIN as f64 {
        i32::MIN
    } else {
        number as i32
    };

    // Set the item type to CJSON_NUMBER
    item.item_type = CJSON_NUMBER;

    // Update the input buffer offset
    input_buffer.offset += i;
    true
}

pub fn parse_hex4(input: &[u8]) -> Option<u32> {
    if input.len() < 4 {
        return None; // Ensure the input has at least 4 characters
    }

    let mut h: u32 = 0;

    for i in 0..4 {
        h <<= 4; // Shift left by 4 bits (equivalent to multiplying by 16)

        // Parse the current hexadecimal digit
        match input[i] {
            b'0'..=b'9' => h += (input[i] - b'0') as u32,
            b'A'..=b'F' => h += (input[i] - b'A' + 10) as u32,
            b'a'..=b'f' => h += (input[i] - b'a' + 10) as u32,
            _ => return None, // Invalid character, return None
        }
    }

    Some(h)
}

pub fn utf16_literal_to_utf8(
    input_pointer: &[u8],
    input_end: &[u8],
    output_pointer: &mut Vec<u8>,
) -> Option<usize> {
    if input_pointer.len() < 6 || input_end.len() < 6 {
        return None; // Input ends unexpectedly
    }

    // Parse the first UTF-16 sequence
    let first_code = parse_hex4(&input_pointer[2..6])?;
    let mut codepoint: u32;
    let mut sequence_length: usize;

    // Check for valid UTF-16 surrogate pair
    if (0xDC00..=0xDFFF).contains(&first_code) {
        return None;
    }

    // Handle UTF-16 surrogate pair
    if (0xD800..=0xDBFF).contains(&first_code) {
        if input_pointer.len() < 12 || &input_pointer[6..8] != b"\\u" {
            return None; // Missing second half of the surrogate pair
        }

        // Parse the second UTF-16 sequence
        let second_code = parse_hex4(&input_pointer[8..12])?;
        if !(0xDC00..=0xDFFF).contains(&second_code) {
            return None; // Invalid second half of the surrogate pair
        }

        // Calculate the Unicode codepoint from the surrogate pair
        codepoint = 0x10000 + (((first_code & 0x3FF) << 10) | (second_code & 0x3FF));
        sequence_length = 12; // \uXXXX\uXXXX
    } else {
        // Single UTF-16 sequence
        codepoint = first_code;
        sequence_length = 6; // \uXXXX
    }

    // Determine the UTF-8 length and encode the codepoint
    let utf8_length = if codepoint < 0x80 {
        output_pointer.push(codepoint as u8);
        1
    } else if codepoint < 0x800 {
        output_pointer.push((0xC0 | (codepoint >> 6)) as u8);
        output_pointer.push((0x80 | (codepoint & 0x3F)) as u8);
        2
    } else if codepoint < 0x10000 {
        output_pointer.push((0xE0 | (codepoint >> 12)) as u8);
        output_pointer.push((0x80 | ((codepoint >> 6) & 0x3F)) as u8);
        output_pointer.push((0x80 | (codepoint & 0x3F)) as u8);
        3
    } else if codepoint <= 0x10FFFF {
        output_pointer.push((0xF0 | (codepoint >> 18)) as u8);
        output_pointer.push((0x80 | ((codepoint >> 12) & 0x3F)) as u8);
        output_pointer.push((0x80 | ((codepoint >> 6) & 0x3F)) as u8);
        output_pointer.push((0x80 | (codepoint & 0x3F)) as u8);
        4
    } else {
        return None; // Invalid Unicode codepoint
    };

    Some(sequence_length)
}

pub fn parse_string(item: &mut CJSON, input_buffer: &mut ParseBuffer) -> bool {
    // Check if the input starts with a double-quote
    if input_buffer.buffer_at_offset().first() != Some(&b'\"') {
        return false;
    }

    let mut input_pointer = &input_buffer.buffer_at_offset()[1..];
    let mut input_end = input_pointer;
    let mut skipped_bytes = 0;

    // Calculate the approximate size of the output (overestimate)
    while input_end.len() > 0 && input_end[0] != b'\"' {
        if input_end[0] == b'\\' {
            if input_end.len() < 2 {
                return false; // Prevent buffer overflow when the last character is a backslash
            }
            skipped_bytes += 1;
            input_end = &input_end[1..];
        }
        input_end = &input_end[1..];
    }

    // Check for unexpected end of the string
    if input_end.is_empty() || input_end[0] != b'\"' {
        return false;
    }

    // Calculate the allocation length for the output string
    let allocation_length = input_end.len() - skipped_bytes;
    let mut output = Vec::with_capacity(allocation_length);

    // Loop through the string literal
    while input_pointer < input_end {
        if input_pointer[0] != b'\\' {
            // Copy regular characters
            output.push(input_pointer[0]);
            input_pointer = &input_pointer[1..];
        } else {
            // Handle escape sequences
            if input_pointer.len() < 2 {
                return false;
            }

            match input_pointer[1] {
                b'b' => output.push(b'\x08'), // Backspace
                b'f' => output.push(b'\x0C'), // Formfeed
                b'n' => output.push(b'\n'),   // Newline
                b'r' => output.push(b'\r'),   // Carriage return
                b't' => output.push(b'\t'),   // Tab
                b'\"' | b'\\' | b'/' => output.push(input_pointer[1]),
                b'u' => {
                    // Handle UTF-16 escape sequence
                    if let Some(sequence_length) = utf16_literal_to_utf8(input_pointer, input_end, &mut output) {
                        input_pointer = &input_pointer[sequence_length..];
                        continue;
                    } else {
                        return false; // Failed to convert UTF-16 literal to UTF-8
                    }
                }
                _ => return false, // Invalid escape sequence
            }
            input_pointer = &input_pointer[2..];
        }
    }

    // Convert output to a Rust string and set the CJSON item
    match String::from_utf8(output) {
        Ok(valuestring) => {
            item.item_type = CJSON_STRING;
            item.valuestring = Some(valuestring);
        }
        Err(_) => return false, // Invalid UTF-8 sequence
    }

    // Update the input buffer offset
    input_buffer.offset += input_end.len() + 1;
    true
}


pub fn parse_object(item: &mut CJSON, input_buffer: &mut ParseBuffer) -> bool {
    let mut head: Option<Rc<RefCell<CJSON>>> = None;
    let mut current_item: Option<Rc<RefCell<CJSON>>> = None;

    // Check for nesting limit
    if input_buffer.depth >= CJSON_NESTING_LIMIT {
        return false;
    }
    input_buffer.depth += 1;

    // Check if the input starts with '{'
    if input_buffer.cannot_access_at_index(0) || input_buffer.buffer_at_offset()[0] != b'{' {
        return false;
    }

    input_buffer.offset += 1;
    input_buffer.skip_whitespace();

    // Check for an empty object
    if input_buffer.can_access_at_index(0) && input_buffer.buffer_at_offset()[0] == b'}' {
        input_buffer.depth -= 1;
        item.item_type = CJSON_OBJECT;
        return true;
    }

    // Step back to the character before the first element
    input_buffer.offset -= 1;

    // Loop through the comma-separated elements
    loop {
        // Allocate a new item
        let new_item = cJSON_New_Item();
        

        // Attach the new item to the linked list
        if head.is_none() {
            // Start the linked list
            current_item = Some(Rc::clone(&new_item));
            head = Some(Rc::clone(&new_item));
        } else {
            // Add to the end and advance
            if let Some(ref mut current) = current_item {
                current.borrow_mut().next = Some(Rc::clone(&new_item));
                new_item.borrow_mut().prev = Some(Rc::clone(current));
            }
            current_item = Some(Rc::clone(&new_item));
        }

        // Parse the name of the child (key)
        input_buffer.offset += 1;
        input_buffer.skip_whitespace();
        if !parse_string(&mut new_item.borrow_mut(), input_buffer) {
            return false;
        }
        input_buffer.skip_whitespace();

        // Swap `valuestring` and `string` fields
        {
            let mut new_item_mut = new_item.borrow_mut();
            new_item_mut.string = new_item_mut.valuestring.take();
        }

        // Check for the colon ':' separator
        if input_buffer.cannot_access_at_index(0) || input_buffer.buffer_at_offset()[0] != b':' {
            return false;
        }

        // Parse the value
        input_buffer.offset += 1;
        input_buffer.skip_whitespace();
        if !parse_value(&mut new_item.borrow_mut(), input_buffer) {
            return false;
        }
        input_buffer.skip_whitespace();

        // Check if the next character is a comma or the end of the object
        if !input_buffer.can_access_at_index(0) || input_buffer.buffer_at_offset()[0] != b',' {
            break;
        }
    }

    // Check for the end of the object '}'
    if input_buffer.cannot_access_at_index(0) || input_buffer.buffer_at_offset()[0] != b'}' {
        if let Some(head_item) = head {
            cjson_delete(Some(head_item));
        }
        return false;
    }

    // Update the CJSON item
    input_buffer.depth -= 1;
    if let Some(head_item) = head.clone() {
        head_item.borrow_mut().prev = current_item.clone();
    }

    item.item_type = CJSON_OBJECT;
    item.child = head;

    input_buffer.offset += 1;
    true
}

pub fn parse_value(item: &mut CJSON, input_buffer: &mut ParseBuffer) -> bool {
    // Check if the input buffer is valid
    if input_buffer.content.is_empty() {
        return false;
    }

    // Parse `null`
    if input_buffer.can_read(4) && input_buffer.buffer_at_offset().starts_with(b"null") {
        item.item_type = CJSON_NULL;
        input_buffer.offset += 4;
        return true;
    }

    // Parse `false`
    if input_buffer.can_read(5) && input_buffer.buffer_at_offset().starts_with(b"false") {
        item.item_type = CJSON_FALSE;
        input_buffer.offset += 5;
        return true;
    }

    // Parse `true`
    if input_buffer.can_read(4) && input_buffer.buffer_at_offset().starts_with(b"true") {
        item.item_type = CJSON_TRUE;
        item.valueint = 1;
        input_buffer.offset += 4;
        return true;
    }

    // Parse a string
    if input_buffer.can_access_at_index(0) && input_buffer.buffer_at_offset()[0] == b'\"' {
        return parse_string(item, input_buffer);
    }

    // Parse a number
    if input_buffer.can_access_at_index(0)
        && (input_buffer.buffer_at_offset()[0] == b'-'
            || (input_buffer.buffer_at_offset()[0] >= b'0' && input_buffer.buffer_at_offset()[0] <= b'9'))
    {
        return parse_number(item, input_buffer);
    }

    // Parse an array
    if input_buffer.can_access_at_index(0) && input_buffer.buffer_at_offset()[0] == b'[' {
        return parse_array(item, input_buffer);
    }

    // Parse an object
    if input_buffer.can_access_at_index(0) && input_buffer.buffer_at_offset()[0] == b'{' {
        return parse_object(item, input_buffer);
    }

    // If no matching type is found, return false
    false
}

pub fn parse_array(item: &mut CJSON, input_buffer: &mut ParseBuffer) -> bool {
    let mut head: Option<Rc<RefCell<CJSON>>> = None;
    let mut current_item: Option<Rc<RefCell<CJSON>>> = None;

    // Check for nesting limit
    if input_buffer.depth >= CJSON_NESTING_LIMIT {
        return false;
    }
    input_buffer.depth += 1;

    // Check if the input starts with '['
    if input_buffer.buffer_at_offset().first() != Some(&b'[') {
        return false;
    }

    input_buffer.offset += 1;
    input_buffer.skip_whitespace();

    // Check for an empty array
    if input_buffer.can_access_at_index(0) && input_buffer.buffer_at_offset()[0] == b']' {
        input_buffer.depth -= 1;
        item.item_type = CJSON_ARRAY;
        return true;
    }

    // Step back to the character before the first element
    input_buffer.offset -= 1;

    // Loop through the comma-separated elements
    loop {
        // Allocate a new item
        let new_item = cJSON_New_Item();

        // Attach the new item to the linked list
        if head.is_none() {
            // Start the linked list
            current_item = Some(Rc::clone(&new_item));
            head = Some(Rc::clone(&new_item));
        } else {
            // Add to the end and advance
            if let Some(ref mut current) = current_item {
                current.borrow_mut().next = Some(Rc::clone(&new_item));
                new_item.borrow_mut().prev = Some(Rc::clone(current));
            }
            current_item = Some(Rc::clone(&new_item));
        }

        // Parse the next value
        input_buffer.offset += 1;
        input_buffer.skip_whitespace();
        if !parse_value(&mut new_item.borrow_mut(), input_buffer) {
            if let Some(head_item) = head {
                cjson_delete(Some(head_item));
            }
            return false;
        }
        input_buffer.skip_whitespace();

        // Check if the next character is a comma or the end of the array
        if !input_buffer.can_access_at_index(0) || input_buffer.buffer_at_offset()[0] != b',' {
            break;
        }
    }

    // Check for the end of the array ']'
    if input_buffer.cannot_access_at_index(0) || input_buffer.buffer_at_offset()[0] != b']' {
        if let Some(head_item) = head {
            cjson_delete(Some(head_item));
        }
        return false;
    }

    // Update the CJSON item
    input_buffer.depth -= 1;
    if let Some(head_item) = head.clone() {
        head_item.borrow_mut().prev = current_item.clone();
    }

    item.item_type = CJSON_ARRAY;
    item.child = head;

    input_buffer.offset += 1;
    true
}

pub fn skip_utf8_bom(buffer: &mut ParseBuffer) -> Option<&mut ParseBuffer> {
    // Check if the buffer is valid and the offset is at the start (0)
    if buffer.content.is_empty() || buffer.offset != 0 {
        return None;
    }

    // Check for the UTF-8 BOM (`\xEF\xBB\xBF`)
    if buffer.can_access_at_index(3) && buffer.buffer_at_offset().starts_with(b"\xEF\xBB\xBF") {
        buffer.offset += 3;
    }

    Some(buffer)
}

fn handle_parse_failure(
    item: Rc<RefCell<CJSON>>,
    value: &str,
    buffer: &mut ParseBuffer,
    return_parse_end: Option<&mut usize>,
) -> Option<Rc<RefCell<CJSON>>> {
    cjson_delete(Some(item));

    let mut local_error = Error {
        json: Some(value.as_bytes().to_vec()),
        position: if buffer.offset < buffer.length {
            buffer.offset
        } else if buffer.length > 0 {
            buffer.length - 1
        } else {
            0
        },
    };

    // Update `return_parse_end` if provided
    if let Some(parse_end) = return_parse_end {
        *parse_end = local_error.position;
    }

    {
    let mut global_error = GLOBAL_ERROR.lock().unwrap();
        *global_error = local_error;
    }

    None
}

pub fn cjson_parse_with_length(value: &str, buffer_length: usize) -> Option<Rc<RefCell<CJSON>>> {
    cjson_parse_with_length_opts(value, buffer_length, None, false)
}

pub fn cjson_parse_with_length_opts(
    value: &str,
    buffer_length: usize,
    return_parse_end: Option<&mut usize>,
    require_null_terminated: bool,
) -> Option<Rc<RefCell<CJSON>>> {
    // Initialize the parse buffer
    let mut buffer = ParseBuffer {
        content: value.as_bytes().to_vec(),
        length: buffer_length,
        offset: 0,
        depth: 0,
    };

    // Reset the global error
    {
    let mut global_error = GLOBAL_ERROR.lock().unwrap();
    global_error.json = None;
    global_error.position = 0;
    }

    // Validate input
    if value.is_empty() || buffer_length == 0 {
        return None;
    }

    // Create a new CJSON item
    let item = cJSON_New_Item();
    
    // Skip UTF-8 BOM and whitespace, then parse the value
    buffer.skip_whitespace();
    if !parse_value(&mut item.borrow_mut(), &mut buffer) {
        return handle_parse_failure(item, value, &mut buffer, return_parse_end);
    }

    // Check for null-terminated JSON if required
    if require_null_terminated {
        buffer.skip_whitespace();
        if buffer.offset >= buffer.length || buffer.buffer_at_offset().get(0) != Some(&b'\0') {
            return handle_parse_failure(item, value, &mut buffer, return_parse_end);
        }
    }

    // Update `return_parse_end` if provided
    if let Some(parse_end) = return_parse_end {
        *parse_end = buffer.offset;
    }

    Some(item)
}


pub fn cjson_parse_with_opts(
    value: &str,
    return_parse_end: Option<&mut usize>,
    require_null_terminated: bool,
) -> Option<Rc<RefCell<CJSON>>> {
    // Check if the input value is `None` (equivalent to NULL in C)
    if value.is_empty() {
        return None;
    }

    // Calculate the buffer length, accounting for null-terminated requirement
    let buffer_length = value.len() + if require_null_terminated { 1 } else { 0 };

    // Delegate to `cjson_parse_with_length_opts`
    cjson_parse_with_length_opts(value, buffer_length, return_parse_end, require_null_terminated)
}


pub fn cjson_parse(value: &str) -> Option<Rc<RefCell<CJSON>>> {
    cjson_parse_with_opts(value, None, false)
}




/*
Unit Tests
*/

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

     #[test]
    fn test_print_string_simple() {
        let item = cjson_create_string("Hello, world!");
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(print_buffer.buffer, "\"Hello, world!\"");
    }

    #[test]
    fn test_print_string_with_escape_characters() {
        let item = cjson_create_string("Line1\nLine2\tTabbed");
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(print_buffer.buffer, "\"Line1\\nLine2\\tTabbed\"");
    }

    #[test]
    fn test_print_string_with_quotes() {
        let item = cjson_create_string("She said, \"Hello!\"");
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(print_buffer.buffer, "\"She said, \\\"Hello!\\\"\"");
    }

    #[test]
    fn test_print_string_with_unicode() {
        let item = cjson_create_string("Emoji: 😊");
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(print_buffer.buffer, "\"Emoji: 😊\"");
    }

    #[test]
    fn test_print_string_null() {
        let item = Rc::new(RefCell::new(CJSON {
            next: None,
            prev: None,
            child: None,
            item_type: CJSON_STRING,
            valuestring: None,
            valueint: 0,
            valuedouble: 0.0,
            string: None,
        }));
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(!result);
    }

    #[test]
    fn test_print_string_multiline() {
        let item = cjson_create_string("Line1\nLine2\nLine3");
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(print_buffer.buffer, "\"Line1\\nLine2\\nLine3\"");
    }

    #[test]
    fn test_print_string_with_control_characters() {
        let item = cjson_create_string("Control chars: \x01\x02\x03");
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(
            print_buffer.buffer,
            "\"Control chars: \\u0001\\u0002\\u0003\""
        );
    }

    #[test]
    fn test_print_string_with_mixed_escape_sequences() {
        let item = cjson_create_string("Tab\tNewline\nQuote\"Backslash\\");
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(
            print_buffer.buffer,
            "\"Tab\\tNewline\\nQuote\\\"Backslash\\\\\""
        );
    }

    #[test]
    fn test_print_string_empty() {
        let item = cjson_create_string("");
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(print_buffer.buffer, "\"\"");
    }
/*
    #[test]
    fn test_print_string_large_input() {
        let large_string = "A".repeat(1000);
        let item = cjson_create_string(&large_string);
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(print_buffer.buffer, format!("\"{}\"", large_string));
    }
 */
    #[test]
    fn test_print_string_with_utf8() {
        let item = cjson_create_string("こんにちは世界");
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(print_buffer.buffer, "\"こんにちは世界\"");
    }

    #[test]
    fn test_print_string_with_emoji() {
        let item = cjson_create_string("Smile 😊, Heart ❤️, Rocket 🚀");
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(print_buffer.buffer, "\"Smile 😊, Heart ❤️, Rocket 🚀\"");
    }

    #[test]
    fn test_print_string_with_backslashes() {
        let item = cjson_create_string("Path: C:\\Program Files\\App");
        let mut buffer = String::new();
        let mut print_buffer = PrintBuffer {
            buffer: &mut buffer,
            length: 0,
            offset: 0,
            noalloc: false,
            format: false,
        };

        let result = print_string(&item, &mut print_buffer);
        assert!(result);
        assert_eq!(print_buffer.buffer, "\"Path: C:\\\\Program Files\\\\App\"");
    }


    fn test_cjson_parse_with_array() {
        // Define the JSON input as a raw string
        let json_input = r#"
        [
            {
                "precision": "zip",
                "Latitude": 37.7668,
                "Longitude": -122.3959,
                "Address": "",
                "City": "SAN FRANCISCO",
                "State": "CA",
                "Zip": "94107",
                "Country": "US"
            },
            {
                "precision": "zip",
                "Latitude": 37.371991,
                "Longitude": -122.026020,
                "Address": "",
                "City": "SUNNYVALE",
                "State": "CA",
                "Zip": "94085",
                "Country": "US"
            }
        ]
        "#;

        // Parse the JSON input
        let parsed = cjson_parse(json_input);
        
        if parsed.is_none() {
            // Retrieve the error pointer using `cjson_get_error_ptr`
            if let Some(error_ptr) = cjson_get_error_ptr() {
                println!("Parsing failed at: {}", error_ptr);
            } else {
                println!("Parsing failed, but no error pointer was set.");
            }
        } else {
            println!("Parsing succeeded, but it was expected to fail.");
        }

        // Assert that the parsing was successful
        assert!(parsed.is_some(), "Failed to parse the JSON input");
      }

}
