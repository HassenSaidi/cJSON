use std::rc::Rc;
use std::cell::RefCell;

use cjson::cJSON::CJSON;
use cjson::cJSON::cjson_print;
use cjson::cJSON::cjson_create_object;
use cjson::cJSON::cjson_add_item_to_object;
use cjson::cJSON::cjson_create_string;
use cjson::cJSON::cjson_add_string_to_object;
use cjson::cJSON::cjson_add_number_to_object;
use cjson::cJSON::cjson_add_false_to_object;
use cjson::cJSON::cjson_delete;
use cjson::cJSON::cjson_create_string_array;
use cjson::cJSON::cjson_create_array;
use cjson::cJSON::cjson_create_int_array;
use cjson::cJSON::cjson_add_item_to_array;
use cjson::cJSON::cjson_version;
use cjson::cJSON::cjson_print_preallocated;




struct Record<'a> {
    precision: &'a str,
    lat: f64,
    lon: f64,
    address: &'a str,
    city: &'a str,
    state: &'a str,
    zip: &'a str,
    country: &'a str,
}

fn print_preallocated(root: &Rc<RefCell<CJSON>>) -> Result<(), String> {
    // Generate formatted JSON string
    let out = cJSON_Print(root).ok_or("Failed to generate JSON string")?;

    // Create a buffer to succeed (with extra space for safety)
    let len = out.len() + 5;
    let mut buf = String::with_capacity(len);

    // Create a buffer with exact size (to simulate potential failure)
    let len_fail = out.len();
    let mut buf_fail = String::with_capacity(len_fail);

    // Attempt to print into the buffer with extra capacity
    if !cJSON_PrintPreallocated(root, &mut buf, len, true) {
        println!("cJSON_PrintPreallocated failed!");

        if out != buf {
            println!("cJSON_PrintPreallocated result is different from cJSON_Print!");
            println!("cJSON_Print result:\n{}", out);
            println!("cJSON_PrintPreallocated result:\n{}", buf);
        }

        return Err("Failed to print JSON with sufficient buffer size".to_string());
    }

    // Print the result
    println!("{}", buf);

    // Force a failure by using the smaller buffer
    if cJSON_PrintPreallocated(root, &mut buf_fail, len_fail, true) {
        println!("cJSON_PrintPreallocated did not fail with insufficient buffer size!");
        println!("cJSON_Print result:\n{}", out);
        println!("cJSON_PrintPreallocated result:\n{}", buf_fail);
        return Err("Failed to detect buffer overflow".to_string());
    }

    Ok(())
}


fn create_objects() {
    // Days of the week
    let strings = [
        "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
    ];

    // Matrix of integers
    let numbers = [[0, -1, 0], [1, 0, 0], [0, 0, 1]];

    // Gallery IDs
    let ids = [116, 943, 234, 38793];

    // Array of records
    let fields = [
        Record {
            precision: "zip",
            lat: 37.7668,
            lon: -122.3959,
            address: "",
            city: "SAN FRANCISCO",
            state: "CA",
            zip: "94107",
            country: "US",
        },
        Record {
            precision: "zip",
            lat: 37.371991,
            lon: -122.026,
            address: "",
            city: "SUNNYVALE",
            state: "CA",
            zip: "94085",
            country: "US",
        },
    ];

    let zero = 0.0;

    // Create a "Video" JSON object
    let root = cJSON_CreateObject();
    cJSON_AddItemToObject(&root, "name", cJSON_CreateString("Jack (\"Bee\") Nimble"));
    let fmt = cJSON_CreateObject();
    cJSON_AddItemToObject(&root, "format", Rc::clone(&fmt));
    cJSON_AddStringToObject(&fmt, "type", "rect");
    cJSON_AddNumberToObject(&fmt, "width", 1920.0);
    cJSON_AddNumberToObject(&fmt, "height", 1080.0);
    cJSON_AddFalseToObject(&fmt, "interlace");
    cJSON_AddNumberToObject(&fmt, "frame rate", 24.0);

    // Print and delete the root object
    if print_preallocated(&root).is_err() {
        cJSON_DELETE(Some(root));
        return;
    }
    cJSON_DELETE(Some(root));

    // Create "Days of the week" string array
    let root = cJSON_CreateStringArray(&strings).unwrap();
    if print_preallocated(&root).is_err() {
        cJSON_DELETE(Some(root));
        return;
    }
    cJSON_DELETE(Some(root));

    // Create a matrix array
    let root = cJSON_CreateArray();
    for row in &numbers {
        let int_array = cJSON_CreateIntArray(row);
        cJSON_AddItemToArray(&root, int_array);
    }

    if print_preallocated(&root).is_err() {
        cJSON_DELETE(Some(root));
        return;
    }
    cJSON_DELETE(Some(root));

    // Create a "Gallery" JSON object
    let root = cJSON_CreateObject();
    let img = cJSON_CreateObject();
    cJSON_AddItemToObject(&root, "Image", Rc::clone(&img));
    cJSON_AddNumberToObject(&img, "Width", 800.0);
    cJSON_AddNumberToObject(&img, "Height", 600.0);
    cJSON_AddStringToObject(&img, "Title", "View from 15th Floor");

    let thm = cJSON_CreateObject();
    cJSON_AddItemToObject(&img, "Thumbnail", Rc::clone(&thm));
    cJSON_AddStringToObject(&thm, "Url", "http://www.example.com/image/481989943");
    cJSON_AddNumberToObject(&thm, "Height", 125.0);
    cJSON_AddStringToObject(&thm, "Width", "100");
    cJSON_AddItemToObject(&img, "IDs", cJSON_CreateIntArray(&ids));

    if print_preallocated(&root).is_err() {
        cJSON_DELETE(Some(root));
        return;
    }
    cJSON_DELETE(Some(root));

    // Create an array of records
    let root = cJSON_CreateArray();
    for record in &fields {
        let fld = cJSON_CreateObject();
        cJSON_AddItemToArray(&root, Rc::clone(&fld));
        cJSON_AddStringToObject(&fld, "precision", record.precision);
        cJSON_AddNumberToObject(&fld, "Latitude", record.lat);
        cJSON_AddNumberToObject(&fld, "Longitude", record.lon);
        cJSON_AddStringToObject(&fld, "Address", record.address);
        cJSON_AddStringToObject(&fld, "City", record.city);
        cJSON_AddStringToObject(&fld, "State", record.state);
        cJSON_AddStringToObject(&fld, "Zip", record.zip);
        cJSON_AddStringToObject(&fld, "Country", record.country);
    }

    if print_preallocated(&root).is_err() {
        cJSON_DELETE(Some(root));
        return;
    }
    cJSON_DELETE(Some(root));

    // Handle division by zero example
    let root = cJSON_CreateObject();
    cJSON_AddNumberToObject(&root, "number", 1.0 / zero);

    if print_preallocated(&root).is_err() {
        cJSON_DELETE(Some(root));
        return;
    }
    cJSON_DELETE(Some(root));
}



fn main() {
    // Print the version
    println!("Version: {}", cjson_version());

    // Run the sample code for building objects
    create_objects();
}
