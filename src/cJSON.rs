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

pub fn create_bool(value: bool) -> CJSON  {
        let mut item = cJSON_New_Item();
        item.type_ = if value { CJSON_TRUE } else { CJSON_FALSE };
        item.valueint = if value { 1 } else { 0 };
        item
}

pub fn create_number(num: f64) -> CJSON  {
        let mut item = cJSON_New_Item();
        item.type_ = CJSON_NUMBER;
        item.valuedouble = num;
        item.valueint = num as i32;
        item
}

pub fn create_string(s: &str) -> CJSON  {
        let mut item = cJSON_New_Item();
        item.type_ = CJSON_STRING;
        item.valuestring = Some(s.to_string());
        item
}

//cJSON_CreateTrue
//cJSON_CreateFalse


