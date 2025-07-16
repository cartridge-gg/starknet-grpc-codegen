use anyhow::Result;

use crate::spec::*;
use crate::proto_gen::{ProtoConfig, types::*, writer::*};

/// Service generator for creating gRPC services from JSON-RPC methods
pub struct ServiceGenerator<'a> {
    service_name: &'a str,
    #[allow(dead_code)]
    config: &'a ProtoConfig,
}

impl<'a> ServiceGenerator<'a> {
    pub fn new(service_name: &'a str, config: &'a ProtoConfig) -> Self {
        Self {
            service_name,
            config,
        }
    }

    pub fn generate_from_methods(&self, methods: &[&Method]) -> Result<ProtoService> {
        let mut rpcs = Vec::new();

        for method in methods {
            let rpc = self.convert_method_to_rpc(method)?;
            rpcs.push(rpc);
        }

        Ok(ProtoService {
            name: self.service_name.to_string(),
            rpcs,
            comment: Some(format!("Generated gRPC service for Starknet {}", self.service_name)),
        })
    }

    fn convert_method_to_rpc(&self, method: &Method) -> Result<ProtoRpc> {
        let rpc_name = self.method_name_to_rpc_name(&method.name);
        let request_type = format!("{}Request", rpc_name);
        let response_type = format!("{}Response", rpc_name);

        // Determine if streaming based on method characteristics
        let (client_streaming, server_streaming) = self.determine_streaming_type(method);

        Ok(ProtoRpc {
            name: rpc_name,
            request_type,
            response_type,
            comment: method.description.clone().or_else(|| Some(method.summary.clone())),
            client_streaming,
            server_streaming,
        })
    }

    fn method_name_to_rpc_name(&self, method_name: &str) -> String {
        // Convert starknet_getBlock to GetBlock
        let name_without_prefix = if let Some(stripped) = method_name.strip_prefix("starknet_") {
            stripped // Remove "starknet_"
        } else {
            method_name
        };
        
        // Convert camelCase or snake_case to PascalCase
        if name_without_prefix.contains('_') {
            // Handle snake_case
            self.snake_to_pascal_case(name_without_prefix)
        } else {
            // Handle camelCase - just capitalize the first letter
            let mut chars = name_without_prefix.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }

    fn snake_to_pascal_case(&self, snake_str: &str) -> String {
        snake_str
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        let mut result = first.to_uppercase().collect::<String>();
                        result.push_str(&chars.as_str().to_lowercase());
                        result
                    }
                }
            })
            .collect()
    }

    fn determine_streaming_type(&self, method: &Method) -> (bool, bool) {
        // Client streaming: typically for batch operations
        let client_streaming = method.name.contains("batch") || 
                              method.name.contains("multi") ||
                              method.params.len() > 5; // Heuristic for complex requests

        // Server streaming: for subscriptions and large responses
        let server_streaming = method.name.contains("subscribe") ||
                              method.name.contains("stream") ||
                              method.name.contains("events") ||
                              method.name.contains("logs");

        (client_streaming, server_streaming)
    }

    pub fn generate_request_response_messages(&self, method: &Method) -> Result<(ProtoMessage, ProtoMessage)> {
        let rpc_name = self.method_name_to_rpc_name(&method.name);
        let request_message = self.generate_request_message(&rpc_name, method)?;
        let response_message = self.generate_response_message(&rpc_name, method)?;

        Ok((request_message, response_message))
    }

    fn generate_request_message(&self, rpc_name: &str, method: &Method) -> Result<ProtoMessage> {
        let mut fields = Vec::new();
        let mut field_number = 1u32;

        for param in &method.params {
            let field_name = to_proto_name(&param.name);
            let field_type = self.schema_to_proto_field_type(&param.schema)?;

            fields.push(ProtoField {
                name: field_name,
                field_type,
                number: field_number,
                json_name: Some(param.name.clone()),
                comment: param.description.clone(),
                optional: !param.required,
                repeated: false,
                oneof_name: None,
            });

            field_number += 1;
        }

        Ok(ProtoMessage {
            name: format!("{}Request", rpc_name),
            fields,
            nested_messages: vec![],
            nested_enums: vec![],
            oneofs: vec![],
            comment: Some(format!("Request message for {}", method.name)),
            options: vec![],
        })
    }

    fn generate_response_message(&self, rpc_name: &str, method: &Method) -> Result<ProtoMessage> {
        let mut fields = Vec::new();

        if let Some(result) = &method.result {
            let field_type = self.schema_to_proto_field_type(&result.schema)?;
            
            fields.push(ProtoField {
                name: "result".to_string(),
                field_type,
                number: 1,
                json_name: Some("result".to_string()),
                comment: result.description.clone(),
                optional: false,
                repeated: false,
                oneof_name: None,
            });
        }

        // Add error field for standard gRPC error handling
        fields.push(ProtoField {
            name: "error".to_string(),
            field_type: ProtoFieldType::Message("Error".to_string()),
            number: 2,
            json_name: Some("error".to_string()),
            comment: Some("Error information if the request failed".to_string()),
            optional: true,
            repeated: false,
            oneof_name: None,
        });

        Ok(ProtoMessage {
            name: format!("{}Response", rpc_name),
            fields,
            nested_messages: vec![],
            nested_enums: vec![],
            oneofs: vec![],
            comment: Some(format!("Response message for {}", method.name)),
            options: vec![],
        })
    }

    fn schema_to_proto_field_type(&self, schema: &Schema) -> Result<ProtoFieldType> {
        schema_to_proto_field_type_impl(schema)
    }

    #[allow(dead_code)]
    pub fn generate_error_message() -> ProtoMessage {
        ProtoMessage {
            name: "Error".to_string(),
            fields: vec![
                ProtoField {
                    name: "code".to_string(),
                    field_type: ProtoFieldType::Int32,
                    number: 1,
                    json_name: Some("code".to_string()),
                    comment: Some("Error code".to_string()),
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                },
                ProtoField {
                    name: "message".to_string(),
                    field_type: ProtoFieldType::String,
                    number: 2,
                    json_name: Some("message".to_string()),
                    comment: Some("Error message".to_string()),
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                },
                ProtoField {
                    name: "data".to_string(),
                    field_type: ProtoFieldType::String,
                    number: 3,
                    json_name: Some("data".to_string()),
                    comment: Some("Additional error data as JSON string".to_string()),
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                },
            ],
            nested_messages: vec![],
            nested_enums: vec![],
            oneofs: vec![],
            comment: Some("Standard error message for failed requests".to_string()),
            options: vec![],
        }
    }
}

