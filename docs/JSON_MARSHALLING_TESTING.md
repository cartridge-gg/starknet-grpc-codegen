# JSON Marshalling Testing for Starknet gRPC Proto Generation

This document describes the comprehensive testing framework for validating JSON marshalling compatibility between Starknet JSON-RPC and our generated protobuf messages.

## Overview

Our gRPC proto generation maintains **1:1 JSON compatibility** with the Starknet JSON-RPC specification. This testing framework validates that:

- Generated protobuf messages can serialize to/from JSON correctly
- Field names are preserved exactly as in the JSON-RPC spec
- Complex nested structures are handled properly
- Real Starknet JSON-RPC data works with our proto definitions

## Test Data Sources

### Hurl Files from Dojo

We've imported comprehensive test data from the [Dojo project](https://github.com/dojoengine/dojo/tree/main/examples/rpc/starknet), which provides real-world Starknet JSON-RPC examples:

```
tests/hurl/
├── starknet_blockHashAndNumber.hurl
├── starknet_blockNumber.hurl
├── starknet_call.hurl
├── starknet_chainId.hurl
├── starknet_getBlockTransactionCount.hurl
├── starknet_getBlockWithReceipts.hurl
├── starknet_getBlockWithTxHashes.hurl
├── starknet_getBlockWithTxs.hurl
├── starknet_getClass.hurl
├── starknet_getClassAt.hurl
├── starknet_getClassHashAt.hurl
├── starknet_getEvents.hurl
├── starknet_getNonce.hurl
├── starknet_getStorageAt.hurl
├── starknet_getTransactionStatus.hurl
├── starknet_specVersion.hurl
└── starknet_trace.hurl
```

Each hurl file contains:
- HTTP request with JSON-RPC payload
- Expected response validation
- Real Starknet addresses and data

## Test Suite Components

### 1. JSON Marshalling Tests (`tests/json_marshalling_tests.rs`)

Core tests that validate JSON structure compatibility:

```rust
// Extract and validate JSON from hurl files
#[test]
fn test_extract_json_from_hurl_files()

// Validate field name preservation
#[test] 
fn test_json_field_name_preservation()

// Test hex string patterns
#[test]
fn test_hex_string_patterns()

// Test version string patterns
#[test]
fn test_version_string_patterns()

// Test array structure identification
#[test]
fn test_array_structure_identification()

// Test JSON-RPC error structure
#[test]
fn test_jsonrpc_error_structure()

// Test protobuf JSON compatibility
#[test]
fn test_protobuf_json_compatibility()
```

### 2. Proto JSON Integration Tests (`tests/proto_json_integration_tests.rs`)

Integration tests that validate generated proto files:

```rust
// Validate generated proto files exist and are structured correctly
#[test]
fn test_generated_proto_files_exist()

// Test JSON-RPC request structure compatibility
#[test]
fn test_jsonrpc_request_structure_compatibility()

// Test JSON-RPC response structure compatibility  
#[test]
fn test_jsonrpc_response_structure_compatibility()

// Test field names match protobuf expectations
#[test]
fn test_hurl_field_names_match_protobuf_expectations()

// Test error response structure compatibility
#[test]
fn test_error_response_structure_compatibility()

// Test streaming method identification
#[test]
fn test_streaming_method_identification()

// Test complex nested structure compatibility
#[test]
fn test_complex_nested_structure_compatibility()

// Test hex string validation patterns
#[test]
fn test_hex_string_validation_patterns()
```

### 3. Comprehensive Test Script (`scripts/test_json_marshalling.sh`)

Automated test runner that validates the entire pipeline:

```bash
./scripts/test_json_marshalling.sh
```

This script:
- Checks dependencies (cargo, jq, hurl)
- Runs all Rust tests
- Validates hurl files contain valid JSON
- Checks generated proto files for correct structure
- Tests specific JSON-RPC method structures
- Validates field name preservation
- Generates a comprehensive summary report

## Running the Tests

### Quick Test Run

```bash
# Run JSON marshalling tests
cargo test --test json_marshalling_tests

# Run integration tests
cargo test --test proto_json_integration_tests
```

