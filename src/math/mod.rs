pub mod caffeine;
pub mod sleep;

// Re-export commonly used items so users don't need to know the internal structure
pub use caffeine::*;
pub use sleep::*;
