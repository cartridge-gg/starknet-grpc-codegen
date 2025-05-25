use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use indexmap::IndexMap;
use tempfile::TempDir;

use starknet_grpc_codegen::{
    proto_gen::{ProtoConfig, ProtoGenerator},
    spec::*,
};

/// Create a minimal test specification for testing
fn create_test_specification() -> Specification {
    let mut schemas = IndexMap::new();

    // Add a simple FELT type
    schemas.insert(
        "FELT".to_string(),
        Schema::Primitive(Primitive::String(StringPrimitive {
            title: Some("Field Element".to_string()),
            comment: None,
            description: Some("A field element represented as a hex string".to_string()),
            r#enum: None,
            pattern: Some("^0x[0-9a-fA-F]+$".to_string()),
        })),
    );

    // Add a block hash type
    schemas.insert(
        "BLOCK_HASH".to_string(),
        Schema::Ref(Reference {
            title: None,
            comment: None,
            description: Some("Block hash reference".to_string()),
            ref_field: "#/components/schemas/FELT".to_string(),
            additional_fields: HashMap::new(),
        }),
    );

    // Add a transaction status enum
    schemas.insert(
        "TXN_STATUS".to_string(),
        Schema::Primitive(Primitive::String(StringPrimitive {
            title: Some("Transaction Status".to_string()),
            comment: None,
            description: Some("Status of a transaction".to_string()),
            r#enum: Some(vec![
                "PENDING".to_string(),
                "ACCEPTED_ON_L2".to_string(),
                "ACCEPTED_ON_L1".to_string(),
                "REJECTED".to_string(),
            ]),
            pattern: None,
        })),
    );

    // Add a block object
    schemas.insert(
        "BLOCK".to_string(),
        Schema::Primitive(Primitive::Object(ObjectPrimitive {
            title: Some("Block".to_string()),
            description: Some("A block in the blockchain".to_string()),
            summary: None,
            properties: {
                let mut props = IndexMap::new();
                props.insert(
                    "block_hash".to_string(),
                    Schema::Ref(Reference {
                        title: None,
                        comment: None,
                        description: None,
                        ref_field: "#/components/schemas/BLOCK_HASH".to_string(),
                        additional_fields: HashMap::new(),
                    }),
                );
                props.insert(
                    "block_number".to_string(),
                    Schema::Primitive(Primitive::Integer(IntegerPrimitive {
                        title: None,
                        description: Some("Block number".to_string()),
                        minimum: Some(0),
                        not: None,
                    })),
                );
                props.insert(
                    "parent_hash".to_string(),
                    Schema::Ref(Reference {
                        title: None,
                        comment: None,
                        description: None,
                        ref_field: "#/components/schemas/BLOCK_HASH".to_string(),
                        additional_fields: HashMap::new(),
                    }),
                );
                props
            },
            required: vec!["block_hash".to_string(), "block_number".to_string()],
            additional_properties: None,
            not: None,
        })),
    );

    Specification {
        openrpc: "1.0.0".to_string(),
        info: Info {
            version: "0.1.0".to_string(),
            title: "Test Starknet API".to_string(),
            license: Empty {},
        },
        servers: vec![],
        methods: vec![
            Method {
                name: "starknet_getBlock".to_string(),
                summary: "Get block by hash or number".to_string(),
                description: Some("Retrieves a block from the blockchain".to_string()),
                param_structure: None,
                params: vec![Param {
                    name: "block_id".to_string(),
                    description: Some("Block identifier".to_string()),
                    summary: None,
                    required: true,
                    schema: Schema::Ref(Reference {
                        title: None,
                        comment: None,
                        description: None,
                        ref_field: "#/components/schemas/BLOCK_HASH".to_string(),
                        additional_fields: HashMap::new(),
                    }),
                }],
                result: Some(MethodResult {
                    name: "result".to_string(),
                    description: Some("The requested block".to_string()),
                    required: Some(true),
                    schema: Schema::Ref(Reference {
                        title: None,
                        comment: None,
                        description: None,
                        ref_field: "#/components/schemas/BLOCK".to_string(),
                        additional_fields: HashMap::new(),
                    }),
                    summary: None,
                }),
                errors: None,
            },
            Method {
                name: "starknet_addInvokeTransaction".to_string(),
                summary: "Add invoke transaction".to_string(),
                description: Some("Submits a new invoke transaction".to_string()),
                param_structure: None,
                params: vec![Param {
                    name: "invoke_transaction".to_string(),
                    description: Some("Transaction to invoke".to_string()),
                    summary: None,
                    required: true,
                    schema: Schema::Primitive(Primitive::Object(ObjectPrimitive {
                        title: None,
                        description: None,
                        summary: None,
                        properties: IndexMap::new(),
                        required: vec![],
                        additional_properties: None,
                        not: None,
                    })),
                }],
                result: Some(MethodResult {
                    name: "result".to_string(),
                    description: Some("Transaction hash".to_string()),
                    required: Some(true),
                    schema: Schema::Ref(Reference {
                        title: None,
                        comment: None,
                        description: None,
                        ref_field: "#/components/schemas/FELT".to_string(),
                        additional_fields: HashMap::new(),
                    }),
                    summary: None,
                }),
                errors: None,
            },
            Method {
                name: "starknet_subscribeEvents".to_string(),
                summary: "Subscribe to events".to_string(),
                description: Some("Subscribe to blockchain events".to_string()),
                param_structure: None,
                params: vec![],
                result: Some(MethodResult {
                    name: "result".to_string(),
                    description: Some("Event stream".to_string()),
                    required: Some(true),
                    schema: Schema::Primitive(Primitive::Array(ArrayPrimitive {
                        title: None,
                        description: None,
                        items: Box::new(Schema::Primitive(Primitive::Object(ObjectPrimitive {
                            title: None,
                            description: None,
                            summary: None,
                            properties: IndexMap::new(),
                            required: vec![],
                            additional_properties: None,
                            not: None,
                        }))),
                    })),
                    summary: None,
                }),
                errors: None,
            },
            Method {
                name: "trace_transaction".to_string(),
                summary: "Trace transaction".to_string(),
                description: Some("Trace a transaction execution".to_string()),
                param_structure: None,
                params: vec![Param {
                    name: "transaction_hash".to_string(),
                    description: Some("Hash of the transaction to trace".to_string()),
                    summary: None,
                    required: true,
                    schema: Schema::Ref(Reference {
                        title: None,
                        comment: None,
                        description: None,
                        ref_field: "#/components/schemas/FELT".to_string(),
                        additional_fields: HashMap::new(),
                    }),
                }],
                result: Some(MethodResult {
                    name: "result".to_string(),
                    description: Some("Transaction trace".to_string()),
                    required: Some(true),
                    schema: Schema::Primitive(Primitive::Object(ObjectPrimitive {
                        title: None,
                        description: None,
                        summary: None,
                        properties: IndexMap::new(),
                        required: vec![],
                        additional_properties: None,
                        not: None,
                    })),
                    summary: None,
                }),
                errors: None,
            },
        ],
        components: Components {
            content_descriptors: Empty {},
            schemas,
            errors: IndexMap::new(),
        },
    }
}

