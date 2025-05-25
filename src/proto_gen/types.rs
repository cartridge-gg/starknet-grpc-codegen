use std::collections::{HashMap, HashSet};
use std::fmt;

use anyhow::Result;

use crate::spec::*;
use crate::proto_gen::{ProtoConfig, writer::*};

/// Protobuf message definition
#[derive(Debug, Clone)]
pub struct ProtoMessage {
    pub name: String,
    pub fields: Vec<ProtoField>,
    #[allow(dead_code)]
    pub nested_messages: Vec<ProtoMessage>,
    #[allow(dead_code)]
    pub nested_enums: Vec<ProtoEnum>,
    pub oneofs: Vec<ProtoOneof>,
    pub comment: Option<String>,
    #[allow(dead_code)]
    pub options: Vec<String>,
}

/// Protobuf field definition
#[derive(Debug, Clone)]
pub struct ProtoField {
    pub name: String,
    pub field_type: ProtoFieldType,
    pub number: u32,
    pub json_name: Option<String>,
    pub comment: Option<String>,
    pub optional: bool,
    pub repeated: bool,
    pub oneof_name: Option<String>,
}

/// Protobuf field types
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ProtoFieldType {
    String,
    Int32,
    Int64,
    Uint32,
    Uint64,
    Bool,
    Bytes,
    Double,
    Float,
    Message(String),
    Enum(String),
    Any,
}

/// Protobuf oneof definition
#[derive(Debug, Clone)]
pub struct ProtoOneof {
    pub name: String,
    pub fields: Vec<ProtoField>,
    #[allow(dead_code)]
    pub comment: Option<String>,
}

/// Protobuf enum definition
#[derive(Debug, Clone)]
pub struct ProtoEnum {
    pub name: String,
    pub values: Vec<ProtoEnumValue>,
    pub comment: Option<String>,
}

/// Protobuf enum value
#[derive(Debug, Clone)]
pub struct ProtoEnumValue {
    pub name: String,
    pub number: i32,
    pub comment: Option<String>,
}

/// Protobuf service definition
#[derive(Debug, Clone)]
pub struct ProtoService {
    pub name: String,
    pub rpcs: Vec<ProtoRpc>,
    pub comment: Option<String>,
}

/// Protobuf RPC definition
#[derive(Debug, Clone)]
pub struct ProtoRpc {
    pub name: String,
    pub request_type: String,
    pub response_type: String,
    pub comment: Option<String>,
    pub client_streaming: bool,
    pub server_streaming: bool,
}

/// Type resolution result
#[derive(Debug, Clone)]
pub struct TypeResolution {
    pub common_types: Vec<ProtoMessage>,
    pub common_enums: Vec<ProtoEnum>,
    pub main_types: Vec<ProtoMessage>,
    pub write_types: Vec<ProtoMessage>,
    pub trace_types: Vec<ProtoMessage>,
    pub ws_types: Vec<ProtoMessage>,
    #[allow(dead_code)]
    pub type_map: HashMap<String, String>, // JSON schema name -> Proto type name
}

/// Type resolver for converting JSON schemas to protobuf types
pub struct TypeResolver {
    #[allow(dead_code)]
    config: ProtoConfig,
    resolved_types: HashMap<String, ProtoMessage>,
    resolved_enums: HashMap<String, ProtoEnum>,
    type_dependencies: HashMap<String, HashSet<String>>,
}

// Type aliases to reduce complexity
type OrganizedTypes = ((Vec<ProtoMessage>, Vec<ProtoEnum>), HashMap<String, Vec<ProtoMessage>>);

impl TypeResolver {
    pub fn new(config: &ProtoConfig) -> Self {
        Self {
            config: config.clone(),
            resolved_types: HashMap::new(),
            resolved_enums: HashMap::new(),
            type_dependencies: HashMap::new(),
        }
    }

    pub fn resolve_types(&mut self, specs: &Specification) -> Result<TypeResolution> {
        // First pass: collect all type names
        for (name, _schema) in &specs.components.schemas {
            self.register_type_name(name);
        }

        // Second pass: resolve all types
        for (name, schema) in &specs.components.schemas {
            self.resolve_schema_type(name, schema)?;
        }

        // Third pass: organize types by service
        let (common_types, service_types) = self.organize_types_by_service(specs)?;
        
        Ok(TypeResolution {
            common_types: common_types.0,
            common_enums: common_types.1,
            main_types: service_types.get("main").cloned().unwrap_or_default(),
            write_types: service_types.get("write").cloned().unwrap_or_default(),
            trace_types: service_types.get("trace").cloned().unwrap_or_default(),
            ws_types: service_types.get("ws").cloned().unwrap_or_default(),
            type_map: self.build_type_map(),
        })
    }

