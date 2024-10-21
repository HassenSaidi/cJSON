/*
This file contains the main cJSON file.
This file is the translation of the original cJSON C implementation in Rust.
*/


// Project version
const CJSON_VERSION_MAJOR: u32 = 1;
const CJSON_VERSION_MINOR: u32 = 7;
const CJSON_VERSION_PATCH: u32 = 15;

// cJSON Types
const cJSON_Invalid: u32 = 0;
const cJSON_False: u32 = 1 << 0;
const cJSON_True: u32 = 1 << 1;
const cJSON_NULL: u32 = 1 << 2;
const cJSON_Number: u32 = 1 << 3;
const cJSON_String: u32 = 1 << 4;
const cJSON_Array: u32 = 1 << 5;
const cJSON_Object: u32 = 1 << 6;
const cJSON_Raw: u32 = 1 << 7; // raw JSON

const cJSON_IsReference: u32 = 256;
const cJSON_StringIsConst: u32 = 512;


// CJSON structure
// type is renamed to type_
#[derive(Debug)]
#[derive(Clone)]
pub struct CJSON {
    pub next: Option<Box<CJSON>>,
    pub prev: Option<Box<CJSON>>, // Raw mutable pointer
    pub child: Option<Box<CJSON>>,
    pub type_: u32,
    pub valuestring: Option<String>,
    pub valueint: i32,
    pub valuedouble: f64,
    pub string: Option<String>,
}


// create a new instance
pub fn cJSON_New_Item() -> CJSON {
    CJSON {
        next: None,
        prev: None,
        child: None,
        type_: 0,
        valuestring: None,
        valueint: 0,
        valuedouble: 0.0,
        string: None,
        }
    }


// The following 
/* Create basic types: */
pub fn cJSON_CreateNull() -> CJSON {
    let mut item = cJSON_New_Item();
    item.type_ = cJSON_NULL;
    item
}

pub fn cJSON_CreateBool(value: bool) -> CJSON  {
        let mut item = cJSON_New_Item();
        item.type_ = if value { cJSON_True } else { cJSON_False };
        item.valueint = if value { 1 } else { 0 };
        item
}

pub fn cJSON_CreateNumber(num: f64) -> CJSON  {
        let mut item = cJSON_New_Item();
        item.type_ = cJSON_Number;
        item.valuedouble = num;
        item.valueint = num as i32;
        item
}

pub fn cJSON_CreateString(s: &str) -> CJSON  {
        let mut item = cJSON_New_Item();
        item.type_ = cJSON_String;
        item.valuestring = Some(s.to_string());
        item
}

pub fn cJSON_CreateTrue()-> CJSON {
    let mut item = cJSON_New_Item();
    item.type_ = cJSON_True;
    item
}    

pub fn cJSON_CreateFalse()-> CJSON {
    let mut item = cJSON_New_Item();
    item.type_ = cJSON_False;
    item
}

pub fn cJSON_CreateArray()-> CJSON {
    let mut item = cJSON_New_Item();
    item.type_ = cJSON_Array;
    item
}

pub fn cJSON_CreateObject()-> CJSON {
    let mut item = cJSON_New_Item();
    item.type_ = cJSON_Object;
    item
}

pub fn cJSON_CreateRaw(raw: &str) -> CJSON {
    let mut item = cJSON_New_Item();
    item.type_ = cJSON_Raw;
    item.valuestring = if raw.is_empty() {
        None    
        } else {
            Some(raw.to_string())
        };
    item
}


/* Create Arrays: */

/// Builds a linked list from a vector of `CJSON` items.
fn build_linked_list(mut items: Vec<CJSON>) -> Option<Box<CJSON>> {
    if items.is_empty() {
        return None;
    }

    let first_item = items.remove(0);
    let mut head = Box::new(first_item);
    let mut current = &mut head;

    for item in items {
        let mut boxed_item = Box::new(item);
        boxed_item.prev = Some(Box::new((*current).clone()));
        current.next = Some(boxed_item);
        current = current.next.as_mut().unwrap();
    }

    Some(head)
}

/// Creates a `CJSON` instance representing a JSON array of integers.
    pub fn cJSON_CreateIntArray(numbers: &[i32]) -> Option<CJSON> {
        if numbers.is_empty() {
            return None;
        }

        let mut array = cJSON_New_Item();
        array.type_ = cJSON_Array;

        let mut children = Vec::new();

        for &number in numbers {
            let number_cjson = cJSON_CreateNumber(number as f64);
            children.push(number_cjson);
        }

        array.child = build_linked_list(children);
        Some(array)
    }

