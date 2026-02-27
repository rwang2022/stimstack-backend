pub mod caffeine;
pub mod crash;
pub mod sleep;

// Re-export commonly used items so users don't need to know the internal structure
pub use caffeine::*;
pub use crash::*;
pub use sleep::*;
