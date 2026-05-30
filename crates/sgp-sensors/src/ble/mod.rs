//! Módulo para comunicação com sensores Bluetooth Low Energy.

/// Driver para sensor de cadência BLE.
pub mod cadence;
/// Abstração de baixo nível para D-Bus e GATT.
pub mod gatt;
/// Driver para sensor de frequência cardíaca BLE.
pub mod heart_rate;
