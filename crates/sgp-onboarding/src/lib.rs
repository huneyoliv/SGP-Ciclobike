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

pub mod state_machine;
pub mod persistence;
pub mod network;
pub mod input;

pub use state_machine::{OnboardingEvent, OnboardingState, TransitionError, transition};
pub use persistence::{ConfigGuard, resume_state, CONFIG_PATH};
pub use input::{TouchReader, TouchEvent};
