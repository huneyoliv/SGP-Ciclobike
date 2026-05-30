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
pub mod secrets;

pub use config::{
    BikeConfig, CountryCode, LanguageCode, OnboardingProgress, OtaChannel, OtaRelease, SensorId,
    UsbModemPath,
};
pub use error::{ConfigError, OtaError, SgpError};
pub use secrets::{strava_app_configured, StravaTokens, STRAVA_CLIENT_ID, STRAVA_CLIENT_SECRET};
