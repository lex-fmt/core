//! Property-based tests for parameter parsing
//!
//! These tests ensure that parameter parsing is robust and handles
//! various valid inputs correctly according to the simplified grammar:
//! - Parameters must have key=value format (no boolean shorthand)
//! - Parameters are separated by commas only (not whitespace)
//! - Whitespace around parameters is ignored

use proptest::prelude::*;
use txxt::txxt::parser::parse_document;

/// Generate valid parameter keys
fn parameter_key_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple keys
        "[a-z][a-z0-9_-]{0,10}",
        // Keys with underscores
        "[a-z][a-z0-9_]{1,10}",
        // Keys with dashes
        "[a-z][a-z0-9-]{1,10}",
        // Mixed
        "[a-z][a-z0-9_-]{2,10}",
    ]
}

/// Generate valid unquoted parameter values
fn unquoted_value_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple alphanumeric values
        "[a-zA-Z0-9]+",
        // Values with dashes
        "[a-zA-Z0-9-]+",
        // Values with periods (for versions)
        "[0-9]+\\.[0-9]+",
        "[0-9]+\\.[0-9]+\\.[0-9]+",
    ]
}

/// Generate valid quoted parameter values
/// Note: We avoid commas and whitespace-only values for simplicity in testing
fn quoted_value_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple text with spaces (at least one non-space character)
        "[a-zA-Z0-9][a-zA-Z0-9 ]{0,19}",
        // Text with punctuation (no commas, at least one non-space)
        "[a-zA-Z0-9][a-zA-Z0-9 .-]{0,19}",
        // Simple alphanumeric text
        "[a-zA-Z0-9]{1,10}",
    ]
}

/// Generate a single valid parameter (key=value format only)
fn parameter_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Unquoted values
        (parameter_key_strategy(), unquoted_value_strategy())
            .prop_map(|(k, v)| format!("{}={}", k, v)),
        // Quoted values
        (parameter_key_strategy(), quoted_value_strategy())
            .prop_map(|(k, v)| format!("{}=\"{}\"", k, v)),
    ]
}

/// Generate valid parameter lists (comma-separated)
fn parameter_list_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(parameter_strategy(), 1..5).prop_map(|params| params.join(","))
}

#[cfg(test)]
mod proptest_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_single_parameter_parsing(param in parameter_strategy()) {
            let source = format!(":: note {} ::\n\nText. {{{{paragraph}}}}\n", param);
            let result = parse_document(&source);

            // Should parse successfully
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);

            if let Ok(doc) = result {
                let annotation = doc.root_session.content[0].as_annotation().unwrap();
                prop_assert_eq!(annotation.parameters.len(), 1);

                // Extract key and value from the parameter string
                let parts: Vec<&str> = param.splitn(2, '=').collect();
                prop_assert_eq!(&annotation.parameters[0].key, parts[0]);
            }
        }

        #[test]
        fn test_multiple_parameters_parsing(params in parameter_list_strategy()) {
            let source = format!(":: note {} ::\n\nText. {{{{paragraph}}}}\n", params);
            let result = parse_document(&source);

            // Should parse successfully
            prop_assert!(result.is_ok(), "Failed to parse: {}", source);

            if let Ok(doc) = result {
                let annotation = doc.root_session.content[0].as_annotation().unwrap();
                let expected_count = params.split(',').count();
                prop_assert_eq!(annotation.parameters.len(), expected_count);
            }
        }

        #[test]
        fn test_parameter_key_preservation(key in parameter_key_strategy(), value in unquoted_value_strategy()) {
            let source = format!(":: note {}={} ::\n\nText. {{{{paragraph}}}}\n", key, value);
            let result = parse_document(&source);

            prop_assert!(result.is_ok(), "Failed to parse: {}", source);

            if let Ok(doc) = result {
                let annotation = doc.root_session.content[0].as_annotation().unwrap();
                prop_assert_eq!(&annotation.parameters[0].key, &key);
                prop_assert_eq!(&annotation.parameters[0].value, &Some(value));
            }
        }

        #[test]
        fn test_quoted_value_preservation(key in parameter_key_strategy(), value in quoted_value_strategy()) {
            let source = format!(":: note {}=\"{}\" ::\n\nText. {{{{paragraph}}}}\n", key, value);
            let result = parse_document(&source);

            prop_assert!(result.is_ok(), "Failed to parse: {}", source);

            if let Ok(doc) = result {
                let annotation = doc.root_session.content[0].as_annotation().unwrap();
                prop_assert_eq!(&annotation.parameters[0].key, &key);
                prop_assert_eq!(&annotation.parameters[0].value, &Some(value));
            }
        }

        #[test]
        fn test_parameter_order_preservation(params in parameter_list_strategy()) {
            let source = format!(":: note {} ::\n\nText. {{{{paragraph}}}}\n", params);
            let result = parse_document(&source);

            prop_assert!(result.is_ok(), "Failed to parse: {}", source);

            if let Ok(doc) = result {
                let annotation = doc.root_session.content[0].as_annotation().unwrap();

                // Extract keys from the parameter string
                let expected_keys: Vec<&str> = params
                    .split(',')
                    .map(|p| p.split('=').next().unwrap())
                    .collect();

                let actual_keys: Vec<&str> = annotation.parameters
                    .iter()
                    .map(|p| p.key.as_str())
                    .collect();

                prop_assert_eq!(actual_keys, expected_keys);
            }
        }
    }
}

