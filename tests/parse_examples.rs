use std::fs;
use std::io::{self, Read};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use cjson::cJSON::CJSON;
use cjson::cJSON::cjson_print;
use cjson::cJSON::cjson_parse;
use cjson::cJSON::cjson_get_error_ptr;
use cjson::cJSON::cjson_parse_with_length;
use cjson::cJSON::cjson_delete;
use std::path::PathBuf;


fn get_test_file_path(filename: &str) -> PathBuf {
    let project_root = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(project_root).join("tests/inputs").join(filename)
}

fn do_test(test_name: &str) -> Result<(), String> {
    // Define the base directory for the test files
    const TEST_DIR_PATH: &str = "tests/inputs/";

    // Construct the paths for the test input and expected output files
    let test_path = format!("{}{}", TEST_DIR_PATH, test_name);
    let expected_path = format!("{}{}.expected", TEST_DIR_PATH, test_name);

    println!("Looking for expected file at: {:?}", expected_path);

    // Read the expected output
    let expected = read_file(&expected_path)
        .map_err(|e| format!("Failed to read expected output: {}", e))?;


    // Read and parse the test input
    let tree = parse_file(&test_path)
        .ok_or("Failed to read or parse test input")?;

    // Print the parsed tree back to JSON
    let actual = cjson_print(&tree)
        .ok_or("Failed to print tree back to JSON")?;

    // Compare the actual output with the expected output
    if expected.trim() == actual.trim() {
        println!("Test '{}' passed!", test_name);
        Ok(())
    } else {
        Err(format!("Test '{}' failed: Output does not match expected", test_name))
    }
}

// Helper function to read the content of a file
fn read_file(filename: &str) -> io::Result<String> {
    let mut file = fs::File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

// Helper function to parse a file (assumes `parse_file` function is implemented)
fn parse_file(filename: &str) -> Option<Rc<RefCell<CJSON>>> {
    let content = read_file(filename).ok()?;
    cjson_parse(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_test1_should_be_parsed_and_printed() {
        do_test("test1").expect("Test 'test_1' failed");
    }
  
    #[test]
    fn file_test2_should_be_parsed_and_printed() {
        do_test("test2").expect("Test 'test_2' failed");
    }

  
    #[test]
    fn file_test3_should_be_parsed_and_printed() {
        do_test("test3").expect("Test 'test_3' failed");
    }
  
    #[test]
    fn file_test4_should_be_parsed_and_printed() {
        do_test("test4").expect("Test 'test_4' failed");
    }
  
    #[test]
    fn file_test5_should_be_parsed_and_printed() {
        do_test("test5").expect("Test 'test_5' failed");
    }

    #[test]
    fn file_test6_should_not_be_parsed() {
      // Read the content of "inputs/test6"
      let test6 = read_file("test6").expect("Failed to read test6 data");

      // Attempt to parse the content
      let tree = cjson_parse(&test6);

      // Assert that parsing fails (tree should be `None`)
      assert!(tree.is_none(), "Should fail to parse what is not JSON");

      // Assert that the error pointer matches the input
      let error_ptr = cjson_get_error_ptr().expect("Error pointer should not be null");
      assert_eq!(test6.as_str(), error_ptr, "Error pointer is incorrect");
    }

    
    #[test]
    fn file_test7_should_be_parsed_and_printed() {
        do_test("test7").expect("Test 'test_7' failed");
    }
    #[test]
    fn file_test8_should_be_parsed_and_printed() {
        do_test("test8").expect("Test 'test_8' failed");
    }
    #[test]
    fn file_test9_should_be_parsed_and_printed() {
        do_test("test9").expect("Test 'test_9' failed");
    }
    #[test]
    fn file_test10_should_be_parsed_and_printed() {
        do_test("test10").expect("Test 'test_10' failed");
    }
    #[test]
    fn file_test11_should_be_parsed_and_printed() {
        do_test("test11").expect("Test 'test_11' failed");
    }

    #[test]
    fn test12_should_not_be_parsed() {
      // Define the incomplete JSON input
      let test12 = "{ \"name\": ";

      // Attempt to parse the JSON input
      let tree = cjson_parse(test12);

      // Assert that parsing fails (tree should be `None`)
      assert!(tree.is_none(), "Should fail to parse incomplete JSON.");

      // Assert that the error pointer matches the expected position
      let error_ptr = cjson_get_error_ptr().expect("Error pointer should not be null");
      let expected_ptr = &test12[test12.len()..];
      assert_eq!(expected_ptr, error_ptr, "Error pointer is incorrect");
    }

    #[test]
    fn test13_should_be_parsed_without_null_termination() {
      // Define the JSON input without null-termination
      let test_13 = concat!(
          "{",
          "\"Image\":{",
          "\"Width\":800,",
          "\"Height\":600,",
          "\"Title\":\"Viewfrom15thFloor\",",
          "\"Thumbnail\":{",
          "\"Url\":\"http://www.example.com/image/481989943\",",
          "\"Height\":125,",
          "\"Width\":\"100\"",
          "},",
          "\"IDs\":[116,943,234,38793]",
          "}",
          "}"
      );

      // Create a byte slice without null termination
      let test_13_wo_null = &test_13.as_bytes()[..test_13.len()];

      // Parse the JSON input without null-termination
      let tree = cjson_parse_with_length(
          std::str::from_utf8(test_13_wo_null).unwrap(),
          test_13_wo_null.len(),
      );

      // Assert that parsing is successful
      assert!(tree.is_some(), "Failed to parse valid JSON.");

      // Clean up the parsed JSON tree
      if let Some(tree) = tree {
          cjson_delete(Some(tree));
      }
    }

     #[test]
     fn test14_should_not_be_parsed() {
      // Define the JSON input
      let test_14 = concat!(
          "{",
          "\"Image\":{",
          "\"Width\":800,",
          "\"Height\":600,",
          "\"Title\":\"Viewfrom15thFloor\",",
          "\"Thumbnail\":{",
          "\"Url\":\"http://www.example.com/image/481989943\",",
          "\"Height\":125,",
          "\"Width\":\"100\"",
          "},",
          "\"IDs\":[116,943,234,38793]",
          "}",
          "}"
      );

      // Define the buffer length to be shorter than the input length (simulate a truncated input)
      let buffer_length = test_14.len() - 2;

      // Parse the JSON input with a limited buffer length
      let tree = cjson_parse_with_length(&test_14[..buffer_length], buffer_length);

      // Assert that parsing fails (tree should be `None`)
      assert!(tree.is_none(), "Should not continue after buffer_length is reached.");

      // Clean up if the tree was incorrectly parsed
      if let Some(tree) = tree {
          cjson_delete(Some(tree));
      }
    }


}