    fn register_type_name(&mut self, name: &str) {
        // Register the type name for reference resolution
        let _proto_name = to_proto_type_name(name);
        self.type_dependencies.insert(name.to_string(), HashSet::new());
    }

    fn resolve_schema_type(&mut self, name: &str, schema: &Schema) -> Result<()> {
        let proto_name = to_proto_type_name(name);
        
        match schema {
            Schema::Primitive(Primitive::Object(obj)) => {
                let message = self.convert_object_to_message(&proto_name, obj)?;
                self.resolved_types.insert(name.to_string(), message);
            }
            Schema::Primitive(Primitive::String(str_primitive)) => {
                if let Some(enum_values) = &str_primitive.r#enum {
                    let proto_enum = self.convert_string_enum_to_enum(&proto_name, enum_values, str_primitive.description.as_deref())?;
                    self.resolved_enums.insert(name.to_string(), proto_enum);
                } else {
                    // String primitive without enum - create alias or wrapper
                    let message = ProtoMessage {
                        name: proto_name,
                        fields: vec![ProtoField {
                            name: "value".to_string(),
                            field_type: ProtoFieldType::String,
                            number: 1,
                            json_name: None,
                            comment: str_primitive.description.clone(),
                            optional: false,
                            repeated: false,
                            oneof_name: None,
                        }],
                        nested_messages: vec![],
                        nested_enums: vec![],
                        oneofs: vec![],
                        comment: str_primitive.description.clone(),
                        options: vec![],
                    };
                    self.resolved_types.insert(name.to_string(), message);
                }
            }
            Schema::OneOf(oneof) => {
                let message = self.convert_oneof_to_message(&proto_name, oneof)?;
                self.resolved_types.insert(name.to_string(), message);
            }
            Schema::AllOf(allof) => {
                let message = self.convert_allof_to_message(&proto_name, allof)?;
                self.resolved_types.insert(name.to_string(), message);
            }
            Schema::Ref(_) => {
                // Reference types are handled during field resolution
            }
            _ => {
                // Handle other primitive types
                let message = self.convert_primitive_to_wrapper(&proto_name, schema)?;
                self.resolved_types.insert(name.to_string(), message);
            }
        }

