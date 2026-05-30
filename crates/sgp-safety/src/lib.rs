//! # sgp-safety
//!
//! Módulo de segurança ativa contendo algoritmos de detecção de queda,
//! janela de alerta e envio de mensagens e chamadas de emergência.

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

pub mod detector;
pub mod alert;
pub mod emergency;

pub use detector::{FallDetector, FallEvent, ImuSample};
pub use alert::{AlertInput, AlertManager, AlertState};
pub use emergency::EmergencyDispatcher;
