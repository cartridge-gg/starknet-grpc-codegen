# Plan: JSON-RPC to gRPC Proto Generation

## Overview
This document outlines the plan to convert the existing Starknet JSON-RPC code generator to produce gRPC proto files instead of Rust code. The generated proto files will maintain 1:1 mapping with the original JSON structure and include native JSON marshalling support.

## Requirements Summary
- Generate `.proto` files only (no Rust implementation)
- Use existing JSON-RPC specs as source of truth
- Embed JSON marshalling in types using gRPC native features
- Maintain current service separation (main, write, trace, ws)
- Remove Rust code generation entirely
- Ensure 1:1 mapping with JSON spec for downstream consumption

## Phase 1: Analysis & Design
**Goal**: Understand the mapping strategy and design the proto generation approach

### 1.1 Analyze JSON-RPC to gRPC Mapping
- Map JSON-RPC methods to gRPC RPC definitions
- Convert JSON Schema types to Protobuf message types
- Design error handling strategy (JSON-RPC errors → gRPC status codes)
- Plan for preserving JSON field names using protobuf field options

### 1.2 Design Proto File Structure
- Create separate .proto files for each service (main, write, trace, ws)
- Define common types in a shared proto file
- Plan import structure between proto files
- Design versioning strategy for generated protos

### 1.3 JSON Marshalling Strategy
- Use protobuf's `json_name` field option to preserve exact JSON field names
- Map JSON schema constraints to protobuf validation rules
- Handle special cases (oneOf, allOf, nullable fields)

## Phase 2: Core Infrastructure Updates
**Goal**: Refactor the codebase structure for proto generation

### 2.1 Remove Rust Code Generation
- Delete Rust-specific generation code from `generate.rs`
- Remove Rust type definitions and serialization logic
- Clean up Rust-specific profile options

### 2.2 Create Proto Generation Module
- Create `src/proto_gen/mod.rs` for proto generation logic
- Implement proto syntax builders
- Add proto file writer with proper formatting

### 2.3 Update Data Structures
- Modify existing schema structures to support proto generation
- Add proto-specific metadata (package names, imports, options)
- Create mapping tables for type conversions

## Phase 3: Proto Generation Implementation
**Goal**: Implement the actual proto file generation

### 3.1 Service Generation
- Generate service definitions from JSON-RPC methods
- Create request/response message pairs for each RPC
- Add service-level options and documentation

### 3.2 Message Generation
- Convert JSON schemas to protobuf messages
- Handle nested types and repeated fields
- Implement oneOf/allOf conversions
- Add field-level json_name options

### 3.3 Type Mapping Implementation
- Map JSON primitive types to protobuf types
- Handle custom types (FELT, ADDRESS, etc.)
- Generate enum types from string enums
- Create wrapper messages for complex unions

### 3.4 Common Types Extraction
- Identify shared types across services
- Generate `common.proto` with shared definitions
- Update service protos to import common types

## Phase 4: Special Cases & Edge Cases
**Goal**: Handle complex scenarios and edge cases

### 4.1 Error Handling
- Design standard error message format
- Map JSON-RPC error codes to gRPC metadata
- Generate error-related messages

### 4.2 Complex Type Handling
- Handle polymorphic types (transaction types, receipts)
- Manage circular dependencies
- Deal with "any" types or untyped JSON

### 4.3 Streaming Support
- Convert WebSocket subscriptions to gRPC streaming
- Design streaming request/response messages

## Phase 5: Output & Integration
**Goal**: Finalize the output format and tooling

### 5.1 Proto File Organization
- Generate organized directory structure
- Add proto file headers with version info
- Include documentation from JSON specs

### 5.2 Build Configuration
- Generate buf.yaml or similar for proto compilation
- Add validation rules configuration
- Create example client generation scripts

### 5.3 Version Management
- Add version markers in generated protos
- Create changelog generation
- Design upgrade path for breaking changes

## Phase 6: Testing & Validation
**Goal**: Ensure correctness and compatibility

### 6.1 Validation Suite
- Validate generated protos compile correctly
- Test JSON marshalling/unmarshalling
- Verify 1:1 mapping with original JSON structure

### 6.2 Compatibility Testing
- Test with different gRPC implementations
- Verify JSON compatibility across languages
- Create test cases for edge cases

## Implementation Order
1. Start with Phase 2 (Core Infrastructure) - refactor existing code
2. Implement basic proto generation (Phase 3, steps 1-2)
3. Add type mapping (Phase 3, steps 3-4)
4. Handle special cases (Phase 4)
5. Finalize output format (Phase 5)
6. Add comprehensive testing (Phase 6)

## Key Technical Decisions

### Proto Package Structure
```
starknet.v0_8_1.main
starknet.v0_8_1.write
starknet.v0_8_1.trace
starknet.v0_8_1.ws
starknet.v0_8_1.common
```

### Type Mapping Examples
| JSON Schema Type | Protobuf Type | Notes |
|-----------------|---------------|-------|
| string | string | With json_name option |
| integer | int64/int32 | Based on constraints |
| boolean | bool | Direct mapping |
| array | repeated | Preserves ordering |
| object | message | Nested message type |
| oneOf | oneof | Proto native support |
| $ref | Import | Cross-file references |

### JSON Marshalling Example
```protobuf
message Transaction {
  string transaction_hash = 1 [json_name = "transaction_hash"];
  string block_hash = 2 [json_name = "block_hash"];
  int64 block_number = 3 [json_name = "block_number"];
}
```

## Success Criteria
- All JSON-RPC methods have corresponding gRPC service definitions
- Generated protos compile without errors
- JSON marshalling produces identical structure to original specs
- Proto files are well-organized and documented
- Downstream services can generate clients in multiple languages
- Version migration path is clear and documented

## Risks & Mitigations
- **Risk**: Complex nested types may not map cleanly to protobuf
  - **Mitigation**: Use wrapper messages and careful structuring
- **Risk**: Loss of JSON schema validation rules
  - **Mitigation**: Document validation requirements in proto comments
- **Risk**: Breaking changes in future spec versions
  - **Mitigation**: Version-specific packages and clear upgrade paths

## Timeline Estimate
- Phase 1-2: 2-3 days (Infrastructure and planning)
- Phase 3: 3-4 days (Core implementation)
- Phase 4: 2-3 days (Edge cases)
- Phase 5-6: 2-3 days (Polish and testing)
- **Total**: ~2 weeks for complete implementation 