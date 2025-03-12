#[cfg(all(
    feature = "core",
    not(feature = "services-core"),
    not(feature = "services-request")
))]
compile_error!("Feature `service-core` or `service-request` must be enabled to use `core`");

#[cfg(feature = "core")]
mod instance;
#[cfg(feature = "core")]
pub use instance::*;

#[cfg(feature = "core")]
mod profile;
#[cfg(feature = "core")]
pub use profile::*;

#[cfg(feature = "core")]
mod config;
#[cfg(feature = "core")]
pub use config::*;

#[cfg(feature = "core")]
mod router;
#[cfg(feature = "core")]
pub use router::*;

pub mod schema;

pub use atom_services;
