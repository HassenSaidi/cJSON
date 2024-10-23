/* This content is genrate by cuttom_gpt.

*/

/* cJSON Types: */
const cJSON_Invalid: u32 = 0;
const cJSON_False: u32 = 1 << 0;
const cJSON_True: u32 = 1 << 1;
const cJSON_NULL: u32 = 1 << 2;
const cJSON_Number: u32 = 1 << 3;
const cJSON_String: u32 = 1 << 4;
const cJSON_Array: u32 = 1 << 5;
const cJSON_Object: u32 = 1 << 6;
const cJSON_Raw: u32 = 1 << 7; /* raw json */

const cJSON_IsReference: u32 = 256;
const cJSON_StringIsConst: u32 = 512;

/* The cJSON structure: */
struct cJSON {
    /* next/prev allow you to walk array/object chains. Alternatively, use GetArraySize/GetArrayItem/GetObjectItem */
    next: Option<Box<cJSON>>,
    prev: Option<Box<cJSON>>,
    /* An array or object item will have a child pointer pointing to a chain of the items in the array/object. */
    child: Option<Box<cJSON>>,

    /* The type of the item, as above. */
    item_type: u32,

    /* The item's string, if item_type == cJSON_String and item_type == cJSON_Raw */
    valuestring: Option<String>,
    /* writing to valueint is DEPRECATED, use cJSON_SetNumberValue instead */
    valueint: i32,
    /* The item's number, if item_type == cJSON_Number */
    valuedouble: f64,

    /* The item's name string, if this item is the child of, or is in the list of subitems of an object. */
    string: Option<String>,
}
