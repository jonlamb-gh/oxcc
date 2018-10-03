//! Vehicle abstraction
//!
//! Pick your vehicle feature set in the `Cargo.toml`
#[cfg(feature = "kia-niro")]
pub use kial_niro::*;
#[cfg(feature = "kia-soul-ev")]
pub use kial_soul_ev::*;
#[cfg(feature = "kia-soul-petrol")]
pub use kial_soul_petrol::*;
