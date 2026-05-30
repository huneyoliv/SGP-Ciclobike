//! # sgp-onboarding
//!
//! Módulo responsável pelo fluxo de configuração inicial (Onboarding Wizard)
//! do ciclocomputador SGP-Ciclobike.

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

pub mod input;
pub mod network;
pub mod persistence;
pub mod state_machine;

pub use input::{TouchEvent, TouchReader};
pub use network::wifi::{connect_wifi, scan_wifi, WpaController};
pub use network::wifi_state::{AccessPoint, WifiEvent, WifiPhase};
pub use persistence::{resume_state, ConfigGuard, CONFIG_PATH};
pub use state_machine::{transition, OnboardingEvent, OnboardingState, TransitionError};
