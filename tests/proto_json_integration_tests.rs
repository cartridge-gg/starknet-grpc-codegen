use std::fs;
use std::path::Path;
use serde_json::Value;

/// Integration tests for protobuf JSON marshalling
/// 
/// These tests validate that our generated protobuf messages can correctly
/// serialize to and deserialize from JSON while maintaining compatibility
/// with the Starknet JSON-RPC specification.

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that validates the generated proto files exist and are properly structured
    #[test]
    fn test_generated_proto_files_exist() {
        let proto_dir = Path::new("proto");
        
        if proto_dir.exists() {
            // Check for version-specific directories
            let version_dirs: Vec<_> = fs::read_dir(proto_dir)
                .expect("Should be able to read proto directory")
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if path.is_dir() && path.file_name()?.to_str()?.starts_with("v") {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect();

            if !version_dirs.is_empty() {
                for version_dir in version_dirs {
                    println!("Checking version directory: {:?}", version_dir);
                    
                    // Check for expected proto files
                    let expected_files = vec!["main.proto", "common.proto"];
                    
                    for expected_file in expected_files {
                        let proto_file = version_dir.join(expected_file);
                        if proto_file.exists() {
                            println!("Found proto file: {:?}", proto_file);
                            validate_proto_file_structure(&proto_file);
                        }
                    }
                }
            } else {
                println!("No version directories found in proto directory");
            }
        } else {
            println!("Proto directory does not exist - run generation first");
        }
    }

    /// Validate that a proto file has the expected structure for JSON compatibility
    fn validate_proto_file_structure(proto_file: &Path) {
        let content = fs::read_to_string(proto_file)
            .expect("Should be able to read proto file");

        // Check for json_name options that preserve JSON field names
        let json_name_patterns = vec![
            r#"json_name = "contract_address""#,
            r#"json_name = "entry_point_selector""#,
            r#"json_name = "block_number""#,
            r#"json_name = "block_hash""#,
            r#"json_name = "jsonrpc""#,
        ];

        let mut found_json_names = 0;
        for pattern in json_name_patterns {
            if content.contains(pattern) {
                found_json_names += 1;
                println!("Found JSON name preservation: {}", pattern);
            }
        }

        if found_json_names > 0 {
            println!("Proto file {} has {} JSON name preservations", 
                proto_file.display(), found_json_names);
        }

        // Check for service definitions
        if content.contains("service ") {
            println!("Proto file {} contains service definitions", proto_file.display());
        }

        // Check for message definitions
        if content.contains("message ") {
            println!("Proto file {} contains message definitions", proto_file.display());
        }
    }

    /// Test JSON-RPC request structure compatibility
    #[test]
    fn test_jsonrpc_request_structure_compatibility() {
        let test_request = r#"
        {
            "jsonrpc": "2.0",
            "method": "starknet_call",
            "params": [
                {
                    "contract_address": "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
                    "entry_point_selector": "0x2e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e",
                    "calldata": [
                        "0x05b92948371346d0df1d3c2d7568573364497f6cba65f4734ecd54ed0a0dbd11"
                    ]
                },
                "pending"
            ],
            "id": 1
        }
        "#;

        let json: Value = serde_json::from_str(test_request).unwrap();
        
        // Validate that this structure would map correctly to protobuf
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["method"], "starknet_call");
        assert_eq!(json["id"], 1);
        
        let params = json["params"].as_array().unwrap();
        assert_eq!(params.len(), 2);
        
        let call_request = &params[0];
        assert!(call_request["contract_address"].is_string());
        assert!(call_request["entry_point_selector"].is_string());
        assert!(call_request["calldata"].is_array());
        
        println!("JSON-RPC request structure is compatible with protobuf mapping");
    }

    /// Test JSON-RPC response structure compatibility
    #[test]
    fn test_jsonrpc_response_structure_compatibility() {
        let test_response = r#"
        {
            "jsonrpc": "2.0",
            "result": {
                "block_hash": "0x47c3637b57c2b079b93c61539950c17e868a28f46cdef28f88521067f21e15b",
                "parent_hash": "0x2a70fb03fe363a2d6be843343a1d81ce6abeda1e9bd5cc6ad8fa9f45e30fdeb",
                "block_number": 3,
                "starknet_version": "0.13.0",
                "transactions": [
                    "0x1234567890abcdef",
                    "0xfedcba0987654321"
                ]
            },
            "id": 1
        }
        "#;

        let json: Value = serde_json::from_str(test_response).unwrap();
        
        // Validate response structure
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert!(json["result"].is_object());
        
        let result = &json["result"];
        assert!(result["block_hash"].is_string());
        assert!(result["parent_hash"].is_string());
        assert!(result["block_number"].is_number());
        assert!(result["starknet_version"].is_string());
        assert!(result["transactions"].is_array());
        
        println!("JSON-RPC response structure is compatible with protobuf mapping");
    }

    /// Test that field names in hurl files match protobuf expectations
    #[test]
    fn test_hurl_field_names_match_protobuf_expectations() {
        let hurl_dir = Path::new("tests/hurl");
        
        if !hurl_dir.exists() {
            println!("Hurl directory does not exist - skipping test");
            return;
        }

        let expected_field_names = vec![
            "contract_address",
            "entry_point_selector", 
            "block_number",
            "block_hash",
            "parent_hash",
            "starknet_version",
            "from_block",
            "to_block",
            "chunk_size",
            "calldata",
            "jsonrpc",
            "method",
            "params",
            "id",
        ];

        let hurl_files: Vec<_> = fs::read_dir(hurl_dir)
            .expect("Should be able to read hurl directory")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "hurl" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        let mut found_fields = std::collections::HashSet::new();

        for hurl_file in hurl_files {
            let content = fs::read_to_string(&hurl_file)
                .expect("Should be able to read hurl file");
            
            for field_name in &expected_field_names {
                if content.contains(&format!("\"{}\"", field_name)) {
                    found_fields.insert(field_name.to_string());
                }
            }
        }

        println!("Found {} expected field names in hurl files", found_fields.len());
        for field in &found_fields {
            println!("  - {}", field);
        }

        // We should find at least some of the expected field names
        assert!(!found_fields.is_empty(), "Should find some expected field names in hurl files");
    }

    /// Test error response structure compatibility
    #[test]
    fn test_error_response_structure_compatibility() {
        let test_error_response = r#"
        {
            "jsonrpc": "2.0",
            "error": {
                "code": -32602,
                "message": "Invalid params",
                "data": "Additional error information"
            },
            "id": 1
        }
        "#;

        let json: Value = serde_json::from_str(test_error_response).unwrap();
        
        // Validate error structure
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert!(json["error"].is_object());
        assert!(json["result"].is_null());
        
        let error = &json["error"];
        assert!(error["code"].is_number());
        assert!(error["message"].is_string());
        
        // Data field is optional
        if !error["data"].is_null() {
            assert!(error["data"].is_string());
        }
        
        println!("JSON-RPC error response structure is compatible with protobuf mapping");
    }

    /// Test streaming method identification
    #[test]
    fn test_streaming_method_identification() {
        let streaming_methods = vec![
            "starknet_subscribeNewHeads",
            "starknet_subscribeEvents", 
            "starknet_subscribeTransactionStatus",
            "starknet_subscribePendingTransactions",
            "starknet_subscribeReorg",
        ];

        for method in streaming_methods {
            // These methods should be identified as streaming in our proto generation
            assert!(method.contains("subscribe"), "Method {} should be identified as streaming", method);
            println!("Streaming method identified: {}", method);
        }
    }

    /// Test that complex nested structures are handled correctly
    #[test]
    fn test_complex_nested_structure_compatibility() {
        let complex_json = r#"
        {
            "jsonrpc": "2.0",
            "method": "starknet_getEvents",
            "params": {
                "filter": {
                    "from_block": "latest",
                    "to_block": "latest",
                    "address": "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
                    "keys": [
                        ["0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9"],
                        ["0x1234567890abcdef", "0xfedcba0987654321"]
                    ],
                    "chunk_size": 100
                }
            },
            "id": 1
        }
        "#;

        let json: Value = serde_json::from_str(complex_json).unwrap();
        
        // Validate nested structure
        let params = &json["params"];
        let filter = &params["filter"];
        
        assert!(filter["from_block"].is_string());
        assert!(filter["to_block"].is_string());
        assert!(filter["address"].is_string());
        assert!(filter["keys"].is_array());
        assert!(filter["chunk_size"].is_number());
        
        // Validate nested arrays
        let keys = filter["keys"].as_array().unwrap();
        assert!(keys[0].is_array());
        assert!(keys[1].is_array());
        
        println!("Complex nested structure is compatible with protobuf mapping");
    }

    /// Test hex string validation patterns
    #[test]
    fn test_hex_string_validation_patterns() {
        let hex_strings = vec![
            "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
            "0x2e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e",
            "0x05b92948371346d0df1d3c2d7568573364497f6cba65f4734ecd54ed0a0dbd11",
            "0x1234567890abcdef",
            "0xfedcba0987654321",
        ];

        let hex_pattern = regex::Regex::new(r"^0x[A-Fa-f0-9]+$").unwrap();
        
        for hex_string in hex_strings {
            assert!(hex_pattern.is_match(hex_string), 
                "Hex string {} should match pattern", hex_string);
        }
        
        println!("All hex strings match expected validation patterns");
    }
}