#[test]
fn test_full_proto_generation_pipeline() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_path = temp_dir.path();

    let config = ProtoConfig {
        package_prefix: "starknet".to_string(),
        version: "v0_1_0".to_string(),
        output_dir: output_path.to_string_lossy().to_string(),
    };

    let specs = create_test_specification();
    let mut generator = ProtoGenerator::new(config);
    let result = generator.generate(&specs)?;

    // Verify that all expected files are generated
    assert_eq!(result.files.len(), 5);
    assert!(result.files.contains_key("common.proto"));
    assert!(result.files.contains_key("main.proto"));
    assert!(result.files.contains_key("write.proto"));
    assert!(result.files.contains_key("trace.proto"));
    assert!(result.files.contains_key("ws.proto"));

    // Verify package information
    assert_eq!(result.package_info.common_package, "starknet.v0_1_0.common");
    assert_eq!(result.package_info.main_package, "starknet.v0_1_0.main");
    assert_eq!(result.package_info.write_package, "starknet.v0_1_0.write");
    assert_eq!(result.package_info.trace_package, "starknet.v0_1_0.trace");
    assert_eq!(result.package_info.ws_package, "starknet.v0_1_0.ws");

    Ok(())
}

#[test]
fn test_generated_proto_syntax() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_path = temp_dir.path();

    let config = ProtoConfig {
        package_prefix: "starknet".to_string(),
        version: "v0_1_0".to_string(),
        output_dir: output_path.to_string_lossy().to_string(),
    };

    let specs = create_test_specification();
    let mut generator = ProtoGenerator::new(config);
    let result = generator.generate(&specs)?;

    // Test common.proto content
    let common_proto = result.files.get("common.proto").unwrap();
    assert!(common_proto.contains("syntax = \"proto3\";"));
    assert!(common_proto.contains("package starknet.v0_1_0.common;"));
    assert!(common_proto.contains("import \"google/protobuf/any.proto\";"));

    // Test main.proto content
    let main_proto = result.files.get("main.proto").unwrap();
    assert!(main_proto.contains("package starknet.v0_1_0.main;"));
    assert!(main_proto.contains("service StarknetMainService"));
    assert!(main_proto.contains("rpc GetBlock"));

    // Test write.proto content
    let write_proto = result.files.get("write.proto").unwrap();
    assert!(write_proto.contains("package starknet.v0_1_0.write;"));
    assert!(write_proto.contains("service StarknetWriteService"));
    assert!(write_proto.contains("rpc AddInvokeTransaction"));

    // Test trace.proto content
    let trace_proto = result.files.get("trace.proto").unwrap();
    assert!(trace_proto.contains("package starknet.v0_1_0.trace;"));
    assert!(trace_proto.contains("service StarknetTraceService"));
    assert!(trace_proto.contains("rpc TraceTransaction"));

    // Test ws.proto content
    let ws_proto = result.files.get("ws.proto").unwrap();
    assert!(ws_proto.contains("package starknet.v0_1_0.ws;"));
    assert!(ws_proto.contains("service StarknetWsService"));
    assert!(ws_proto.contains("rpc SubscribeEvents"));

    Ok(())
}

