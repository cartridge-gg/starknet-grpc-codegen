use std::fmt;

use crate::proto_gen::types::{ProtoMessage, ProtoEnum, ProtoService, ProtoField};

/// Writer for generating formatted protobuf files
pub struct ProtoWriter {
    package: String,
    imports: Vec<String>,
    messages: Vec<ProtoMessage>,
    enums: Vec<ProtoEnum>,
    services: Vec<ProtoService>,
    options: Vec<String>,
}

impl ProtoWriter {
    pub fn new(package: &str) -> Self {
        Self {
            package: package.to_string(),
            imports: Vec::new(),
            messages: Vec::new(),
            enums: Vec::new(),
            services: Vec::new(),
            options: vec![
                // Java options
                "java_multiple_files = true".to_string(),
                "java_outer_classname = \"StarknetProto\"".to_string(),
                format!("java_package = \"com.{}\"", package.replace('.', "_")),
                
                // Go options
                format!("go_package = \"github.com/cartridge-gg/starknet-grpc-codegen/go/{}\"", package.replace('.', "/")),
                
                // C# options (useful for .NET ecosystem)
                format!("csharp_namespace = \"Starknet.{}\";", package.split('.').map(|s| {
                    let mut chars = s.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                }).collect::<Vec<_>>().join(".")),
                
                // PHP options
                format!("php_namespace = \"Starknet\\\\{}\";", package.split('.').map(|s| {
                    let mut chars = s.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                }).collect::<Vec<_>>().join("\\\\")),
            ],
        }
    }

    pub fn add_import(&mut self, import: &str) {
        if !self.imports.contains(&import.to_string()) {
            self.imports.push(import.to_string());
        }
    }

    pub fn add_message(&mut self, message: &ProtoMessage) {
        self.messages.push(message.clone());
    }

    pub fn add_enum(&mut self, proto_enum: &ProtoEnum) {
        self.enums.push(proto_enum.clone());
    }

    pub fn add_service(&mut self, service: &ProtoService) {
        self.services.push(service.clone());
    }

    #[allow(dead_code)]
    pub fn add_option(&mut self, option: &str) {
        self.options.push(option.to_string());
    }
}

impl fmt::Display for ProtoWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Proto file header
        writeln!(f, "syntax = \"proto3\";")?;
        writeln!(f)?;
        
        // Package declaration
        writeln!(f, "package {};", self.package)?;
        writeln!(f)?;
        
        // Imports
        for import in &self.imports {
            writeln!(f, "import \"{}\";", import)?;
        }
        if !self.imports.is_empty() {
            writeln!(f)?;
        }
        
        // Options
        for option in &self.options {
            writeln!(f, "option {};", option)?;
        }
        if !self.options.is_empty() {
            writeln!(f)?;
        }
        
        // Enums
        for proto_enum in &self.enums {
            write!(f, "{}", proto_enum)?;
            writeln!(f)?;
        }
        
        // Messages
        for message in &self.messages {
            write!(f, "{}", message)?;
            writeln!(f)?;
        }
        
        // Services
        for service in &self.services {
            write!(f, "{}", service)?;
            writeln!(f)?;
        }
        
        Ok(())
    }
}