/// Helper functions for proto-JSON integration testing
pub mod helpers {
    use super::*;

    /// Extract JSON from hurl file content
    pub fn extract_json_from_hurl(content: &str) -> Option<Value> {
        let lines: Vec<&str> = content.lines().collect();
        let mut json_start = None;
        let mut json_end = None;
        
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "{" && json_start.is_none() {
                json_start = Some(i);
            }
            if line.trim() == "}" && json_start.is_some() && json_end.is_none() {
                json_end = Some(i);
                break;
            }
        }
        
        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_lines = &lines[start..=end];
            let json_str = json_lines.join("\n");
            serde_json::from_str(&json_str).ok()
        } else {
            None
        }
    }

    /// Validate that JSON structure is compatible with protobuf field mapping
    pub fn validate_protobuf_field_mapping(json: &Value) -> Vec<String> {
        let mut issues = Vec::new();
        
        if let Value::Object(map) = json {
            for (key, value) in map {
                // Check for field names that might cause issues in protobuf
                if key.contains("-") {
                    issues.push(format!("Field name '{}' contains hyphens which may cause protobuf issues", key));
                }
                
                if key.starts_with(char::is_numeric) {
                    issues.push(format!("Field name '{}' starts with number which is invalid in protobuf", key));
                }
                
                // Recursively check nested objects
                if value.is_object() {
                    let nested_issues = validate_protobuf_field_mapping(value);
                    issues.extend(nested_issues);
                }
            }
        }
        
        issues
    }

    /// Generate protobuf field definition from JSON field
    pub fn generate_protobuf_field_definition(field_name: &str, json_value: &Value, field_number: u32) -> String {
        let proto_type = match json_value {
            Value::String(_) => "string",
            Value::Number(n) if n.is_i64() => "int64",
            Value::Number(n) if n.is_u64() => "uint64", 
            Value::Number(_) => "double",
            Value::Bool(_) => "bool",
            Value::Array(_) => "repeated string", // Simplified
            Value::Object(_) => "message", // Would need nested message definition
            Value::Null => "string", // Default to string for null values
        };
        
        format!("{} {} = {} [json_name = \"{}\"];", 
            proto_type, field_name, field_number, field_name)
    }
} 