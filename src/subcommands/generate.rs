use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::Parser;

use crate::{
    built_info, 
    spec::*,
    proto_gen::{ProtoGenerator, ProtoConfig},
    GenerationProfile, SpecVersion,
};

#[derive(Debug, Parser)]
pub struct Generate {
    #[clap(long, env, help = "Version of the specification")]
    spec: SpecVersion,
    #[clap(long, env, help = "Output directory for generated proto files", default_value = "proto")]
    output_dir: String,
}

impl Generate {
    pub(crate) fn run(self, profiles: &[GenerationProfile]) -> Result<()> {
        let profile = profiles
            .iter()
            .find(|profile| profile.version == self.spec)
            .expect("Unable to find profile");

        let specs = profile
            .raw_specs
            .parse_full()
            .expect("Failed to parse specification");

        println!("Generating gRPC proto files for Starknet specification version {:?}", self.spec);
        
        // Create proto config
        let version_str = self.spec.to_version_string();
        let config = ProtoConfig::new(&version_str);
        
        // Generate proto files
        let mut generator = ProtoGenerator::new(config.clone());
        let result = generator.generate(&specs)?;
        
        // Create output directory structure
        let output_path = Path::new(&self.output_dir).join(&config.version);
        fs::create_dir_all(&output_path)?;
        
        // Write proto files
        for (filename, content) in &result.files {
            let file_path = output_path.join(filename);
            
            println!("Writing {}", file_path.display());
            
            // Add generation header
            let header = self.generate_file_header(filename, &specs);
            let full_content = format!("{}\n{}", header, content);
            
            fs::write(&file_path, full_content)?;
        }
        
        // Generate buf.yaml for proto compilation
        self.generate_buf_config(&output_path, &config)?;
        
        // Generate language-specific buf configurations
        self.generate_language_configs(&output_path, &config)?;
        
        // Generate README with instructions
        self.generate_readme(&output_path, &result.package_info)?;
        
        println!("✅ Successfully generated {} proto files in {}", 
                 result.files.len(), 
                 output_path.display());
        
        println!("\nGenerated packages:");
        println!("  - {} (common types)", result.package_info.common_package);
        println!("  - {} (main service)", result.package_info.main_package);
        println!("  - {} (write service)", result.package_info.write_package);
        println!("  - {} (trace service)", result.package_info.trace_package);
        println!("  - {} (websocket service)", result.package_info.ws_package);

        Ok(())
    }
    
    fn generate_file_header(&self, filename: &str, specs: &Specification) -> String {
        let mut header = String::new();
        
        // Add generation info as comments
        header.push_str("// AUTO-GENERATED PROTOBUF FILE. DO NOT EDIT\n");
        header.push_str("// Generated from Starknet JSON-RPC specification\n");
        header.push_str("// \n");
        header.push_str("// Generation tool: https://github.com/cartridge-gg/starknet-grpc-codegen\n");
        
        if let Some(commit_hash) = built_info::GIT_COMMIT_HASH {
            header.push_str(&format!("// Generated with commit: {}\n", commit_hash));
        }
        
        header.push_str(&format!("// Specification version: {}\n", specs.info.version));
        header.push_str(&format!("// Generated file: {}\n", filename));
        header.push_str("// \n");
        header.push_str("// This file contains protobuf definitions with JSON marshalling support.\n");
        header.push_str("// Field names preserve the exact JSON structure using json_name options.\n");
        
        header
    }
    
    fn generate_buf_config(&self, output_path: &Path, _config: &ProtoConfig) -> Result<()> {
        let buf_content = r#"version: v1
breaking:
  use:
    - FILE
lint:
  use:
    - DEFAULT
  except:
    - FIELD_LOWER_SNAKE_CASE  # We use json_name for JSON compatibility
    - ENUM_VALUE_PREFIX      # We preserve original enum values
build:
  roots:
    - .
  excludes: []
"#.to_string();
        
        let buf_path = output_path.join("buf.yaml");
        fs::write(buf_path, buf_content)?;
        
        Ok(())
    }
    
