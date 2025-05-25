use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Specification {
    pub openrpc: String,
    pub info: Info,
    pub servers: Vec<String>,
    pub methods: Vec<Method>,
    pub components: Components,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Info {
    pub version: String,
    pub title: String,
    pub license: Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Method {
    pub name: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param_structure: Option<String>,
    pub params: Vec<Param>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<MethodResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<Reference>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Components {
    pub content_descriptors: Empty,
    pub schemas: IndexMap<String, Schema>,
    pub errors: IndexMap<String, ErrorType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Empty {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Param {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub required: bool,
    pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MethodResult {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    pub schema: Schema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Schema {
    Ref(Reference),
    OneOf(OneOf),
    AllOf(AllOf),
    Primitive(Primitive),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "$comment")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "$ref")]
    pub ref_field: String,
    // Allow additional fields that we don't need to parse
    #[serde(flatten)]
    pub additional_fields: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OneOf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub one_of: Vec<Schema>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AllOf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub all_of: Vec<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Primitive {
    Array(ArrayPrimitive),
    Boolean(BooleanPrimitive),
    Integer(IntegerPrimitive),
    Object(ObjectPrimitive),
    String(StringPrimitive),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ArrayPrimitive {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub items: Box<Schema>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BooleanPrimitive {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct IntegerPrimitive {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i32>,
    // Field not handled for now
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ObjectPrimitive {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub properties: IndexMap<String, Schema>,
    pub required: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<bool>,
    // Field not handled for now
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringPrimitive {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "$comment")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorType {
    Error(Error),
    Reference(Reference),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Error {
    pub code: u32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Schema>,
}

impl Schema {
    #[allow(dead_code)]
    pub fn title(&self) -> Option<&String> {
        match self {
            Schema::Primitive(primitive) => primitive.title(),
            Schema::Ref(reference) => reference.title.as_ref(),
            Schema::OneOf(oneof) => oneof.title.as_ref(),
            Schema::AllOf(allof) => allof.title.as_ref(),
        }
    }

    pub fn description(&self) -> Option<&String> {
        match self {
            Schema::Primitive(primitive) => primitive.description(),
            Schema::Ref(reference) => reference.description.as_ref(),
            Schema::OneOf(oneof) => oneof.description.as_ref(),
            Schema::AllOf(allof) => allof.description.as_ref(),
        }
    }

    #[allow(dead_code)]
    pub fn summary(&self) -> Option<&String> {
        match self {
            Schema::Primitive(primitive) => primitive.summary(),
            Schema::Ref(_) => None,
            Schema::OneOf(_) => None,
            Schema::AllOf(_) => None,
        }
    }
}

impl Primitive {
    #[allow(dead_code)]
    pub fn title(&self) -> Option<&String> {
        match self {
            Primitive::String(string_primitive) => string_primitive.title.as_ref(),
            Primitive::Integer(integer_primitive) => integer_primitive.title.as_ref(),
            Primitive::Boolean(boolean_primitive) => boolean_primitive.title.as_ref(),
            Primitive::Array(array_primitive) => array_primitive.title.as_ref(),
            Primitive::Object(object_primitive) => object_primitive.title.as_ref(),
        }
    }

    pub fn description(&self) -> Option<&String> {
        match self {
            Primitive::String(string_primitive) => string_primitive.description.as_ref(),
            Primitive::Integer(integer_primitive) => integer_primitive.description.as_ref(),
            Primitive::Boolean(boolean_primitive) => boolean_primitive.description.as_ref(),
            Primitive::Array(array_primitive) => array_primitive.description.as_ref(),
            Primitive::Object(object_primitive) => object_primitive.description.as_ref(),
        }
    }

    #[allow(dead_code)]
    pub fn summary(&self) -> Option<&String> {
        match self {
            Primitive::String(_) => None,
            Primitive::Integer(_) => None,
            Primitive::Boolean(_) => None,
            Primitive::Array(_) => None,
            Primitive::Object(object_primitive) => object_primitive.summary.as_ref(),
        }
    }
}

impl Reference {
    pub fn name(&self) -> &str {
        match self.ref_field.rfind('/') {
            Some(ind_slash) => &self.ref_field[ind_slash + 1..],
            None => &self.ref_field,
        }
    }
}
