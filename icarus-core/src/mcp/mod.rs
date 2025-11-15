// MCP Server Module for Icarus
// Exposes Icarus cognitive capabilities via Model Context Protocol

pub mod protocol;
pub mod server;
pub mod stdio;

pub use protocol::*;
pub use server::IcarusMCPServer;
pub use stdio::StdioHandler;