### Comprehensive Validation

```bash
# Run the full test suite
./scripts/test_json_marshalling.sh
```

### Individual Test Categories

```bash
# Test specific functionality
cargo test test_extract_json_from_hurl_files
cargo test test_protobuf_json_compatibility
cargo test test_generated_proto_files_exist
```

## Key Validation Points

### 1. Field Name Preservation

Our protobuf generation uses `json_name` options to preserve exact JSON field names:

```protobuf
message CallRequest {
  string contract_address = 1 [json_name = "contract_address"];
  string entry_point_selector = 2 [json_name = "entry_point_selector"];
  repeated string calldata = 3 [json_name = "calldata"];
}
```

### 2. Critical Fields Tested

The test suite validates these essential Starknet JSON-RPC fields:

- `contract_address` - Smart contract addresses
- `entry_point_selector` - Function selectors
- `block_number` / `block_hash` - Block identifiers
- `parent_hash` - Block parent references
- `starknet_version` - Network version strings
- `from_block` / `to_block` - Event filtering ranges
- `chunk_size` - Pagination parameters
- `calldata` - Function call parameters
- `jsonrpc` / `method` / `params` / `id` - JSON-RPC structure

### 3. Data Type Validation

Tests validate proper handling of:

- **Hex strings**: `0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7`
- **Version strings**: `0.13.0`, `0.13.1`
- **Arrays**: Transaction lists, calldata arrays
- **Nested objects**: Event filters, block structures
- **Numbers**: Block numbers, chunk sizes
- **Booleans**: Status flags

### 4. JSON-RPC Structure Validation

Tests ensure compatibility with standard JSON-RPC 2.0:

```json
{
  "jsonrpc": "2.0",
  "method": "starknet_call",
  "params": [...],
  "id": 1
}
```

And error responses:

```json
{
  "jsonrpc": "2.0", 
  "error": {
    "code": -32602,
    "message": "Invalid params"
  },
  "id": 1
}
```

## Test Results Interpretation

### Successful Test Run

```
📊 JSON Marshalling Test Summary
================================

📁 Hurl test files: 17
📄 Generated proto files: 5
✅ All JSON marshalling tests passed
✅ Proto JSON integration tests passed  
✅ Field name preservation validated
✅ JSON-RPC structure compatibility confirmed

🎯 Key Achievements:
   • 1:1 JSON compatibility maintained
   • Protobuf field mappings validated
   • Real Starknet JSON-RPC data tested
   • Multi-language client generation ready
```

### Common Issues and Solutions

1. **Missing proto files**: Run `cargo run -- generate --spec 0.8.1` first
2. **Field name mismatches**: Check `json_name` options in proto generation
3. **Invalid JSON in hurl files**: Validate hurl file syntax
4. **Type mapping issues**: Review protobuf type mappings for JSON compatibility

## Integration with CI/CD

Add to your CI pipeline:

```yaml
- name: Test JSON Marshalling
  run: |
    cargo test --test json_marshalling_tests
    cargo test --test proto_json_integration_tests
    ./scripts/test_json_marshalling.sh
```

## Benefits of This Testing Framework

1. **Confidence**: Validates that generated proto files work with real data
2. **Compatibility**: Ensures 1:1 JSON compatibility is maintained
3. **Regression Prevention**: Catches breaking changes in proto generation
4. **Documentation**: Serves as living documentation of expected behavior
5. **Multi-language Support**: Validates that any language can use the protos correctly

## Future Enhancements

- **Live API Testing**: Test against running Starknet nodes
- **Performance Benchmarks**: Measure JSON marshalling performance
- **Schema Evolution**: Test backward compatibility across spec versions
- **Client Generation**: Test actual client code generation and usage
- **Streaming Validation**: Test WebSocket/streaming method compatibility

## Conclusion

This comprehensive testing framework ensures that our Starknet gRPC proto generation maintains perfect JSON compatibility while providing the benefits of gRPC's type safety, performance, and multi-language support. The use of real-world test data from the Dojo project gives us confidence that our generated protobuf messages will work correctly with existing Starknet infrastructure and tooling. 