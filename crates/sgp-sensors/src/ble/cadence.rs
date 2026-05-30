//! Driver BLE para sensor de cadencia.

#[cfg(feature = "ble-hardware")]
use crate::traits::{SensorData, SensorError, SensorReader};

/// Driver de cadencia BLE.
#[cfg(feature = "ble-hardware")]
pub struct CadenceDriver {
    rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
}

#[cfg(feature = "ble-hardware")]
impl CadenceDriver {
    /// Conecta ao sensor de cadencia BLE.
    pub async fn connect(device_mac: &str) -> Result<Self, SensorError> {
        let mut conn = super::gatt::DbusConnection::connect().await?;
        let _path = super::gatt::resolve_characteristic_path(&conn, "hci0", device_mac, "1816")?;
        let rx = conn
            .subscribe_signal("org.bluez.GattCharacteristic1", "PropertiesChanged")
            .await?;
        Ok(Self { rx })
    }
}

#[cfg(feature = "ble-hardware")]
impl SensorReader for CadenceDriver {
    async fn read(&mut self) -> Result<SensorData, SensorError> {
        if let Some(payload) = self.rx.recv().await {
            let (crank_revolutions, last_crank_event_time) = super::gatt::parse_cadence(&payload)?;
            Ok(SensorData::Cadence {
                crank_revolutions,
                last_crank_event_time,
            })
        } else {
            Err(SensorError::DBusError("Notificacoes BLE pararam".into()))
        }
    }
}
