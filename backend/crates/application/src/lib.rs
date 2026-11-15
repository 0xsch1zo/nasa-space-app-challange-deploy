//! Application layer: use cases, ports and authorization rules.
//!
//! * `ports::inbound`  – what driving adapters (HTTP, device gateways, seeders) may call.
//! * `ports::outbound` – what this layer needs from the outside world (DB, JWT, hashing).
//! * `security`        – the access policy, enforced inside every use case so it
//!   cannot be bypassed by a new route or a different adapter.

pub mod error;
pub mod ports;
pub mod security;
pub mod services;

pub use error::AppError;
