pub mod spec;
pub mod proto_gen;

// Re-export commonly used types for convenience
pub use spec::*;
pub use proto_gen::{ProtoConfig, ProtoGenerator}; 