    fn generate_language_configs(&self, output_path: &Path, _config: &ProtoConfig) -> Result<()> {
        // Go configuration
        let go_config = r#"version: v1
plugins:
  - plugin: buf.build/protocolbuffers/go
    out: gen/go
    opt:
      - paths=source_relative
  - plugin: buf.build/grpc/go
    out: gen/go
    opt:
      - paths=source_relative
      - require_unimplemented_servers=false
"#;
        
        // Rust configuration (using prost + tonic)
        let rust_config = r#"version: v1
plugins:
  - plugin: buf.build/community/neoeinstein-prost
    out: gen/rust/src
    opt:
      - bytes=.
      - file_descriptor_set=false
      - compile_well_known_types=true
  - plugin: buf.build/community/neoeinstein-tonic
    out: gen/rust/src
    opt:
      - compile_well_known_types=true
      - no_include=true
"#;

        // TypeScript/JavaScript configuration
        let ts_config = r#"version: v1
plugins:
  - plugin: buf.build/protocolbuffers/js
    out: gen/js
    opt:
      - import_style=commonjs
      - binary
  - plugin: buf.build/grpc/web
    out: gen/js
    opt:
      - import_style=typescript
      - mode=grpcwebtext
  - plugin: buf.build/bufbuild/es
    out: gen/ts
    opt:
      - target=ts
      - js_import_style=module
"#;

        // Node.js gRPC configuration  
        let node_config = r#"version: v1
plugins:
  - plugin: buf.build/protocolbuffers/js
    out: gen/node
    opt:
      - import_style=commonjs
  - plugin: buf.build/grpc/node
    out: gen/node
"#;

        // Python configuration
        let python_config = r#"version: v1
plugins:
  - plugin: buf.build/protocolbuffers/python
    out: gen/python
  - plugin: buf.build/grpc/python
    out: gen/python
"#;

        // Write configuration files
        fs::write(output_path.join("buf.gen.go.yaml"), go_config)?;
        fs::write(output_path.join("buf.gen.rust.yaml"), rust_config)?;
        fs::write(output_path.join("buf.gen.ts.yaml"), ts_config)?;
        fs::write(output_path.join("buf.gen.node.yaml"), node_config)?;
        fs::write(output_path.join("buf.gen.python.yaml"), python_config)?;
        
        // Create a comprehensive generation script
        let gen_script = r#"#!/bin/bash
# Starknet Proto Code Generation Script
# This script generates client code for multiple languages

set -e

echo "🚀 Generating Starknet gRPC client code for multiple languages..."

# Create output directories
mkdir -p gen/{go,rust/src,js,ts,node,python}

# Generate Go code
echo "📦 Generating Go code..."
buf generate --template buf.gen.go.yaml

# Generate Rust code
echo "🦀 Generating Rust code..."
buf generate --template buf.gen.rust.yaml

# Generate TypeScript/JavaScript code
echo "📜 Generating TypeScript code..."
buf generate --template buf.gen.ts.yaml

# Generate Node.js code
echo "🟢 Generating Node.js code..."
buf generate --template buf.gen.node.yaml

# Generate Python code
echo "🐍 Generating Python code..."
buf generate --template buf.gen.python.yaml

echo "✅ Code generation complete!"
echo ""
echo "Generated client code locations:"
echo "  - Go:         gen/go/"
echo "  - Rust:       gen/rust/src/"
echo "  - TypeScript: gen/ts/"
echo "  - JavaScript: gen/js/"
echo "  - Node.js:    gen/node/"
echo "  - Python:     gen/python/"
echo ""
echo "Next steps:"
echo "  1. Copy the generated code to your project"
echo "  2. Install language-specific dependencies"
echo "  3. Import and use the generated clients"
"#;

        let gen_script_path = output_path.join("generate-clients.sh");
        fs::write(&gen_script_path, gen_script)?;
        
        // Make the script executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&gen_script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&gen_script_path, perms)?;
        }
        
