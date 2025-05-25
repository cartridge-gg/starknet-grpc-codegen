# Starknet gRPC Protocol Buffers

This directory contains auto-generated Protocol Buffer definitions for the Starknet JSON-RPC API.

## Generated Files

- `common.proto` - Common types shared across all services
- `main.proto` - Main Starknet API service
- `write.proto` - Write operations service  
- `trace.proto` - Transaction tracing service
- `ws.proto` - WebSocket/streaming service

## Package Structure

```
starknet.v0_8_1.common     - Common types
starknet.v0_8_1.main       - Main service
starknet.v0_8_1.write      - Write service
starknet.v0_8_1.trace      - Trace service
starknet.v0_8_1.ws         - WebSocket service
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
    pb "github.com/cartridge-gg/starknet-grpc-codegen/go/starknet/v0_8_1/main"
)

func main() {
    conn, err := grpc.Dial("localhost:50051", grpc.WithInsecure())
    if err != nil {
        log.Fatalf("Failed to connect: %v", err)
    }
    defer conn.Close()
    
    client := pb.NewStarknetMainServiceClient(conn)
    
    resp, err := client.SpecVersion(context.Background(), &pb.SpecVersionRequest{})
    if err != nil {
        log.Fatalf("SpecVersion failed: %v", err)
    }
    
    log.Printf("Spec version: %s", resp.Result)
}
```

**Dependencies:**
```bash
go mod init your-project
go get google.golang.org/grpc
go get google.golang.org/protobuf
```

### 🦀 Rust
```rust
use tonic::{transport::Channel, Request};

// Import generated types
use starknet_v0_8_1_main::{
    starknet_main_service_client::StarknetMainServiceClient,
    SpecVersionRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://localhost:50051").connect().await?;
    let mut client = StarknetMainServiceClient::new(channel);
    
    let request = Request::new(SpecVersionRequest {});
    let response = client.spec_version(request).await?;
    
    println!("Spec version: {}", response.into_inner().result);
    Ok(())
}
```

**Dependencies (Cargo.toml):**
```toml
[dependencies]
tonic = "0.11"
prost = "0.12"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

### 📜 TypeScript/JavaScript
```typescript
import { StarknetMainServiceClient } from './gen/ts/main_grpc_pb';
import { SpecVersionRequest } from './gen/ts/main_pb';

// For gRPC-Web (browser)
const client = new StarknetMainServiceClient('http://localhost:8080');

const request = new SpecVersionRequest();
client.specVersion(request, {}, (err, response) => {
    if (err) {
        console.error('Error:', err);
        return;
    }
    console.log('Spec version:', response.getResult());
});
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
const { StarknetMainServiceClient } = require('./gen/node/main_grpc_pb');
const { SpecVersionRequest } = require('./gen/node/main_pb');

const client = new StarknetMainServiceClient(
    'localhost:50051',
    grpc.credentials.createInsecure()
);

const request = new SpecVersionRequest();
client.specVersion(request, (err, response) => {
    if (err) {
        console.error('Error:', err);
        return;
    }
    console.log('Spec version:', response.getResult());
});
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
        
        print(f"Spec version: {response.result}")

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