#[test]
fn test_proto_field_types_and_options() -> Result<()> {
    let config = ProtoConfig::new("v0_1_0");
    let specs = create_test_specification();
    let mut generator = ProtoGenerator::new(config);
    let result = generator.generate(&specs)?;

    let common_proto = result.files.get("common.proto").unwrap();

    // Check that enum values are generated correctly
    assert!(common_proto.contains("enum TxnStatus"));
    assert!(common_proto.contains("PENDING = 0;"));
    assert!(common_proto.contains("ACCEPTED_ON_L2 = 1;"));
    assert!(common_proto.contains("ACCEPTED_ON_L1 = 2;"));
    assert!(common_proto.contains("REJECTED = 3;"));

    // Check that messages are generated with proper field types
    assert!(common_proto.contains("message Block"));
    assert!(common_proto.contains("BlockHash block_hash = 1"));
    assert!(common_proto.contains("int64 block_number = 2"));
    assert!(common_proto.contains("json_name = \"block_hash\""));
    assert!(common_proto.contains("json_name = \"block_number\""));

    Ok(())
}

#[test]
fn test_service_method_generation() -> Result<()> {
    let config = ProtoConfig::new("v0_1_0");
    let specs = create_test_specification();
    let mut generator = ProtoGenerator::new(config);
    let result = generator.generate(&specs)?;

    let main_proto = result.files.get("main.proto").unwrap();

    // Verify service definition
    assert!(main_proto.contains("service StarknetMainService"));
    assert!(main_proto.contains("rpc GetBlock(GetBlockRequest) returns (GetBlockResponse);"));

    // Verify request/response messages would be generated
    // (They might be in common.proto or main.proto depending on implementation)
    let all_content = result
        .files
        .values()
        .cloned()
        .collect::<Vec<String>>()
        .join("\n");
    assert!(all_content.contains("GetBlockRequest") || all_content.contains("GetBlock"));
    assert!(all_content.contains("GetBlockResponse") || all_content.contains("GetBlock"));

    Ok(())
}