#[cfg(test)]
mod specific_tests {
    use super::*;

    #[test]
    fn test_comma_only_separator() {
        let source = ":: note key1=val1,key2=val2,key3=val3 ::\n\nText. {{paragraph}}\n";
        let result = parse_document(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(annotation.parameters.len(), 3);
    }

    #[test]
    fn test_whitespace_around_commas_ignored() {
        let source = ":: note key1=val1 , key2=val2 , key3=val3 ::\n\nText. {{paragraph}}\n";
        let result = parse_document(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(annotation.parameters.len(), 3);
        assert_eq!(annotation.parameters[0].key, "key1");
        assert_eq!(annotation.parameters[1].key, "key2");
        assert_eq!(annotation.parameters[2].key, "key3");
    }

    #[test]
    fn test_whitespace_around_equals_ignored() {
        let source = ":: note key1 = val1 , key2 = val2 ::\n\nText. {{paragraph}}\n";
        let result = parse_document(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(annotation.parameters.len(), 2);
        assert_eq!(annotation.parameters[0].value, Some("val1".to_string()));
        assert_eq!(annotation.parameters[1].value, Some("val2".to_string()));
    }

    #[test]
    fn test_quoted_values_with_spaces() {
        let source = ":: note message=\"Hello World\" ::\n\nText. {{paragraph}}\n";
        let result = parse_document(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(
            annotation.parameters[0].value,
            Some("Hello World".to_string())
        );
    }

    #[test]
    fn test_quoted_values_with_commas() {
        let source = ":: note message=\"value with, comma\" ::\n\nText. {{paragraph}}\n";
        let result = parse_document(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(
            annotation.parameters[0].value,
            Some("value with, comma".to_string())
        );
    }

    #[test]
    fn test_empty_quoted_value() {
        let source = ":: note message=\"\" ::\n\nText. {{paragraph}}\n";
        let result = parse_document(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(annotation.parameters[0].value, Some("".to_string()));
    }

    #[test]
    fn test_version_number_values() {
        let source = ":: note version=3.11.2 ::\n\nText. {{paragraph}}\n";
        let result = parse_document(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(annotation.parameters[0].value, Some("3.11.2".to_string()));
    }

    #[test]
    fn test_keys_with_dashes_and_underscores() {
        let source = ":: note ref-id=123,api_version=2 ::\n\nText. {{paragraph}}\n";
        let result = parse_document(source);
        assert!(result.is_ok());

        let doc = result.unwrap();
        let annotation = doc.root_session.content[0].as_annotation().unwrap();
        assert_eq!(annotation.parameters.len(), 2);
        assert_eq!(annotation.parameters[0].key, "ref-id");
        assert_eq!(annotation.parameters[1].key, "api_version");
    }
}
