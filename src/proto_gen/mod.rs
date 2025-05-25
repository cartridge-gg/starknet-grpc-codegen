use std::collections::HashMap;

use anyhow::Result;

use crate::spec::*;

pub mod writer;
pub mod types;
pub mod service;

pub use writer::ProtoWriter;
pub use types::*;
pub use service::*;

/// Configuration for proto generation
#[derive(Debug, Clone)]
pub struct ProtoConfig {
    pub package_prefix: String,
    pub version: String,
    #[allow(dead_code)]
    pub output_dir: String,
}

/// Proto file generation result
#[derive(Debug, Clone)]
pub struct ProtoGenerationResult {
    pub files: HashMap<String, String>,
    pub package_info: PackageInfo,
}

/// Package information for generated protos
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub main_package: String,
    pub write_package: String,
    pub trace_package: String,
    pub ws_package: String,
    pub common_package: String,
}

/// Main proto generator
pub struct ProtoGenerator {
    config: ProtoConfig,
    type_resolver: TypeResolver,
}

impl ProtoGenerator {
    pub fn new(config: ProtoConfig) -> Self {
        Self {
            type_resolver: TypeResolver::new(&config),
            config,
        }
    }

    pub fn generate(&mut self, specs: &Specification) -> Result<ProtoGenerationResult> {
        let mut files = HashMap::new();
        
        // Resolve all types first
        let type_resolution = self.type_resolver.resolve_types(specs)?;
        
        // Generate common types file
        let common_proto = self.generate_common_proto(&type_resolution)?;
        files.insert("common.proto".to_string(), common_proto);
        
        // Generate service files
        let main_proto = self.generate_main_service_proto(specs, &type_resolution)?;
        files.insert("main.proto".to_string(), main_proto);
        
        let write_proto = self.generate_write_service_proto(specs, &type_resolution)?;
        files.insert("write.proto".to_string(), write_proto);
        
        let trace_proto = self.generate_trace_service_proto(specs, &type_resolution)?;
        files.insert("trace.proto".to_string(), trace_proto);
        
        let ws_proto = self.generate_ws_service_proto(specs, &type_resolution)?;
        files.insert("ws.proto".to_string(), ws_proto);
        
        let package_info = PackageInfo {
            main_package: format!("{}.{}.main", self.config.package_prefix, self.config.version),
            write_package: format!("{}.{}.write", self.config.package_prefix, self.config.version),
            trace_package: format!("{}.{}.trace", self.config.package_prefix, self.config.version),
            ws_package: format!("{}.{}.ws", self.config.package_prefix, self.config.version),
            common_package: format!("{}.{}.common", self.config.package_prefix, self.config.version),
        };
        
        Ok(ProtoGenerationResult {
            files,
            package_info,
        })
    }
    
    fn generate_common_proto(&self, type_resolution: &TypeResolution) -> Result<String> {
        let mut writer = ProtoWriter::new(&self.config.common_package());
        
        // Add common imports for better language compatibility
        writer.add_import("google/protobuf/any.proto");
        writer.add_import("google/protobuf/timestamp.proto");
        writer.add_import("google/protobuf/duration.proto");
        writer.add_import("google/protobuf/empty.proto");
        writer.add_import("google/protobuf/wrappers.proto");
        
        // Generate common types
        for proto_type in &type_resolution.common_types {
            writer.add_message(proto_type);
        }
        
        // Generate common enums
        for proto_enum in &type_resolution.common_enums {
            writer.add_enum(proto_enum);
        }
        
        Ok(writer.to_string())
    }
    
    fn generate_main_service_proto(&self, specs: &Specification, type_resolution: &TypeResolution) -> Result<String> {
        let mut writer = ProtoWriter::new(&self.config.main_package());
        writer.add_import(&format!("{}/common.proto", self.config.version));
        
        let service = ServiceGenerator::new("StarknetMainService", &self.config)
            .generate_from_methods(&self.filter_main_methods(&specs.methods))?;
        
        writer.add_service(&service);
        
        // Add service-specific types
        for proto_type in &type_resolution.main_types {
            writer.add_message(proto_type);
        }
        
        Ok(writer.to_string())
    }
    
