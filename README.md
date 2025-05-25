# Starknet gRPC Proto Codegen

Tool for generating gRPC protobuf definitions from Starknet JSON-RPC specifications. This tool converts Starknet JSON-RPC specs into `.proto` files with full JSON compatibility, enabling multi-language client generation and gRPC-based Starknet integrations.

## Features

- **1:1 JSON Compatibility**: Generated proto files preserve exact JSON field names using `json_name` options
- **Service Separation**: Generates separate services for main, write, trace, and WebSocket operations
- **Multi-language Support**: Standard protobuf output enables client generation for any language
- **Version Management**: Organized by specification version with proper package namespacing
- **Streaming Support**: Automatic detection and implementation of streaming methods for subscriptions

## Usage

Run the tool and choose which version of the specification to use:

```console
$ cargo run -- generate --spec 0.8.1
```

Generated proto files will be written to the `proto/` directory with the following structure:

```
proto/
├── v0_8_1/
│   ├── main.proto      # Main Starknet operations
│   ├── write.proto     # Write operations (transactions)
│   ├── trace.proto     # Trace operations
│   ├── ws.proto        # WebSocket/streaming operations
│   └── common.proto    # Shared types and messages
├── buf.yaml            # Buf configuration for proto management
└── README.md           # Generated documentation
```

## Example Output

The generated proto files include:

- **gRPC Services**: Each JSON-RPC method becomes a gRPC service method
- **Message Types**: Request/response pairs with JSON-compatible field mapping
- **Shared Types**: Common Starknet types (blocks, transactions, etc.) in `common.proto`
- **Language Options**: Package names and namespaces for Java, Go, C#, and PHP

Example service definition:
```protobuf
service StarknetMainService {
  rpc GetBlockWithTxHashes(GetBlockWithTxHashesRequest) returns (GetBlockWithTxHashesResponse);
  rpc Call(CallRequest) returns (CallResponse);
  // ... more methods
}
```

## Supported spec versions

The following versions are supported:

- `0.1.0`
- `0.2.1`
- `0.3.0`
- `0.4.0`
- `0.5.1`
- `0.6.0`
- `0.7.1`
- `0.8.1`

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