// Helper function to resolve the recursion issue
fn schema_to_proto_field_type_impl(schema: &Schema) -> Result<ProtoFieldType> {
    match schema {
        Schema::Primitive(primitive) => match primitive {
            Primitive::String(str_prim) => {
                if str_prim.r#enum.is_some() {
                    // This should be handled as an enum type
                    Ok(ProtoFieldType::Enum("String".to_string()))
                } else {
                    Ok(ProtoFieldType::String)
                }
            }
            Primitive::Integer(_) => Ok(ProtoFieldType::Int64),
            Primitive::Boolean(_) => Ok(ProtoFieldType::Bool),
            Primitive::Array(array) => {
                // Return the inner type - the repeated flag will be set separately
                schema_to_proto_field_type_impl(&array.items)
            }
            Primitive::Object(_) => Ok(ProtoFieldType::Message("starknet.v0_8_1.common.Object".to_string())),
        },
        Schema::Ref(reference) => {
            let ref_name = reference.name();
            
            // Handle common type aliases that should map to primitive types
            match ref_name {
                "FELT" | "TXN_HASH" | "BLOCK_HASH" | "ADDRESS" | "CLASS_HASH" | "STORAGE_KEY" | "HASH_256" => {
                    Ok(ProtoFieldType::String)
                }
                "BLOCK_NUMBER" | "NUM_AS_HEX" | "L1_TXN_HASH" => {
                    Ok(ProtoFieldType::Uint64)
                }
                "u64" => {
                    Ok(ProtoFieldType::Uint64)
                }
                "u128" => {
                    Ok(ProtoFieldType::String) // Use string for large integers
                }
                "ETH_ADDRESS" => {
                    Ok(ProtoFieldType::String)
                }
                // Handle generic object types
                "Object" => {
                    // Try to determine what kind of object this should be based on context
                    // For now, we'll use a generic Object type, but this should be improved
                    Ok(ProtoFieldType::Message("starknet.v0_8_1.common.Object".to_string()))
                }
                // Handle cross-service type references
                "NestedCall" | "NESTED_CALL" => {
                    Ok(ProtoFieldType::Message("starknet.v0_8_1.common.FunctionInvocation".to_string())) // NestedCall is an alias for FunctionInvocation
                }
                "BroadcastedInvokeTxn" | "BROADCASTED_INVOKE_TXN" | "BROADCASTED_INVOKE_TXN_V3" => {
                    Ok(ProtoFieldType::Message("starknet.v0_8_1.common.InvokeTxnV3Content".to_string())) // Map to the actual content type
                }
                "BroadcastedDeclareTxn" | "BROADCASTED_DECLARE_TXN" | "BROADCASTED_DECLARE_TXN_V3" => {
                    Ok(ProtoFieldType::Message("starknet.v0_8_1.common.DeclareTxnV3Content".to_string())) // Map to the actual content type
                }
                "BroadcastedDeployAccountTxn" | "BROADCASTED_DEPLOY_ACCOUNT_TXN" | "BROADCASTED_DEPLOY_ACCOUNT_TXN_V3" => {
                    Ok(ProtoFieldType::Message("starknet.v0_8_1.common.DeployAccountTxnV3Content".to_string())) // Map to the actual content type
                }
                // For other references, treat as message types with qualified names
                _ => {
                    let proto_type_name = to_proto_type_name(ref_name);
                    Ok(ProtoFieldType::Message(format!("starknet.v0_8_1.common.{}", proto_type_name)))
                }
            }
        }
        Schema::OneOf(_) => Ok(ProtoFieldType::Message("starknet.v0_8_1.common.Object".to_string())),
        Schema::AllOf(_) => Ok(ProtoFieldType::Message("starknet.v0_8_1.common.AllOf".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProtoConfig {
        ProtoConfig::new("v0_1_0")
    }

    fn create_test_method(name: &str, params: Vec<Param>, has_result: bool) -> Method {
        Method {
            name: name.to_string(),
            summary: format!("Summary for {}", name),
            description: Some(format!("Description for {}", name)),
            param_structure: None,
            params,
            result: if has_result {
                Some(MethodResult {
                    name: "result".to_string(),
                    description: Some("The result".to_string()),
                    required: Some(true),
                    schema: Schema::Primitive(Primitive::String(StringPrimitive {
                        title: None,
                        comment: None,
                        description: Some("Result value".to_string()),
                        r#enum: None,
                        pattern: None,
                    })),
                    summary: None,
                })
            } else {
                None
            },
            errors: None,
        }
    }

    fn create_test_param(name: &str, required: bool) -> Param {
        Param {
            name: name.to_string(),
            description: Some(format!("Parameter {}", name)),
            summary: None,
            required,
            schema: Schema::Primitive(Primitive::String(StringPrimitive {
                title: None,
                comment: None,
                description: Some("String parameter".to_string()),
                r#enum: None,
                pattern: None,
            })),
        }
    }

    #[test]
    fn test_method_name_to_rpc_name() {
        let config = create_test_config();
        let generator = ServiceGenerator::new("TestService", &config);
        
        assert_eq!(generator.method_name_to_rpc_name("starknet_getBlock"), "GetBlock");
        assert_eq!(generator.method_name_to_rpc_name("starknet_getBlockWithTxHashes"), "GetBlockWithTxHashes");
        assert_eq!(generator.method_name_to_rpc_name("starknet_addInvokeTransaction"), "AddInvokeTransaction");
        assert_eq!(generator.method_name_to_rpc_name("trace_transaction"), "TraceTransaction");
        assert_eq!(generator.method_name_to_rpc_name("custom_method_name"), "CustomMethodName");
    }

    #[test]
    fn test_snake_to_pascal_case() {
        let config = create_test_config();
        let generator = ServiceGenerator::new("TestService", &config);
        
        assert_eq!(generator.snake_to_pascal_case("get_block"), "GetBlock");
        assert_eq!(generator.snake_to_pascal_case("get_block_with_tx_hashes"), "GetBlockWithTxHashes");
        assert_eq!(generator.snake_to_pascal_case("add_invoke_transaction"), "AddInvokeTransaction");
        assert_eq!(generator.snake_to_pascal_case("simple"), "Simple");
        assert_eq!(generator.snake_to_pascal_case("trace_block_transactions"), "TraceBlockTransactions");
    }

    #[test]
    fn test_determine_streaming_type() {
        let config = create_test_config();
        let generator = ServiceGenerator::new("TestService", &config);
        
        // Test non-streaming methods
        let method = create_test_method("starknet_getBlock", vec![], true);
        let (client_streaming, server_streaming) = generator.determine_streaming_type(&method);
        assert!(!client_streaming);
        assert!(!server_streaming);
        
        // Test methods with many parameters (should be client streaming)
        let method = create_test_method("starknet_getBatch", 
            (0..6).map(|i| create_test_param(&format!("param{}", i), true)).collect(),
            true
        );
        let (client_streaming, server_streaming) = generator.determine_streaming_type(&method);
        assert!(client_streaming);
        assert!(!server_streaming);
        
        // Test subscription methods (should be server streaming)
        let method = create_test_method("starknet_subscribeEvents", vec![], true);
        let (client_streaming, server_streaming) = generator.determine_streaming_type(&method);
        assert!(!client_streaming);
        assert!(server_streaming);
        
        // Test streaming methods
        let method = create_test_method("starknet_streamLogs", vec![], true);
        let (client_streaming, server_streaming) = generator.determine_streaming_type(&method);
        assert!(!client_streaming);
        assert!(server_streaming);
    }

    #[test]
    fn test_convert_method_to_rpc() {
        let config = create_test_config();
        let generator = ServiceGenerator::new("StarknetService", &config);
        
        let method = create_test_method(
            "starknet_getBlock",
            vec![
                create_test_param("block_id", true),
                create_test_param("include_txs", false),
            ],
            true
        );
        
        let rpc = generator.convert_method_to_rpc(&method).unwrap();
        
        assert_eq!(rpc.name, "GetBlock");
        assert_eq!(rpc.request_type, "GetBlockRequest");
        assert_eq!(rpc.response_type, "GetBlockResponse");
        assert!(rpc.comment.is_some());
        assert!(!rpc.client_streaming);
        assert!(!rpc.server_streaming);
    }

    #[test]
    fn test_generate_request_message() {
        let config = create_test_config();
        let generator = ServiceGenerator::new("StarknetService", &config);
        
        let method = create_test_method(
            "starknet_getBlock",
            vec![
                create_test_param("block_id", true),
                create_test_param("include_txs", false),
            ],
            true
        );
        
        let request = generator.generate_request_message("GetBlock", &method).unwrap();
        
        assert_eq!(request.name, "GetBlockRequest");
        assert_eq!(request.fields.len(), 2);
        assert_eq!(request.comment, Some("Request message for starknet_getBlock".to_string()));
        
        let block_id_field = &request.fields[0];
        assert_eq!(block_id_field.name, "block_id");
        assert_eq!(block_id_field.json_name, Some("block_id".to_string()));
        assert!(!block_id_field.optional); // Required parameter
        
        let include_txs_field = &request.fields[1];
        assert_eq!(include_txs_field.name, "include_txs");
        assert_eq!(include_txs_field.json_name, Some("include_txs".to_string()));
        assert!(include_txs_field.optional); // Not required
    }

    #[test]
    fn test_generate_response_message() {
        let config = create_test_config();
        let generator = ServiceGenerator::new("StarknetService", &config);
        
        let method = create_test_method("starknet_getBlock", vec![], true);
        
        let response = generator.generate_response_message("GetBlock", &method).unwrap();
        
        assert_eq!(response.name, "GetBlockResponse");
        assert_eq!(response.fields.len(), 2); // result + error
        assert_eq!(response.comment, Some("Response message for starknet_getBlock".to_string()));
        
        let result_field = &response.fields[0];
        assert_eq!(result_field.name, "result");
        assert_eq!(result_field.json_name, Some("result".to_string()));
        assert!(!result_field.optional);
        
        let error_field = &response.fields[1];
        assert_eq!(error_field.name, "error");
        assert_eq!(error_field.json_name, Some("error".to_string()));
        assert!(error_field.optional);
        assert!(matches!(error_field.field_type, ProtoFieldType::Message(ref name) if name == "Error"));
    }

    #[test]
    fn test_generate_from_methods() {
        let config = create_test_config();
        let generator = ServiceGenerator::new("StarknetMainService", &config);
        
        let methods = vec![
            create_test_method("starknet_getBlock", vec![create_test_param("block_id", true)], true),
            create_test_method("starknet_getTransaction", vec![create_test_param("txn_hash", true)], true),
            create_test_method("starknet_chainId", vec![], true),
        ];
        
        let method_refs: Vec<&Method> = methods.iter().collect();
        let service = generator.generate_from_methods(&method_refs).unwrap();
        
        assert_eq!(service.name, "StarknetMainService");
        assert_eq!(service.rpcs.len(), 3);
        assert!(service.comment.is_some());
        
        assert_eq!(service.rpcs[0].name, "GetBlock");
        assert_eq!(service.rpcs[1].name, "GetTransaction");
        assert_eq!(service.rpcs[2].name, "ChainId");
    }

    #[test]
    fn test_generate_error_message() {
        let error_message = ServiceGenerator::generate_error_message();
        
        assert_eq!(error_message.name, "Error");
        assert_eq!(error_message.fields.len(), 3);
        
        let code_field = &error_message.fields[0];
        assert_eq!(code_field.name, "code");
        assert!(matches!(code_field.field_type, ProtoFieldType::Int32));
        assert!(!code_field.optional);
        
        let message_field = &error_message.fields[1];
        assert_eq!(message_field.name, "message");
        assert!(matches!(message_field.field_type, ProtoFieldType::String));
        assert!(!message_field.optional);
        
        let data_field = &error_message.fields[2];
        assert_eq!(data_field.name, "data");
        assert!(matches!(data_field.field_type, ProtoFieldType::String));
        assert!(data_field.optional);
    }

    #[test]
    fn test_schema_to_proto_field_type() {
        let config = create_test_config();
        let generator = ServiceGenerator::new("TestService", &config);
        
        // Test string schema
        let string_schema = Schema::Primitive(Primitive::String(StringPrimitive {
            title: None,
            comment: None,
            description: None,
            r#enum: None,
            pattern: None,
        }));
        let field_type = generator.schema_to_proto_field_type(&string_schema).unwrap();
        assert!(matches!(field_type, ProtoFieldType::String));
        
        // Test string enum schema
        let enum_schema = Schema::Primitive(Primitive::String(StringPrimitive {
            title: None,
            comment: None,
            description: None,
            r#enum: Some(vec!["value1".to_string(), "value2".to_string()]),
            pattern: None,
        }));
        let field_type = generator.schema_to_proto_field_type(&enum_schema).unwrap();
        assert!(matches!(field_type, ProtoFieldType::Enum(_)));
        
        // Test array schema
        let array_schema = Schema::Primitive(Primitive::Array(ArrayPrimitive {
            title: None,
            description: None,
            items: Box::new(Schema::Primitive(Primitive::String(StringPrimitive {
                title: None,
                comment: None,
                description: None,
                r#enum: None,
                pattern: None,
            }))),
        }));
        let field_type = generator.schema_to_proto_field_type(&array_schema).unwrap();
        assert!(matches!(field_type, ProtoFieldType::String)); // Inner type, repeated will be set separately
        
        // Test reference schema
        let ref_schema = Schema::Ref(Reference {
            title: None,
            comment: None,
            description: None,
            ref_field: "#/components/schemas/BLOCK".to_string(),
            additional_fields: std::collections::HashMap::new(),
        });
        let field_type = generator.schema_to_proto_field_type(&ref_schema).unwrap();
        if let ProtoFieldType::Message(type_name) = field_type {
            assert_eq!(type_name, "Block");
        } else {
            panic!("Expected Message type");
        }
    }

    #[test]
    fn test_generate_request_response_messages() {
        let config = create_test_config();
        let generator = ServiceGenerator::new("StarknetService", &config);
        
        let method = create_test_method(
            "starknet_getBlock",
            vec![create_test_param("block_id", true)],
            true
        );
        
        let (request, response) = generator.generate_request_response_messages(&method).unwrap();
        
        assert_eq!(request.name, "GetBlockRequest");
        assert_eq!(response.name, "GetBlockResponse");
        
        assert_eq!(request.fields.len(), 1);
        assert_eq!(response.fields.len(), 2); // result + error
        
        assert_eq!(request.fields[0].name, "block_id");
        assert_eq!(response.fields[0].name, "result");
        assert_eq!(response.fields[1].name, "error");
    }
} 