        Ok(())
    }

    fn convert_object_to_message(&self, name: &str, obj: &ObjectPrimitive) -> Result<ProtoMessage> {
        let mut fields = Vec::new();
        let mut field_number = 1u32;

        for (field_name, field_schema) in &obj.properties {
            let proto_field_name = to_proto_name(field_name);
            let field_type = self.schema_to_proto_field_type(field_schema)?;
            let is_required = obj.required.contains(field_name);

            fields.push(ProtoField {
                name: proto_field_name,
                field_type,
                number: field_number,
                json_name: Some(field_name.clone()),
                comment: field_schema.description().cloned(),
                optional: !is_required,
                repeated: false,
                oneof_name: None,
            });

            field_number += 1;
        }

        Ok(ProtoMessage {
            name: name.to_string(),
            fields,
            nested_messages: vec![],
            nested_enums: vec![],
            oneofs: vec![],
            comment: obj.description.clone(),
            options: vec![],
        })
    }

    fn convert_oneof_to_message(&self, name: &str, oneof: &OneOf) -> Result<ProtoMessage> {
        let mut oneof_fields = Vec::new();
        let mut field_number = 1u32;

        for (i, variant_schema) in oneof.one_of.iter().enumerate() {
            let variant_name = format!("variant_{}", i + 1);
            let field_type = self.schema_to_proto_field_type(variant_schema)?;

            oneof_fields.push(ProtoField {
                name: variant_name,
                field_type,
                number: field_number,
                json_name: None,
                comment: variant_schema.description().cloned(),
                optional: false,
                repeated: false,
                oneof_name: Some("value".to_string()),
            });

            field_number += 1;
        }

        let proto_oneof = ProtoOneof {
            name: "value".to_string(),
            fields: oneof_fields.clone(),
            comment: None,
        };

        Ok(ProtoMessage {
            name: name.to_string(),
            fields: oneof_fields,
            nested_messages: vec![],
            nested_enums: vec![],
            oneofs: vec![proto_oneof],
            comment: oneof.description.clone(),
            options: vec![],
        })
    }

    fn convert_allof_to_message(&self, name: &str, allof: &AllOf) -> Result<ProtoMessage> {
        // For allOf, we'll flatten all fields into a single message
        let mut all_fields = Vec::new();
        let mut field_number = 1u32;

        for schema in &allof.all_of {
            match schema {
                Schema::Primitive(Primitive::Object(obj)) => {
                    for (field_name, field_schema) in &obj.properties {
                        let proto_field_name = to_proto_name(field_name);
                        let field_type = self.schema_to_proto_field_type(field_schema)?;
                        let is_required = obj.required.contains(field_name);

                        all_fields.push(ProtoField {
                            name: proto_field_name,
                            field_type,
                            number: field_number,
                            json_name: Some(field_name.clone()),
                            comment: field_schema.description().cloned(),
                            optional: !is_required,
                            repeated: false,
                            oneof_name: None,
                        });

                        field_number += 1;
                    }
                }
                Schema::Ref(reference) => {
                    // Handle reference in allOf by creating a field of that type
                    let ref_name = reference.name();
                    let field_type = ProtoFieldType::Message(to_proto_type_name(ref_name));
                    
                    all_fields.push(ProtoField {
                        name: to_proto_name(ref_name),
                        field_type,
                        number: field_number,
                        json_name: None,
                        comment: reference.description.clone(),
                        optional: false,
                        repeated: false,
                        oneof_name: None,
                    });

                    field_number += 1;
                }
                _ => {
                    // Handle other schema types in allOf
                }
            }
        }

        Ok(ProtoMessage {
            name: name.to_string(),
            fields: all_fields,
            nested_messages: vec![],
            nested_enums: vec![],
            oneofs: vec![],
            comment: allof.description.clone(),
            options: vec![],
        })
    }

    fn convert_string_enum_to_enum(&self, name: &str, values: &[String], description: Option<&str>) -> Result<ProtoEnum> {
        let mut enum_values = Vec::new();
        
        for (i, value) in values.iter().enumerate() {
            let enum_value_name = value.to_uppercase().replace("-", "_");
            enum_values.push(ProtoEnumValue {
                name: enum_value_name,
                number: i as i32,
                comment: None,
            });
        }

        Ok(ProtoEnum {
            name: name.to_string(),
            values: enum_values,
            comment: description.map(|s| s.to_string()),
        })
    }

    fn convert_primitive_to_wrapper(&self, name: &str, schema: &Schema) -> Result<ProtoMessage> {
        let field_type = self.schema_to_proto_field_type(schema)?;
        
        Ok(ProtoMessage {
            name: name.to_string(),
            fields: vec![ProtoField {
                name: "value".to_string(),
                field_type,
                number: 1,
                json_name: None,
                comment: schema.description().cloned(),
                optional: false,
                repeated: false,
                oneof_name: None,
            }],
            nested_messages: vec![],
            nested_enums: vec![],
            oneofs: vec![],
            comment: schema.description().cloned(),
            options: vec![],
        })
    }

    fn schema_to_proto_field_type(&self, schema: &Schema) -> Result<ProtoFieldType> {
        schema_to_proto_field_type_impl(schema)
    }

    fn organize_types_by_service(&self, _specs: &Specification) -> Result<OrganizedTypes> {
        // For now, put all types in common
        let common_messages: Vec<ProtoMessage> = self.resolved_types.values().cloned().collect();
        let common_enums: Vec<ProtoEnum> = self.resolved_enums.values().cloned().collect();
        
        let service_types = HashMap::new();
        
        Ok(((common_messages, common_enums), service_types))
    }

    fn build_type_map(&self) -> HashMap<String, String> {
        let mut type_map = HashMap::new();
        
        for json_name in self.resolved_types.keys() {
            let proto_name = to_proto_type_name(json_name);
            type_map.insert(json_name.clone(), proto_name);
        }
        
        for json_name in self.resolved_enums.keys() {
            let proto_name = to_proto_type_name(json_name);
            type_map.insert(json_name.clone(), proto_name);
        }
        
        type_map
    }
}