        Ok(())
    }
    
    fn generate_readme(&self, output_path: &Path, package_info: &crate::proto_gen::PackageInfo) -> Result<()> {
        let readme_content = format!(r#"# Starknet gRPC Protocol Buffers

This directory contains auto-generated Protocol Buffer definitions for the Starknet JSON-RPC API.

## Generated Files

- `common.proto` - Common types shared across all services
- `main.proto` - Main Starknet API service
- `write.proto` - Write operations service  
- `trace.proto` - Transaction tracing service
- `ws.proto` - WebSocket/streaming service

## Package Structure

```
{common_package}     - Common types
{main_package}       - Main service
{write_package}      - Write service
{trace_package}      - Trace service
{ws_package}         - WebSocket service
```

## JSON Compatibility

All messages maintain 1:1 compatibility with the original JSON-RPC specification:

- Field names use `json_name` options to preserve exact JSON structure
- Optional fields map to the JSON optional behavior
- Enums preserve original string values
- Arrays map to `repeated` fields

## Multi-Language Support

This package provides first-class support for:

### 🚀 Quick Start
```bash
# Generate all language clients
./generate-clients.sh

# Or generate specific languages
buf generate --template buf.gen.go.yaml      # Go
buf generate --template buf.gen.rust.yaml    # Rust  
buf generate --template buf.gen.ts.yaml      # TypeScript
buf generate --template buf.gen.node.yaml    # Node.js
buf generate --template buf.gen.python.yaml  # Python
```

### 📦 Go
```go
package main

import (
    "context"
    "log"
    
    "google.golang.org/grpc"
    pb "github.com/cartridge-gg/starknet-grpc-codegen/go/{main_package_path}"
)

func main() {{
    conn, err := grpc.Dial("localhost:50051", grpc.WithInsecure())
    if err != nil {{
        log.Fatalf("Failed to connect: %v", err)
    }}
    defer conn.Close()
    
    client := pb.NewStarknetMainServiceClient(conn)
    
    resp, err := client.SpecVersion(context.Background(), &pb.SpecVersionRequest{{}})
    if err != nil {{
        log.Fatalf("SpecVersion failed: %v", err)
    }}
    
    log.Printf("Spec version: %s", resp.Result)
}}
```

**Dependencies:**
```bash
go mod init your-project
go get google.golang.org/grpc
go get google.golang.org/protobuf
```

### 🦀 Rust
```rust
use tonic::{{transport::Channel, Request}};

// Import generated types
use {main_package_rust}::{{
    starknet_main_service_client::StarknetMainServiceClient,
    SpecVersionRequest,
}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let channel = Channel::from_static("http://localhost:50051").connect().await?;
    let mut client = StarknetMainServiceClient::new(channel);
    
    let request = Request::new(SpecVersionRequest {{}});
    let response = client.spec_version(request).await?;
    
    println!("Spec version: {{}}", response.into_inner().result);
    Ok(())
}}
```

**Dependencies (Cargo.toml):**
```toml
[dependencies]
tonic = "0.11"
prost = "0.12"
tokio = {{ version = "1.0", features = ["macros", "rt-multi-thread"] }}
```

### 📜 TypeScript/JavaScript
```typescript
import {{ StarknetMainServiceClient }} from './gen/ts/main_grpc_pb';
import {{ SpecVersionRequest }} from './gen/ts/main_pb';

// For gRPC-Web (browser)
const client = new StarknetMainServiceClient('http://localhost:8080');

const request = new SpecVersionRequest();
client.specVersion(request, {{}}, (err, response) => {{
    if (err) {{
        console.error('Error:', err);
        return;
    }}
    console.log('Spec version:', response.getResult());
}});
```

**Dependencies:**
```bash
npm install grpc-web
npm install google-protobuf
npm install @types/google-protobuf  # For TypeScript
```

### 🟢 Node.js
```javascript
const grpc = require('@grpc/grpc-js');
const {{ StarknetMainServiceClient }} = require('./gen/node/main_grpc_pb');
const {{ SpecVersionRequest }} = require('./gen/node/main_pb');

const client = new StarknetMainServiceClient(
    'localhost:50051',
    grpc.credentials.createInsecure()
);

const request = new SpecVersionRequest();
client.specVersion(request, (err, response) => {{
    if (err) {{
        console.error('Error:', err);
        return;
    }}
    console.log('Spec version:', response.getResult());
}});
```

**Dependencies:**
```bash
npm install @grpc/grpc-js
npm install google-protobuf
```

### 🐍 Python
```python
import grpc
from gen.python import main_pb2, main_pb2_grpc

def main():
    with grpc.insecure_channel('localhost:50051') as channel:
        client = main_pb2_grpc.StarknetMainServiceStub(channel)
        
        request = main_pb2.SpecVersionRequest()
        response = client.SpecVersion(request)
        
        print(f"Spec version: {{response.result}}")

if __name__ == '__main__':
    main()
```

**Dependencies:**
```bash
pip install grpcio grpcio-tools
```

## Error Handling

All RPC methods include standard error responses with:
- `code` - Error code matching JSON-RPC error codes
- `message` - Human readable error message  
- `data` - Additional error context (optional)

## Streaming

Methods that support streaming (subscriptions, large responses) are marked with appropriate streaming options:

- `stream` request types for client streaming
- `stream` response types for server streaming
- Bidirectional streaming where applicable

## Development Workflow

1. **Generate clients**: Run `./generate-clients.sh`
2. **Integrate**: Copy generated code to your project
3. **Connect**: Use the appropriate gRPC client for your language
4. **Call methods**: All JSON-RPC methods are available as gRPC calls

## Package Options

The generated proto files include language-specific options:

- **Go**: `go_package` for proper Go module support
- **Java**: `java_package` and `java_multiple_files` for clean Java code
- **C#**: `csharp_namespace` for .NET integration
- **PHP**: `php_namespace` for PHP projects

---

Generated from Starknet JSON-RPC specification
Tool: https://github.com/cartridge-gg/starknet-grpc-codegen
"#, 
            common_package = package_info.common_package,
            main_package = package_info.main_package, 
            write_package = package_info.write_package,
            trace_package = package_info.trace_package,
            ws_package = package_info.ws_package,
            main_package_path = package_info.main_package.replace('.', "/"),
            main_package_rust = package_info.main_package.replace(['.', '-'], "_")
        );
        
        let readme_path = output_path.join("README.md");
        fs::write(readme_path, readme_content)?;
        
        Ok(())
    }
}

impl SpecVersion {
    fn to_version_string(self) -> String {
        match self {
            SpecVersion::V0_1_0 => "v0_1_0".to_string(),
            SpecVersion::V0_2_1 => "v0_2_1".to_string(),
            SpecVersion::V0_3_0 => "v0_3_0".to_string(),
            SpecVersion::V0_4_0 => "v0_4_0".to_string(),
            SpecVersion::V0_5_1 => "v0_5_1".to_string(),
            SpecVersion::V0_6_0 => "v0_6_0".to_string(),
            SpecVersion::V0_7_1 => "v0_7_1".to_string(),
            SpecVersion::V0_8_1 => "v0_8_1".to_string(),
        }
    }
}
