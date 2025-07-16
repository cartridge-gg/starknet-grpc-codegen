use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::Path;

#[derive(Debug, Parser)]
pub struct Test {
    #[clap(long, env, help = "Version of the specification")]
    spec: String,
    #[clap(long, env, help = "Test specific types (comma-separated)")]
    types: Option<String>,
    #[clap(long, env, help = "Seed for reproducible test generation", default_value = "42")]
    seed: u64,
}

impl Test {
    pub(crate) fn run(self) -> Result<()> {
        println!("🧪 Running round-trip tests for specification version: {}", self.spec);
        
        // Check if proto files exist
        let proto_dir = Path::new("proto").join(&self.spec);
        if !proto_dir.exists() {
            println!("❌ Proto files not found. Please run generate first:");
            println!("   cargo run -- generate --spec {}", self.spec);
            return Ok(());
        }
        
        println!("✅ Found proto files in: {}", proto_dir.display());
        
        // List the proto files
        let entries = fs::read_dir(&proto_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("proto") {
                println!("   📄 {}", path.file_name().unwrap().to_string_lossy());
            }
        }
        
        // Simple validation test
        self.validate_proto_files(&proto_dir)?;
        
        println!("✅ Basic validation completed!");
        println!("");
        println!("Note: Full round-trip testing requires protobuf compilation.");
        println!("To enable full testing, install protoc and implement the complete test suite.");
        
        Ok(())
    }
    
    fn validate_proto_files(&self, proto_dir: &Path) -> Result<()> {
        println!("🔍 Validating proto files...");
        
        let entries = fs::read_dir(proto_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("proto") {
                let content = fs::read_to_string(&path)?;
                
                // Basic validation checks
                if !content.contains("syntax = \"proto3\";") {
                    println!("   ⚠️  {}: Missing proto3 syntax declaration", path.file_name().unwrap().to_string_lossy());
                }
                
                if !content.contains("package starknet") {
                    println!("   ⚠️  {}: Missing package declaration", path.file_name().unwrap().to_string_lossy());
                }
                
                // Count message definitions
                let message_count = content.matches("message ").count();
                let enum_count = content.matches("enum ").count();
                let service_count = content.matches("service ").count();
                
                println!("   📊 {}: {} messages, {} enums, {} services", 
                    path.file_name().unwrap().to_string_lossy(),
                    message_count,
                    enum_count,
                    service_count
                );
            }
        }
        
        Ok(())
    }
}