    fn generate_write_service_proto(&self, specs: &Specification, type_resolution: &TypeResolution) -> Result<String> {
        let mut writer = ProtoWriter::new(&self.config.write_package());
        writer.add_import(&format!("{}/common.proto", self.config.version));
        
        let service = ServiceGenerator::new("StarknetWriteService", &self.config)
            .generate_from_methods(&self.filter_write_methods(&specs.methods))?;
        
        writer.add_service(&service);
        
        // Add service-specific types
        for proto_type in &type_resolution.write_types {
            writer.add_message(proto_type);
        }
        
        Ok(writer.to_string())
    }
    
    fn generate_trace_service_proto(&self, specs: &Specification, type_resolution: &TypeResolution) -> Result<String> {
        let mut writer = ProtoWriter::new(&self.config.trace_package());
        writer.add_import(&format!("{}/common.proto", self.config.version));
        
        let service = ServiceGenerator::new("StarknetTraceService", &self.config)
            .generate_from_methods(&self.filter_trace_methods(&specs.methods))?;
        
        writer.add_service(&service);
        
        // Add service-specific types
        for proto_type in &type_resolution.trace_types {
            writer.add_message(proto_type);
        }
        
        Ok(writer.to_string())
    }
    
    fn generate_ws_service_proto(&self, specs: &Specification, type_resolution: &TypeResolution) -> Result<String> {
        let mut writer = ProtoWriter::new(&self.config.ws_package());
        writer.add_import(&format!("{}/common.proto", self.config.version));
        
        let service = ServiceGenerator::new("StarknetWsService", &self.config)
            .generate_from_methods(&self.filter_ws_methods(&specs.methods))?;
        
        writer.add_service(&service);
        
        // Add service-specific types
        for proto_type in &type_resolution.ws_types {
            writer.add_message(proto_type);
        }
        
        Ok(writer.to_string())
    }
    
    fn filter_main_methods<'a>(&self, methods: &'a [Method]) -> Vec<&'a Method> {
        methods.iter()
            .filter(|m| !m.name.contains("write") && !m.name.contains("trace") && !m.name.contains("subscribe"))
            .collect()
    }
    
    fn filter_write_methods<'a>(&self, methods: &'a [Method]) -> Vec<&'a Method> {
        methods.iter()
            .filter(|m| m.name.contains("write") || m.name.contains("add") || m.name.contains("invoke"))
            .collect()
    }
    
    fn filter_trace_methods<'a>(&self, methods: &'a [Method]) -> Vec<&'a Method> {
        methods.iter()
            .filter(|m| m.name.contains("trace"))
            .collect()
    }
    
    fn filter_ws_methods<'a>(&self, methods: &'a [Method]) -> Vec<&'a Method> {
        methods.iter()
            .filter(|m| m.name.contains("subscribe"))
            .collect()
    }
}

impl ProtoConfig {
    pub fn new(version: &str) -> Self {
        Self {
            package_prefix: "starknet".to_string(),
            version: version.replace(".", "_"),
            output_dir: "proto".to_string(),
        }
    }
    
    pub fn main_package(&self) -> String {
        format!("{}.{}.main", self.package_prefix, self.version)
    }
    
    pub fn write_package(&self) -> String {
        format!("{}.{}.write", self.package_prefix, self.version)
    }
    
    pub fn trace_package(&self) -> String {
        format!("{}.{}.trace", self.package_prefix, self.version)
    }
    
    pub fn ws_package(&self) -> String {
        format!("{}.{}.ws", self.package_prefix, self.version)
    }
    
    pub fn common_package(&self) -> String {
        format!("{}.{}.common", self.package_prefix, self.version)
    }
} 