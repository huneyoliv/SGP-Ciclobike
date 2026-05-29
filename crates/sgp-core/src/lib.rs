//! # sgp-core
//!
//! Tipos de domínio, erros e configuração compartilhada do SGP-Ciclobike.
//! Esta crate é a fundação do sistema — todas as outras crates dependem dela.
//!
//! ## Módulos
//! - [`config`]: Estruturas de configuração persistidas em `/etc/bike_config.toml`
//! - [`error`]: Hierarquia de erros tipados com `thiserror`

#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod config;
pub mod error;

// Re-exports convenientes do domínio público
pub use config::{
    BikeConfig, CountryCode, LanguageCode, OnboardingProgress, SensorId, UsbModemPath,
};
pub use error::{ConfigError, OtaError, SgpError};
