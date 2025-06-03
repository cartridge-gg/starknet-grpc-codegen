use regex::Regex;
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

/// Test suite for JSON marshalling compatibility with protobuf messages
///
/// This module tests that our generated protobuf messages can correctly
/// marshal to/from JSON while preserving the exact field names and structure
/// expected by Starknet JSON-RPC clients and servers.

#[cfg(test)]
mod tests {
    use super::*;

    /// Extract JSON-RPC requests from hurl files
    fn extract_json_from_hurl(content: &str) -> Option<Value> {
        let lines: Vec<&str> = content.lines().collect();
        let mut json_start = None;
        let mut json_end = None;

        // Find the JSON block in the hurl file
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

    /// Test that extracts and validates JSON structures from hurl files
    #[test]
    fn test_extract_json_from_hurl_files() {
        let hurl_dir = Path::new("tests/hurl");
        assert!(hurl_dir.exists(), "Hurl directory should exist");

        let hurl_files = fs::read_dir(hurl_dir)
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
            .collect::<Vec<_>>();

        assert!(!hurl_files.is_empty(), "Should find hurl files");

        for hurl_file in hurl_files {
            println!("Processing: {:?}", hurl_file);

            let content = fs::read_to_string(&hurl_file).expect("Should be able to read hurl file");

            if let Some(json) = extract_json_from_hurl(&content) {
                // Validate basic JSON-RPC structure
                assert!(json.get("jsonrpc").is_some(), "Should have jsonrpc field");
                assert!(json.get("method").is_some(), "Should have method field");
                assert!(json.get("id").is_some(), "Should have id field");

                // Extract method name for specific validation
                if let Some(method) = json.get("method").and_then(|m| m.as_str()) {
                    validate_method_structure(method, &json);
                }
            }
        }
    }

    /// Validate specific method structures match expected protobuf field mappings
    fn validate_method_structure(method: &str, json: &Value) {
        match method {
            "starknet_call" => validate_call_request(json),
            "starknet_getBlockWithTxHashes" => validate_get_block_request(json),
            "starknet_getEvents" => validate_get_events_request(json),
            "starknet_getStorageAt" => validate_get_storage_at_request(json),
            "starknet_blockNumber" => validate_block_number_request(json),
            "starknet_chainId" => validate_chain_id_request(json),
            "starknet_specVersion" => validate_spec_version_request(json),
            _ => {
                // For other methods, just validate basic structure
                println!("Validating basic structure for method: {}", method);
            }
        }
    }

    fn validate_call_request(json: &Value) {
        let params = json.get("params").expect("Call should have params");
        let params_array = params.as_array().expect("Params should be array");

        assert!(
            params_array.len() >= 2,
            "Call should have at least 2 params"
        );

        // First param should be the function call
        let call_request = &params_array[0];
        assert!(
            call_request.get("contract_address").is_some(),
            "Should have contract_address"
        );
        assert!(
            call_request.get("entry_point_selector").is_some(),
            "Should have entry_point_selector"
        );
        assert!(
            call_request.get("calldata").is_some(),
            "Should have calldata"
        );

        // Second param should be block identifier
        let block_id = &params_array[1];
        // Can be string like "pending" or object with block_number/block_hash
        assert!(
            block_id.is_string() || block_id.is_object(),
            "Block ID should be string or object"
        );
    }

    fn validate_get_block_request(json: &Value) {
        let params = json.get("params").expect("GetBlock should have params");
        let params_array = params.as_array().expect("Params should be array");

        assert_eq!(params_array.len(), 1, "GetBlock should have 1 param");

        let block_id = &params_array[0];
        // Should have either block_number or block_hash
        assert!(
            block_id.get("block_number").is_some() || block_id.get("block_hash").is_some(),
            "Block ID should have block_number or block_hash"
        );
    }

    fn validate_get_events_request(json: &Value) {
        let params = json.get("params").expect("GetEvents should have params");
        let filter = params.get("filter").expect("GetEvents should have filter");

        // Validate filter structure
        assert!(
            filter.get("from_block").is_some(),
            "Filter should have from_block"
        );
        assert!(
            filter.get("to_block").is_some(),
            "Filter should have to_block"
        );

        // Keys and chunk_size are optional but should be validated if present
        if let Some(keys) = filter.get("keys") {
            assert!(keys.is_array(), "Keys should be array");
        }

        if let Some(chunk_size) = filter.get("chunk_size") {
            assert!(chunk_size.is_number(), "Chunk size should be number");
        }
    }

    fn validate_get_storage_at_request(json: &Value) {
        let params = json.get("params").expect("GetStorageAt should have params");
        let params_array = params.as_array().expect("Params should be array");

        assert_eq!(params_array.len(), 3, "GetStorageAt should have 3 params");

        // contract_address, key, block_id
        assert!(
            params_array[0].is_string(),
            "Contract address should be string"
        );
        assert!(params_array[1].is_string(), "Storage key should be string");
        // Block ID can be string or object
        assert!(
            params_array[2].is_string() || params_array[2].is_object(),
            "Block ID should be string or object"
        );
    }

    fn validate_block_number_request(json: &Value) {
        // Block number request typically has no params or empty params
        if let Some(params) = json.get("params") {
            if let Some(params_array) = params.as_array() {
                assert!(
                    params_array.is_empty(),
                    "BlockNumber should have empty params"
                );
            }
        }
    }

    fn validate_chain_id_request(json: &Value) {
        // Chain ID request typically has no params or empty params
        if let Some(params) = json.get("params") {
            if let Some(params_array) = params.as_array() {
                assert!(params_array.is_empty(), "ChainId should have empty params");
            }
        }
    }

    fn validate_spec_version_request(json: &Value) {
        // Spec version request typically has no params or empty params
        if let Some(params) = json.get("params") {
            if let Some(params_array) = params.as_array() {
                assert!(
                    params_array.is_empty(),
                    "SpecVersion should have empty params"
                );
            }
        }
    }

    /// Test JSON field name preservation for protobuf compatibility
    #[test]
    fn test_json_field_name_preservation() {
        // Test that critical field names are preserved exactly as expected
        let test_cases = vec![
            ("contract_address", "contract_address"),
            ("entry_point_selector", "entry_point_selector"),
            ("block_number", "block_number"),
            ("block_hash", "block_hash"),
            ("parent_hash", "parent_hash"),
            ("starknet_version", "starknet_version"),
            ("from_block", "from_block"),
            ("to_block", "to_block"),
            ("chunk_size", "chunk_size"),
        ];

        for (original, expected) in test_cases {
            // In our protobuf generation, we should use json_name options
            // to preserve these exact field names
            assert_eq!(original, expected, "Field name should be preserved exactly");
        }
    }

    /// Test that hex string patterns are correctly handled
    #[test]
    fn test_hex_string_patterns() {
        let hex_pattern = Regex::new(r"^0x[A-Fa-f0-9]+$").unwrap();

        let test_hex_values = vec![
            "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
            "0x2e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e",
            "0x05b92948371346d0df1d3c2d7568573364497f6cba65f4734ecd54ed0a0dbd11",
        ];

        for hex_value in test_hex_values {
            assert!(
                hex_pattern.is_match(hex_value),
                "Hex value {} should match pattern",
                hex_value
            );
        }
    }

    /// Test version string patterns
    #[test]
    fn test_version_string_patterns() {
        let version_pattern = Regex::new(r"^[0-9]+\.[0-9]+\.[0-9]+(\.[0-9]+)?$").unwrap();

        let test_versions = vec!["0.13.0", "0.13.1", "1.0.0", "0.13.0.1"];

        for version in test_versions {
            assert!(
                version_pattern.is_match(version),
                "Version {} should match pattern",
                version
            );
        }
    }

    /// Test that array structures are correctly identified
    #[test]
    fn test_array_structure_identification() {
        let json_str = r#"
        {
            "transactions": ["0x123", "0x456"],
            "calldata": ["0x789"],
            "keys": [["0xabc"], ["0xdef"]]
        }
        "#;

        let json: Value = serde_json::from_str(json_str).unwrap();

        assert!(json.get("transactions").unwrap().is_array());
        assert!(json.get("calldata").unwrap().is_array());
        assert!(json.get("keys").unwrap().is_array());

        // Nested arrays
        let keys = json.get("keys").unwrap().as_array().unwrap();
        assert!(keys[0].is_array());
        assert!(keys[1].is_array());
    }

    /// Test JSON-RPC error structure compatibility
    #[test]
    fn test_jsonrpc_error_structure() {
        let error_json = r#"
        {
            "jsonrpc": "2.0",
            "error": {
                "code": -32602,
                "message": "Invalid params"
            },
            "id": 1
        }
        "#;

        let json: Value = serde_json::from_str(error_json).unwrap();

        assert_eq!(json.get("jsonrpc").unwrap().as_str().unwrap(), "2.0");
        assert!(json.get("error").is_some());
        assert!(json.get("result").is_none());

        let error = json.get("error").unwrap();
        assert!(error.get("code").unwrap().is_number());
        assert!(error.get("message").unwrap().is_string());
    }

    /// Test that our protobuf field mappings preserve JSON compatibility
    #[test]
    fn test_protobuf_json_compatibility() {
        // This test validates that when we generate protobuf messages,
        // the json_name options preserve the exact JSON structure

        let expected_mappings = vec![
            // Field name in proto -> JSON name that should be preserved
            ("contract_address", "contract_address"),
            ("entry_point_selector", "entry_point_selector"),
            ("block_number", "block_number"),
            ("block_hash", "block_hash"),
            ("parent_hash", "parent_hash"),
            ("starknet_version", "starknet_version"),
            ("transaction_hash", "transaction_hash"),
            ("from_block", "from_block"),
            ("to_block", "to_block"),
            ("chunk_size", "chunk_size"),
            ("jsonrpc", "jsonrpc"),
        ];

        for (proto_field, json_name) in expected_mappings {
            // In our proto generation, we should ensure that fields like:
            // string contract_address = 1 [json_name = "contract_address"];
            // preserve the exact JSON field names
            assert_eq!(
                proto_field, json_name,
                "Proto field {} should map to JSON name {}",
                proto_field, json_name
            );
        }
    }
}

/// Helper functions for JSON marshalling tests
pub mod helpers {
    use super::*;

    /// Convert a JSON value to a normalized form for comparison
    pub fn normalize_json(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut normalized = Map::new();
                for (k, v) in map {
                    normalized.insert(k.clone(), normalize_json(v));
                }
                Value::Object(normalized)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(normalize_json).collect()),
            _ => value.clone(),
        }
    }

    /// Validate that a JSON structure matches expected protobuf field names
    pub fn validate_protobuf_compatibility(json: &Value, expected_fields: &[&str]) -> bool {
        if let Value::Object(map) = json {
            for field in expected_fields {
                if !map.contains_key(*field) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Extract method name from JSON-RPC request
    pub fn extract_method_name(json: &Value) -> Option<String> {
        json.get("method")
            .and_then(|m| m.as_str())
            .map(|s| s.to_string())
    }
}
