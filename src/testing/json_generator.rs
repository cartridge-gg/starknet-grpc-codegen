use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use rand::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

use crate::spec::{AllOf, OneOf, Primitive, Reference, Schema};

/// Generator for creating test JSON data from OpenRPC schemas
pub struct JsonGenerator {
    /// Random number generator for reproducible test data
    rng: StdRng,
    /// Maximum depth for nested object generation
    max_depth: usize,
    /// Maximum number of items in arrays
    max_array_items: usize,
    /// Whether to generate optional fields
    generate_optional: bool,
    /// Cache of generated values to avoid infinite recursion
    generated_cache: HashMap<String, Value>,
    /// Current depth in object generation
    current_depth: usize,
}

impl JsonGenerator {
    pub fn new(seed: Option<u64>, config: &super::TestConfig) -> Self {
        let rng = if let Some(seed) = seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        Self {
            rng,
            max_depth: config.max_depth,
            max_array_items: config.max_array_items,
            generate_optional: config.generate_optional,
            generated_cache: HashMap::new(),
            current_depth: 0,
        }
    }

    /// Generate test JSON data from a schema
    pub fn generate_json(&mut self, schema: &Schema, schemas: &IndexMap<String, Schema>) -> Result<Value> {
        self.current_depth = 0;
        self.generated_cache.clear();
        self.generate_json_internal(schema, schemas)
    }

    /// Internal method for generating JSON with depth tracking
    fn generate_json_internal(&mut self, schema: &Schema, schemas: &IndexMap<String, Schema>) -> Result<Value> {
        if self.current_depth > self.max_depth {
            return Ok(Value::Null);
        }

        self.current_depth += 1;
        let result = match schema {
            Schema::Ref(reference) => self.generate_from_reference(reference, schemas)?,
            Schema::OneOf(oneof) => self.generate_from_oneof(oneof, schemas)?,
            Schema::AllOf(allof) => self.generate_from_allof(allof, schemas)?,
            Schema::Primitive(primitive) => self.generate_from_primitive(primitive, schemas)?,
        };
        self.current_depth -= 1;

        Ok(result)
    }

    /// Generate JSON from a reference schema
    fn generate_from_reference(&mut self, reference: &Reference, schemas: &IndexMap<String, Schema>) -> Result<Value> {
        let ref_name = reference.name();
        
        // Check cache to avoid infinite recursion
        if let Some(cached) = self.generated_cache.get(ref_name) {
            return Ok(cached.clone());
        }

        // Find the referenced schema
        let schema = schemas
            .get(ref_name)
            .ok_or_else(|| anyhow!("Referenced schema not found: {}", ref_name))?;

        // Generate the value
        let value = self.generate_json_internal(schema, schemas)?;
        
        // Cache the result
        self.generated_cache.insert(ref_name.to_string(), value.clone());
        
        Ok(value)
    }

    /// Generate JSON from a oneOf schema
    fn generate_from_oneof(&mut self, oneof: &OneOf, schemas: &IndexMap<String, Schema>) -> Result<Value> {
        if oneof.one_of.is_empty() {
            return Ok(Value::Null);
        }

        // Randomly select one of the schemas
        let index = self.rng.gen_range(0..oneof.one_of.len());
        let selected_schema = &oneof.one_of[index];

        self.generate_json_internal(selected_schema, schemas)
    }

    /// Generate JSON from an allOf schema
    fn generate_from_allof(&mut self, allof: &AllOf, schemas: &IndexMap<String, Schema>) -> Result<Value> {
        if allof.all_of.is_empty() {
            return Ok(Value::Null);
        }

        // For allOf, we need to merge all schemas
        // This is complex, so we'll start with the first schema and merge others
        let mut result = self.generate_json_internal(&allof.all_of[0], schemas)?;

        // For now, we'll just use the first schema
        // TODO: Implement proper merging of multiple schemas
        Ok(result)
    }

