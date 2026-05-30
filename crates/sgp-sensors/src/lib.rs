//! # sgp-sensors
//!
//! Drivers e abstrações para leitura de sensores via I2C, GPIO e Serial.

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

pub mod traits;
pub mod imu;
pub mod speed;
pub mod gps;
pub mod mock;

pub use traits::{SensorData, SensorError, SensorReader};
pub use imu::Mpu6050Driver;
pub use speed::ReedSwitchDriver;
pub use gps::NmeaGpsDriver;
pub use mock::MockSensor;