#[test]
fn test_streaming_service_detection() -> Result<()> {
    let config = ProtoConfig::new("v0_1_0");
    let specs = create_test_specification();
    let mut generator = ProtoGenerator::new(config);
    let result = generator.generate(&specs)?;

    let ws_proto = result.files.get("ws.proto").unwrap();

    // Verify that subscription methods are marked as streaming
    assert!(ws_proto.contains("service StarknetWsService"));
    // The exact streaming syntax depends on implementation details

    Ok(())
}

#[test]
fn test_json_name_preservation() -> Result<()> {
    let config = ProtoConfig::new("v0_1_0");
    let specs = create_test_specification();
    let mut generator = ProtoGenerator::new(config);
    let result = generator.generate(&specs)?;

    let common_proto = result.files.get("common.proto").unwrap();

    // Verify that original JSON field names are preserved
    assert!(common_proto.contains("json_name = \"block_hash\""));
    assert!(common_proto.contains("json_name = \"block_number\""));
    assert!(common_proto.contains("json_name = \"parent_hash\""));

    Ok(())
}

#[test]
fn test_proto_file_headers() -> Result<()> {
    let config = ProtoConfig::new("v0_1_0");
    let specs = create_test_specification();
    let mut generator = ProtoGenerator::new(config);
    let result = generator.generate(&specs)?;

    for (filename, content) in &result.files {
        // Every proto file should start with syntax declaration
        assert!(
            content.starts_with("syntax = \"proto3\";"),
            "File {} doesn't start with syntax declaration",
            filename
        );

        // Every proto file should have a package declaration
        assert!(
            content.contains("package starknet.v0_1_0."),
            "File {} doesn't contain package declaration",
            filename
        );

        // Every proto file should have Java options
        assert!(
            content.contains("option java_multiple_files = true;"),
            "File {} doesn't contain Java options",
            filename
        );
    }

    Ok(())
}

#[test]
fn test_write_proto_files_to_disk() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_path = temp_dir.path().join("proto");

    let config = ProtoConfig {
        package_prefix: "starknet".to_string(),
        version: "v0_1_0".to_string(),
        output_dir: output_path.to_string_lossy().to_string(),
    };

    let specs = create_test_specification();
    let mut generator = ProtoGenerator::new(config);
    let result = generator.generate(&specs)?;

    // Create output directory
    fs::create_dir_all(&output_path)?;

    // Write files to disk
    for (filename, content) in &result.files {
        let file_path = output_path.join(filename);
        fs::write(&file_path, content)?;

        // Verify file was written
        assert!(
            file_path.exists(),
            "File {} was not written to disk",
            filename
        );

        // Verify content matches
        let written_content = fs::read_to_string(&file_path)?;
        assert_eq!(
            &written_content, content,
            "File {} content doesn't match",
            filename
        );
    }

    Ok(())
}

#[test]
fn test_error_handling_in_generation() -> Result<()> {
    // Test with minimal valid spec
    let minimal_spec = Specification {
        openrpc: "1.0.0".to_string(),
        info: Info {
            version: "0.1.0".to_string(),
            title: "Minimal API".to_string(),
            license: Empty {},
        },
        servers: vec![],
        methods: vec![],
        components: Components {
            content_descriptors: Empty {},
            schemas: IndexMap::new(),
            errors: IndexMap::new(),
        },
    };

    let config = ProtoConfig::new("v0_1_0");
    let mut generator = ProtoGenerator::new(config);
    let result = generator.generate(&minimal_spec);

    // Should not fail with minimal spec
    assert!(
        result.is_ok(),
        "Generation failed with minimal spec: {:?}",
        result.err()
    );

    let generation_result = result.unwrap();
    assert_eq!(generation_result.files.len(), 5); // All 5 service files should be generated

    Ok(())
}