// Helper function to resolve the recursion issue
fn schema_to_proto_field_type_impl(schema: &Schema) -> Result<ProtoFieldType> {
    match schema {
        Schema::Primitive(primitive) => match primitive {
            Primitive::String(_) => Ok(ProtoFieldType::String),
            Primitive::Integer(_) => Ok(ProtoFieldType::Int64),
            Primitive::Boolean(_) => Ok(ProtoFieldType::Bool),
            Primitive::Array(array) => {
                // For arrays, we need to mark the field as repeated
                schema_to_proto_field_type_impl(&array.items)
            }
            Primitive::Object(_) => Ok(ProtoFieldType::Message("Object".to_string())),
        },
        Schema::Ref(reference) => {
            let ref_name = reference.name();
            let proto_type_name = to_proto_type_name(ref_name);
            Ok(ProtoFieldType::Message(proto_type_name))
        }
        Schema::OneOf(_) => Ok(ProtoFieldType::Any),
        Schema::AllOf(_) => Ok(ProtoFieldType::Message("AllOf".to_string())),
    }
}

// Display implementations for proto types
impl fmt::Display for ProtoMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(comment) = &self.comment {
            writeln!(f, "{}", format_comment(comment, 0))?;
        }
        
        writeln!(f, "message {} {{", self.name)?;
        
        // Write oneofs
        for oneof in &self.oneofs {
            writeln!(f, "  oneof {} {{", oneof.name)?;
            for field in &oneof.fields {
                if let Some(comment) = &field.comment {
                    writeln!(f, "{}", format_comment(comment, 4))?;
                }
                write!(f, "    {} {} = {}", field.field_type, field.name, field.number)?;
                if let Some(json_name) = &field.json_name {
                    write!(f, " [json_name = \"{}\"]", json_name)?;
                }
                writeln!(f, ";")?;
            }
            writeln!(f, "  }}")?;
        }
        
        // Write regular fields
        for field in &self.fields {
            if field.oneof_name.is_some() {
                continue; // Skip oneof fields, already written above
            }
            
            if let Some(comment) = &field.comment {
                writeln!(f, "{}", format_comment(comment, 2))?;
            }
            
            write!(f, "  ")?;
            if field.repeated {
                write!(f, "repeated ")?;
            } else if field.optional {
                write!(f, "optional ")?;
            }
            
            write!(f, "{} {} = {}", field.field_type, field.name, field.number)?;
            
            if let Some(json_name) = &field.json_name {
                write!(f, " [json_name = \"{}\"]", json_name)?;
            }
            
            writeln!(f, ";")?;
        }
        
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl fmt::Display for ProtoEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(comment) = &self.comment {
            writeln!(f, "{}", format_comment(comment, 0))?;
        }
        
        writeln!(f, "enum {} {{", self.name)?;
        
        for value in &self.values {
            if let Some(comment) = &value.comment {
                writeln!(f, "{}", format_comment(comment, 2))?;
            }
            writeln!(f, "  {} = {};", value.name, value.number)?;
        }
        
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl fmt::Display for ProtoService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(comment) = &self.comment {
            writeln!(f, "{}", format_comment(comment, 0))?;
        }
        
        writeln!(f, "service {} {{", self.name)?;
        
        for rpc in &self.rpcs {
            if let Some(comment) = &rpc.comment {
                writeln!(f, "{}", format_comment(comment, 2))?;
            }
            
            write!(f, "  rpc {}(", rpc.name)?;
            if rpc.client_streaming {
                write!(f, "stream ")?;
            }
            write!(f, "{}) returns (", rpc.request_type)?;
            if rpc.server_streaming {
                write!(f, "stream ")?;
            }
            writeln!(f, "{});", rpc.response_type)?;
        }
        
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl fmt::Display for ProtoFieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtoFieldType::String => write!(f, "string"),
            ProtoFieldType::Int32 => write!(f, "int32"),
            ProtoFieldType::Int64 => write!(f, "int64"),
            ProtoFieldType::Uint32 => write!(f, "uint32"),
            ProtoFieldType::Uint64 => write!(f, "uint64"),
            ProtoFieldType::Bool => write!(f, "bool"),
            ProtoFieldType::Bytes => write!(f, "bytes"),
            ProtoFieldType::Double => write!(f, "double"),
            ProtoFieldType::Float => write!(f, "float"),
            ProtoFieldType::Message(name) => write!(f, "{}", name),
            ProtoFieldType::Enum(name) => write!(f, "{}", name),
            ProtoFieldType::Any => write!(f, "google.protobuf.Any"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn create_test_config() -> ProtoConfig {
        ProtoConfig::new("v0_1_0")
    }

    fn create_test_resolver() -> TypeResolver {
        TypeResolver::new(&create_test_config())
    }

    #[test]
    fn test_proto_message_display() {
        let message = ProtoMessage {
            name: "TestMessage".to_string(),
            fields: vec![
                ProtoField {
                    name: "id".to_string(),
                    field_type: ProtoFieldType::Int64,
                    number: 1,
                    json_name: Some("id".to_string()),
                    comment: Some("Unique identifier".to_string()),
                    optional: false,
                    repeated: false,
                    oneof_name: None,
                },
                ProtoField {
                    name: "name".to_string(),
                    field_type: ProtoFieldType::String,
                    number: 2,
                    json_name: Some("name".to_string()),
                    comment: None,
                    optional: true,
                    repeated: false,
                    oneof_name: None,
                },
            ],
            nested_messages: vec![],
            nested_enums: vec![],
            oneofs: vec![],
            comment: Some("Test message for validation".to_string()),
            options: vec![],
        };

        let output = format!("{}", message);
        assert!(output.contains("message TestMessage {"));
        assert!(output.contains("int64 id = 1 [json_name = \"id\"];"));
        assert!(output.contains("optional string name = 2 [json_name = \"name\"];"));
        assert!(output.contains("// Test message for validation"));
        assert!(output.contains("// Unique identifier"));
    }

    #[test]
    fn test_proto_enum_display() {
        let proto_enum = ProtoEnum {
            name: "TestEnum".to_string(),
            values: vec![
                ProtoEnumValue {
                    name: "UNKNOWN".to_string(),
                    number: 0,
                    comment: Some("Unknown value".to_string()),
                },
                ProtoEnumValue {
                    name: "ACTIVE".to_string(),
                    number: 1,
                    comment: None,
                },
            ],
            comment: Some("Test enumeration".to_string()),
        };

        let output = format!("{}", proto_enum);
        assert!(output.contains("enum TestEnum {"));
        assert!(output.contains("UNKNOWN = 0;"));
        assert!(output.contains("ACTIVE = 1;"));
        assert!(output.contains("// Test enumeration"));
        assert!(output.contains("// Unknown value"));
    }

    #[test]
    fn test_proto_service_display() {
        let service = ProtoService {
            name: "TestService".to_string(),
            rpcs: vec![
                ProtoRpc {
                    name: "GetItem".to_string(),
                    request_type: "GetItemRequest".to_string(),
                    response_type: "GetItemResponse".to_string(),
                    comment: Some("Retrieves an item".to_string()),
                    client_streaming: false,
                    server_streaming: false,
                },
                ProtoRpc {
                    name: "StreamUpdates".to_string(),
                    request_type: "StreamRequest".to_string(),
                    response_type: "Update".to_string(),
                    comment: None,
                    client_streaming: false,
                    server_streaming: true,
                },
            ],
            comment: Some("Test service".to_string()),
        };

        let output = format!("{}", service);
        assert!(output.contains("service TestService {"));
        assert!(output.contains("rpc GetItem(GetItemRequest) returns (GetItemResponse);"));
        assert!(output.contains("rpc StreamUpdates(StreamRequest) returns (stream Update);"));
        assert!(output.contains("// Test service"));
        assert!(output.contains("// Retrieves an item"));
    }

    #[test]
    fn test_object_to_message_conversion() {
        let resolver = create_test_resolver();
        
        let obj = ObjectPrimitive {
            title: Some("Test Object".to_string()),
            description: Some("A test object".to_string()),
            summary: None,
            properties: {
                let mut props = IndexMap::new();
                props.insert("id".to_string(), Schema::Primitive(Primitive::Integer(IntegerPrimitive {
                    title: None,
                    description: Some("ID field".to_string()),
                    minimum: Some(0),
                    not: None,
                })));
                props.insert("name".to_string(), Schema::Primitive(Primitive::String(StringPrimitive {
                    title: None,
                    comment: None,
                    description: Some("Name field".to_string()),
                    r#enum: None,
                    pattern: None,
                })));
                props
            },
            required: vec!["id".to_string()],
            additional_properties: None,
            not: None,
        };

        let message = resolver.convert_object_to_message("TestObject", &obj).unwrap();
        
        assert_eq!(message.name, "TestObject");
        assert_eq!(message.fields.len(), 2);
        assert_eq!(message.comment, Some("A test object".to_string()));
        
        let id_field = &message.fields[0];
        assert_eq!(id_field.name, "id");
        assert_eq!(id_field.json_name, Some("id".to_string()));
        assert!(!id_field.optional); // Required field
        
        let name_field = &message.fields[1];
        assert_eq!(name_field.name, "name");
        assert_eq!(name_field.json_name, Some("name".to_string()));
        assert!(name_field.optional); // Not in required list
    }

    #[test]
    fn test_string_enum_conversion() {
        let resolver = create_test_resolver();
        let values = vec!["pending".to_string(), "completed".to_string(), "failed".to_string()];
        
        let proto_enum = resolver.convert_string_enum_to_enum(
            "Status",
            &values,
            Some("Status enumeration")
        ).unwrap();
        
        assert_eq!(proto_enum.name, "Status");
        assert_eq!(proto_enum.values.len(), 3);
        assert_eq!(proto_enum.comment, Some("Status enumeration".to_string()));
        
        assert_eq!(proto_enum.values[0].name, "PENDING");
        assert_eq!(proto_enum.values[0].number, 0);
        
        assert_eq!(proto_enum.values[1].name, "COMPLETED");
        assert_eq!(proto_enum.values[1].number, 1);
        
        assert_eq!(proto_enum.values[2].name, "FAILED");
        assert_eq!(proto_enum.values[2].number, 2);
    }

    #[test]
    fn test_oneof_conversion() {
        let resolver = create_test_resolver();
        
        let oneof = OneOf {
            title: Some("Union Type".to_string()),
            description: Some("A union of types".to_string()),
            one_of: vec![
                Schema::Primitive(Primitive::String(StringPrimitive {
                    title: None,
                    comment: None,
                    description: Some("String variant".to_string()),
                    r#enum: None,
                    pattern: None,
                })),
                Schema::Primitive(Primitive::Integer(IntegerPrimitive {
                    title: None,
                    description: Some("Integer variant".to_string()),
                    minimum: None,
                    not: None,
                })),
            ],
        };

        let message = resolver.convert_oneof_to_message("UnionType", &oneof).unwrap();
        
        assert_eq!(message.name, "UnionType");
        assert_eq!(message.oneofs.len(), 1);
        assert_eq!(message.oneofs[0].name, "value");
        assert_eq!(message.oneofs[0].fields.len(), 2);
        assert_eq!(message.comment, Some("A union of types".to_string()));
    }

    #[test]
    fn test_schema_to_proto_field_type() {
        let resolver = create_test_resolver();
        
        // Test string
        let string_schema = Schema::Primitive(Primitive::String(StringPrimitive {
            title: None,
            comment: None,
            description: None,
            r#enum: None,
            pattern: None,
        }));
        let field_type = resolver.schema_to_proto_field_type(&string_schema).unwrap();
        assert!(matches!(field_type, ProtoFieldType::String));
        
        // Test integer
        let int_schema = Schema::Primitive(Primitive::Integer(IntegerPrimitive {
            title: None,
            description: None,
            minimum: None,
            not: None,
        }));
        let field_type = resolver.schema_to_proto_field_type(&int_schema).unwrap();
        assert!(matches!(field_type, ProtoFieldType::Int64));
        
        // Test boolean
        let bool_schema = Schema::Primitive(Primitive::Boolean(BooleanPrimitive {
            title: None,
            description: None,
        }));
        let field_type = resolver.schema_to_proto_field_type(&bool_schema).unwrap();
        assert!(matches!(field_type, ProtoFieldType::Bool));
        
        // Test reference
        let ref_schema = Schema::Ref(Reference {
            title: None,
            comment: None,
            description: None,
            ref_field: "#/components/schemas/FELT".to_string(),
            additional_fields: std::collections::HashMap::new(),
        });
        let field_type = resolver.schema_to_proto_field_type(&ref_schema).unwrap();
        if let ProtoFieldType::Message(type_name) = field_type {
            assert_eq!(type_name, "Felt");
        } else {
            panic!("Expected Message type");
        }
    }
} 