/// Helper functions for proto formatting
pub fn format_comment(text: &str, indent: usize) -> String {
    let prefix = " ".repeat(indent);
    text.lines()
        .map(|line| format!("{}// {}", prefix, line.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(dead_code)]
pub fn format_field_number(fields: &[ProtoField]) -> Vec<ProtoField> {
    fields.iter()
        .enumerate()
        .map(|(i, field)| {
            let mut field = field.clone();
            field.number = i as u32 + 1;
            field
        })
        .collect()
}

pub fn to_proto_name(json_name: &str) -> String {
    // Convert camelCase to snake_case for proto field names
    let mut result = String::new();
    let mut prev_char_was_lowercase = false;
    
    for ch in json_name.chars() {
        if ch.is_uppercase() && prev_char_was_lowercase {
            result.push('_');
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch.to_lowercase().next().unwrap());
        }
        prev_char_was_lowercase = ch.is_lowercase();
    }
    
    // Handle special cases
    match result.as_str() {
        "type" => "type_".to_string(),
        "ref" => "ref_".to_string(),
        _ => result,
    }
}

pub fn to_proto_type_name(json_name: &str) -> String {
    // Convert to PascalCase for message names
    let parts: Vec<&str> = json_name.split('_').collect();
    parts.iter()
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn escape_proto_keyword(name: &str) -> String {
    match name {
        "import" | "package" | "message" | "enum" | "service" | "rpc" | 
        "returns" | "stream" | "option" | "extend" | "extensions" | 
        "reserved" | "syntax" => format!("{}_", name),
        _ => name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto_gen::types::*;

    #[test]
    fn test_proto_writer_basic_structure() {
        let writer = ProtoWriter::new("starknet.v0_1_0.test");
        
        let output = writer.to_string();
        assert!(output.contains("syntax = \"proto3\";"));
        assert!(output.contains("package starknet.v0_1_0.test;"));
        assert!(output.contains("option java_multiple_files = true;"));
        assert!(output.contains("option java_outer_classname = \"StarknetProto\";"));
    }

    #[test]
    fn test_proto_writer_with_imports() {
        let mut writer = ProtoWriter::new("test.package");
        writer.add_import("google/protobuf/any.proto");
        writer.add_import("google/protobuf/timestamp.proto");
        
        let output = writer.to_string();
        assert!(output.contains("import \"google/protobuf/any.proto\";"));
        assert!(output.contains("import \"google/protobuf/timestamp.proto\";"));
    }

    #[test]
    fn test_proto_writer_with_message() {
        let mut writer = ProtoWriter::new("test.package");
        
        let message = ProtoMessage {
            name: "TestMessage".to_string(),
            fields: vec![
                ProtoField {
                    name: "id".to_string(),
                    field_type: ProtoFieldType::Int64,
                    number: 1,
                    json_name: Some("id".to_string()),
                    comment: None,
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                },
            ],
            nested_messages: vec![],
            nested_enums: vec![],
            oneofs: vec![],
            comment: None,
            options: vec![],
        };
        
        writer.add_message(&message);
        let output = writer.to_string();
        
        assert!(output.contains("message TestMessage {"));
        assert!(output.contains("int64 id = 1 [json_name = \"id\"];"));
    }

    #[test]
    fn test_proto_writer_with_enum() {
        let mut writer = ProtoWriter::new("test.package");
        
        let proto_enum = ProtoEnum {
            name: "Status".to_string(),
            values: vec![
                ProtoEnumValue {
                    name: "UNKNOWN".to_string(),
                    number: 0,
                    comment: None,
                },
                ProtoEnumValue {
                    name: "ACTIVE".to_string(),
                    number: 1,
                    comment: None,
                },
            ],
            comment: None,
        };
        
        writer.add_enum(&proto_enum);
        let output = writer.to_string();
        
        assert!(output.contains("enum Status {"));
        assert!(output.contains("UNKNOWN = 0;"));
        assert!(output.contains("ACTIVE = 1;"));
    }

    #[test]
    fn test_proto_writer_with_service() {
        use crate::proto_gen::types::ProtoRpc;
        
        let mut writer = ProtoWriter::new("test.package");
        
        let service = ProtoService {
            name: "TestService".to_string(),
            rpcs: vec![
                ProtoRpc {
                    name: "GetData".to_string(),
                    request_type: "GetDataRequest".to_string(),
                    response_type: "GetDataResponse".to_string(),
                    comment: None,
                    client_streaming: false,
                    server_streaming: false,
                },
            ],
            comment: None,
        };
        
        writer.add_service(&service);
        let output = writer.to_string();
        
        assert!(output.contains("service TestService {"));
        assert!(output.contains("rpc GetData(GetDataRequest) returns (GetDataResponse);"));
    }

    #[test]
    fn test_to_proto_name_conversion() {
        assert_eq!(to_proto_name("camelCase"), "camel_case");
        assert_eq!(to_proto_name("PascalCase"), "pascal_case");
        assert_eq!(to_proto_name("snake_case"), "snake_case");
        assert_eq!(to_proto_name("type"), "type_");
        assert_eq!(to_proto_name("ref"), "ref_");
        assert_eq!(to_proto_name("blockNumber"), "block_number");
        assert_eq!(to_proto_name("transactionHash"), "transaction_hash");
    }

    #[test]
    fn test_to_proto_type_name_conversion() {
        assert_eq!(to_proto_type_name("BLOCK_HASH"), "BlockHash");
        assert_eq!(to_proto_type_name("TXN_HASH"), "TxnHash");
        assert_eq!(to_proto_type_name("FELT"), "Felt");
        assert_eq!(to_proto_type_name("EVENT_FILTER"), "EventFilter");
        assert_eq!(to_proto_type_name("simple"), "Simple");
    }

    #[test]
    fn test_escape_proto_keyword() {
        assert_eq!(escape_proto_keyword("import"), "import_");
        assert_eq!(escape_proto_keyword("package"), "package_");
        assert_eq!(escape_proto_keyword("message"), "message_");
        assert_eq!(escape_proto_keyword("normal"), "normal");
        assert_eq!(escape_proto_keyword("field"), "field");
    }

    #[test]
    fn test_format_comment() {
        let comment = "This is a test comment\nwith multiple lines\nand proper formatting";
        let formatted = format_comment(comment, 2);
        
        assert!(formatted.contains("  // This is a test comment"));
        assert!(formatted.contains("  // with multiple lines"));
        assert!(formatted.contains("  // and proper formatting"));
    }

    #[test]
    fn test_complete_proto_file_structure() {
        use crate::proto_gen::types::ProtoRpc;
        
        let mut writer = ProtoWriter::new("starknet.v0_1_0.main");
        
        // Add imports
        writer.add_import("google/protobuf/any.proto");
        writer.add_import("starknet/v0_1_0/common.proto");
        
        // Add enum
        let status_enum = ProtoEnum {
            name: "TransactionStatus".to_string(),
            values: vec![
                ProtoEnumValue { name: "PENDING".to_string(), number: 0, comment: None },
                ProtoEnumValue { name: "ACCEPTED".to_string(), number: 1, comment: None },
                ProtoEnumValue { name: "REJECTED".to_string(), number: 2, comment: None },
            ],
            comment: Some("Transaction status enumeration".to_string()),
        };
        writer.add_enum(&status_enum);
        
        // Add message
        let transaction_msg = ProtoMessage {
            name: "Transaction".to_string(),
            fields: vec![
                ProtoField {
                    name: "hash".to_string(),
                    field_type: ProtoFieldType::String,
                    number: 1,
                    json_name: Some("transaction_hash".to_string()),
                    comment: Some("Transaction hash".to_string()),
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                },
                ProtoField {
                    name: "status".to_string(),
                    field_type: ProtoFieldType::Enum("TransactionStatus".to_string()),
                    number: 2,
                    json_name: Some("status".to_string()),
                    comment: None,
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                },
            ],
            nested_messages: vec![],
            nested_enums: vec![],
            oneofs: vec![],
            comment: Some("Transaction message".to_string()),
            options: vec![],
        };
        writer.add_message(&transaction_msg);
        
        // Add service
        let service = ProtoService {
            name: "StarknetService".to_string(),
            rpcs: vec![
                ProtoRpc {
                    name: "GetTransaction".to_string(),
                    request_type: "GetTransactionRequest".to_string(),
                    response_type: "Transaction".to_string(),
                    comment: Some("Get transaction by hash".to_string()),
                    client_streaming: false,
                    server_streaming: false,
                },
            ],
            comment: Some("Main Starknet service".to_string()),
        };
        writer.add_service(&service);
        
        let output = writer.to_string();
        
        // Verify structure order
        let lines: Vec<&str> = output.lines().collect();
        let mut syntax_found = false;
        let mut package_found = false;
        let mut imports_found = false;
        let mut options_found = false;
        let mut enum_found = false;
        let mut message_found = false;
        let mut service_found = false;
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("syntax") {
                syntax_found = true;
                assert!(!package_found);
            } else if trimmed.starts_with("package") {
                package_found = true;
                assert!(syntax_found);
            } else if trimmed.starts_with("import") {
                imports_found = true;
                assert!(package_found);
            } else if trimmed.starts_with("option") {
                options_found = true;
                assert!(package_found);
            } else if trimmed.starts_with("enum") {
                enum_found = true;
                assert!(syntax_found && package_found);
            } else if trimmed.starts_with("message") {
                message_found = true;
                assert!(enum_found);
            } else if trimmed.starts_with("service") {
                service_found = true;
                assert!(message_found);
            }
        }
        
        assert!(syntax_found && package_found && imports_found && 
                options_found && enum_found && message_found && service_found);
    }
} 