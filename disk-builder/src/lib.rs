//! only reason this exists is that adding bootloader as a dependency for 
//! the root dependencies and build-dependencies introduces 
//! rust-analyzer errors that are really annoying

pub use bootloader::DiskImageBuilder;