    /// Generate JSON from a primitive schema
    fn generate_from_primitive(&mut self, primitive: &Primitive, schemas: &IndexMap<String, Schema>) -> Result<Value> {
        match primitive {
            Primitive::Array(array) => self.generate_array(array, schemas),
            Primitive::Boolean(boolean) => self.generate_boolean(boolean),
            Primitive::Integer(integer) => self.generate_integer(integer),
            Primitive::Object(object) => self.generate_object(object, schemas),
            Primitive::String(string) => self.generate_string(string),
        }
    }

    /// Generate an array value
    fn generate_array(&mut self, array: &crate::spec::ArrayPrimitive, schemas: &IndexMap<String, Schema>) -> Result<Value> {
        let item_count = self.rng.gen_range(1..=self.max_array_items);
        let mut items = Vec::new();

        for _ in 0..item_count {
            let item = self.generate_json_internal(&array.items, schemas)?;
            items.push(item);
        }

        Ok(Value::Array(items))
    }

    /// Generate a boolean value
    fn generate_boolean(&mut self, _boolean: &crate::spec::BooleanPrimitive) -> Result<Value> {
        Ok(Value::Bool(self.rng.gen()))
    }

    /// Generate an integer value
    fn generate_integer(&mut self, integer: &crate::spec::IntegerPrimitive) -> Result<Value> {
        let min = integer.minimum.unwrap_or(i32::MIN);
        let max = i32::MAX; // TODO: Add maximum constraint support
        
        let value = self.rng.gen_range(min..=max);
        Ok(Value::Number(value.into()))
    }

    /// Generate an object value
    fn generate_object(&mut self, object: &crate::spec::ObjectPrimitive, schemas: &IndexMap<String, Schema>) -> Result<Value> {
        let mut obj = serde_json::Map::new();

        for (field_name, field_schema) in &object.properties {
            let is_required = object.required.contains(field_name);
            
            // Skip optional fields if not generating them
            if !is_required && !self.generate_optional {
                continue;
            }

            // Generate the field value
            let field_value = self.generate_json_internal(field_schema, schemas)?;
            
            // Only add non-null values for optional fields
            if is_required || field_value != Value::Null {
                obj.insert(field_name.clone(), field_value);
            }
        }

        Ok(Value::Object(obj))
    }

    /// Generate a string value
    fn generate_string(&mut self, string: &crate::spec::StringPrimitive) -> Result<Value> {
        // Handle enums
        if let Some(enum_values) = &string.r#enum {
            if !enum_values.is_empty() {
                let index = self.rng.gen_range(0..enum_values.len());
                return Ok(Value::String(enum_values[index].clone()));
            }
        }

        // Handle patterns (simplified)
        if let Some(pattern) = &string.pattern {
            return self.generate_string_from_pattern(pattern);
        }

        // Generate based on field name hints
        let field_name = string.title.as_deref().unwrap_or("");
        let generated_string = self.generate_string_from_hint(field_name);
        
