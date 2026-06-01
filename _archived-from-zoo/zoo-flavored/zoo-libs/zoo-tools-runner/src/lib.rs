//! Zoo Tools Runner - Thin wrapper around hanzo_tools_runner
//!
//! This crate re-exports hanzo_tools_runner with zoo-specific extensions.
//! Follows DRY principle - single source of truth from hanzo implementation.

// Re-export everything from hanzo_tools_runner at the root level
pub use hanzo_tools_runner::*;

// Re-export tools module with zoo extensions
pub mod tools {
    // Re-export all hanzo tools
    pub use hanzo_tools_runner::tools::*;

    // Add zoo-specific zoo_node_location module
    pub mod zoo_node_location {
        // Re-export HanzoNodeLocation as ZooNodeLocation (type alias)
        pub use hanzo_tools_runner::tools::hanzo_node_location::HanzoNodeLocation as ZooNodeLocation;
    }
}
