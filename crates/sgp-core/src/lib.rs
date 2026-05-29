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
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_panics_doc,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::doc_markdown,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::ptr_as_ptr,
    clippy::collapsible_if,
    clippy::result_large_err,
    clippy::match_same_arms
)]

pub mod config;
pub mod error;

// Re-exports convenientes do domínio público
pub use config::{
    BikeConfig, CountryCode, LanguageCode, OnboardingProgress, SensorId, UsbModemPath,
    OtaChannel, OtaRelease,
};
pub use error::{ConfigError, OtaError, SgpError};
