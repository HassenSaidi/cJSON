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
    // Get the formatted JSON string
    let out = cjson_print(root).ok_or_else(|| "Failed to generate JSON string".to_string())?;

    // Create a buffer with extra space (for safety)
    let len = out.len() + 5;
    let mut buf = String::with_capacity(len);

    // Create a buffer with exact size (to simulate potential failure)
    let len_fail = out.len();
    let mut buf_fail = String::with_capacity(len_fail);

    // Attempt to print into the buffer with extra capacity
    if !cjson_print_preallocated(root, &mut buf, len as i32, true) {
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
    if cjson_print_preallocated(root, &mut buf_fail, len_fail as i32, true) {
        println!("cJSON_PrintPreallocated did not fail with insufficient buffer size!");
        println!("cJSON_Print result:\n{}", out);
        println!("cJSON_PrintPreallocated result:\n{}", buf_fail);
        return Err("Failed to detect buffer overflow".to_string());
    }

    Ok(())
}



pub fn create_objects() {
    // Days of the week
    let strings = [
        "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
    ];

    // Matrix
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

    // Video object
    let root = cjson_create_object();
    cjson_add_item_to_object(&root, "name", cjson_create_string("Jack (\"Bee\") Nimble"));
    let fmt = cjson_create_object();
    cjson_add_item_to_object(&root, "format", Rc::clone(&fmt));
    cjson_add_string_to_object(&fmt, "type", "rect");
    cjson_add_number_to_object(&fmt, "width", 1920.0);
    cjson_add_number_to_object(&fmt, "height", 1080.0);
    cjson_add_false_to_object(&fmt, "interlace");
    cjson_add_number_to_object(&fmt, "frame rate", 24.0);

    if print_preallocated(&root) {
        cjson_delete(Some(root));
        return;
    }
    cjson_delete(Some(root));

    // Days of the week array
    let root = cjson_create_string_array(&strings);

    if print_preallocated(&root) {
        cjson_delete(Some(root));
        return;
    }
    cjson_delete(Some(root));

    // Matrix array
    let root = cjson_create_array();
    for row in &numbers {
        let int_array = cjson_create_int_array(row);
        cjson_add_item_to_array(&root, int_array);
    }

    if print_preallocated(&root) {
        cjson_delete(Some(root));
        return;
    }
    cjson_delete(Some(root));

    // Gallery item
    let root = cjson_create_object();
    let img = cjson_create_object();
    cjson_add_item_to_object(&root, "Image", Rc::clone(&img));
    cjson_add_number_to_object(&img, "Width", 800.0);
    cjson_add_number_to_object(&img, "Height", 600.0);
    cjson_add_string_to_object(&img, "Title", "View from 15th Floor");

    let thm = cjson_create_object();
    cjson_add_item_to_object(&img, "Thumbnail", Rc::clone(&thm));
    cjson_add_string_to_object(&thm, "Url", "http://www.example.com/image/481989943");
    cjson_add_number_to_object(&thm, "Height", 125.0);
    cjson_add_string_to_object(&thm, "Width", "100");

    cjson_add_item_to_object(&img, "IDs", cjson_create_int_array(&ids));

    if print_preallocated(&root) {
        cjson_delete(Some(root));
        return;
    }
    cjson_delete(Some(root));

    // Array of records
    let root = cjson_create_array();
    for record in &fields {
        let fld = cjson_create_object();
        cjson_add_item_to_array(&root, Rc::clone(&fld));
        cjson_add_string_to_object(&fld, "precision", record.precision);
        cjson_add_number_to_object(&fld, "Latitude", record.lat);
        cjson_add_number_to_object(&fld, "Longitude", record.lon);
        cjson_add_string_to_object(&fld, "Address", record.address);
        cjson_add_string_to_object(&fld, "City", record.city);
        cjson_add_string_to_object(&fld, "State", record.state);
        cjson_add_string_to_object(&fld, "Zip", record.zip);
        cjson_add_string_to_object(&fld, "Country", record.country);
    }

    if print_preallocated(&root) {
        cjson_delete(Some(root));
        return;
    }
    cjson_delete(Some(root));

    // Division by zero example
    let root = cjson_create_object();
    cjson_add_number_to_object(&root, "number", 1.0 / zero);

    if print_preallocated(&root) {
        cjson_delete(Some(root));
        return;
    }
    cjson_delete(Some(root));
}

fn main() {
    // Print the version
    println!("Version: {}", cjson_version());

    // Run the sample code for building objects
    create_objects();
}
