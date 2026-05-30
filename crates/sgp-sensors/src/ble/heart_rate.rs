//! Driver BLE para sensor de frequencia cardiaca.

#[cfg(feature = "ble-hardware")]
use crate::traits::{SensorData, SensorError, SensorReader};

/// Driver de frequencia cardiaca BLE.
#[cfg(feature = "ble-hardware")]
pub struct HeartRateDriver {
    rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
}

#[cfg(feature = "ble-hardware")]
impl HeartRateDriver {
    /// Conecta ao sensor de frequencia cardiaca BLE.
    pub async fn connect(device_mac: &str) -> Result<Self, SensorError> {
        let mut conn = super::gatt::DbusConnection::connect().await?;
        let _path = super::gatt::resolve_characteristic_path(&conn, "hci0", device_mac, "180d")?;
        let rx = conn
            .subscribe_signal("org.bluez.GattCharacteristic1", "PropertiesChanged")
            .await?;
        Ok(Self { rx })
    }
}

#[cfg(feature = "ble-hardware")]
impl SensorReader for HeartRateDriver {
    async fn read(&mut self) -> Result<SensorData, SensorError> {
        if let Some(payload) = self.rx.recv().await {
            let (bpm, contact_detected) = super::gatt::parse_heart_rate(&payload)?;
            Ok(SensorData::HeartRate {
                bpm,
                contact_detected,
            })
        } else {
            Err(SensorError::DBusError("Notificacoes BLE pararam".into()))
        }
    }
}