        Ok(Value::String(generated_string))
    }

    /// Generate a string from a pattern (simplified implementation)
    fn generate_string_from_pattern(&mut self, pattern: &str) -> Result<Value> {
        // This is a simplified pattern matcher
        // In a real implementation, you'd want a proper regex-based generator
        
        if pattern.contains("^0x[0-9a-fA-F]+$") {
            // Hex string pattern
            let hex_chars = "0123456789abcdef";
            let length = self.rng.gen_range(8..=64);
            let hex_string: String = (0..length)
                .map(|_| hex_chars.chars().nth(self.rng.gen_range(0..16)).unwrap())
                .collect();
            return Ok(Value::String(format!("0x{}", hex_string)));
        }

        // Default to a simple string
        Ok(Value::String("test_string".to_string()))
    }

    /// Generate a string based on field name hints
    fn generate_string_from_hint(&mut self, field_name: &str) -> String {
        let field_lower = field_name.to_lowercase();
        
        match field_lower {
            s if s.contains("hash") => {
                let hex_chars = "0123456789abcdef";
                let length = self.rng.gen_range(32..=64);
                let hex_string: String = (0..length)
                    .map(|_| hex_chars.chars().nth(self.rng.gen_range(0..16)).unwrap())
                    .collect();
                format!("0x{}", hex_string)
            }
            s if s.contains("address") => {
                let hex_chars = "0123456789abcdef";
                let hex_string: String = (0..64)
                    .map(|_| hex_chars.chars().nth(self.rng.gen_range(0..16)).unwrap())
                    .collect();
                format!("0x{}", hex_string)
            }
            s if s.contains("url") => "https://example.com/api".to_string(),
            s if s.contains("email") => "test@example.com".to_string(),
            s if s.contains("uuid") => "123e4567-e89b-12d3-a456-426614174000".to_string(),
            s if s.contains("date") => "2023-01-01T00:00:00Z".to_string(),
            s if s.contains("version") => "1.0.0".to_string(),
            _ => {
                // Generate a random string
                let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
                let length = self.rng.gen_range(5..=20);
                (0..length)
                    .map(|_| chars[self.rng.gen_range(0..chars.len())])
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{ArrayPrimitive, BooleanPrimitive, IntegerPrimitive, ObjectPrimitive, StringPrimitive};

    #[test]
    fn test_generate_boolean() {
        let config = super::super::TestConfig::default();
        let mut generator = JsonGenerator::new(Some(42), &config);
        let schema = Schema::Primitive(Primitive::Boolean(BooleanPrimitive {
            title: None,
            description: None,
        }));

        let value = generator.generate_json(&schema, &IndexMap::new()).unwrap();
        assert!(value.is_boolean());
    }

    #[test]
    fn test_generate_integer() {
        let config = super::super::TestConfig::default();
        let mut generator = JsonGenerator::new(Some(42), &config);
        let schema = Schema::Primitive(Primitive::Integer(IntegerPrimitive {
            title: None,
            description: None,
            minimum: Some(0),
            not: None,
        }));

        let value = generator.generate_json(&schema, &IndexMap::new()).unwrap();
        assert!(value.is_number());
        assert!(value.as_i64().unwrap() >= 0);
    }

    #[test]
    fn test_generate_string() {
        let config = super::super::TestConfig::default();
        let mut generator = JsonGenerator::new(Some(42), &config);
        let schema = Schema::Primitive(Primitive::String(StringPrimitive {
            title: Some("test_field".to_string()),
            comment: None,
            description: None,
            r#enum: None,
            pattern: None,
        }));

        let value = generator.generate_json(&schema, &IndexMap::new()).unwrap();
        assert!(value.is_string());
    }

    #[test]
    fn test_generate_array() {
        let config = super::super::TestConfig::default();
        let mut generator = JsonGenerator::new(Some(42), &config);
        let schema = Schema::Primitive(Primitive::Array(ArrayPrimitive {
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

        let value = generator.generate_json(&schema, &IndexMap::new()).unwrap();
        assert!(value.is_array());
        let array = value.as_array().unwrap();
        assert!(!array.is_empty());
        assert!(array.len() <= config.max_array_items);
    }

    #[test]
    fn test_generate_object() {
        let config = super::super::TestConfig::default();
        let mut generator = JsonGenerator::new(Some(42), &config);
        
        let mut properties = IndexMap::new();
        properties.insert(
            "name".to_string(),
            Schema::Primitive(Primitive::String(StringPrimitive {
                title: None,
                comment: None,
                description: None,
                r#enum: None,
                pattern: None,
            })),
        );
        properties.insert(
            "age".to_string(),
            Schema::Primitive(Primitive::Integer(IntegerPrimitive {
                title: None,
                description: None,
                minimum: Some(0),
                not: None,
            })),
        );

        let schema = Schema::Primitive(Primitive::Object(ObjectPrimitive {
            title: None,
            description: None,
            summary: None,
            properties,
            required: vec!["name".to_string()],
            additional_properties: None,
            not: None,
        }));

        let value = generator.generate_json(&schema, &IndexMap::new()).unwrap();
        assert!(value.is_object());
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("name")); // Required field
        assert!(obj.get("name").unwrap().is_string());
    }
}