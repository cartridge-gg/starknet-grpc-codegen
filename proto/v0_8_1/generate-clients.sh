#!/bin/bash
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
