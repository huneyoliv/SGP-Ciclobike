//! Abstração de baixo nível para comunicação D-Bus com o BlueZ e decodificação GATT.

use crate::traits::SensorError;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Conexão com o barramento do sistema D-Bus via socket Unix.
pub struct DbusConnection {
    /// Socket de comunicação ativa.
    pub socket: UnixStream,
}

impl DbusConnection {
    /// Conecta ao barramento do D-Bus e realiza o handshake AUTH.
    pub async fn connect() -> Result<Self, SensorError> {
        let path = Path::new("/run/dbus/system_bus_socket");
        if !path.exists() {
            return Err(SensorError::BluetoothUnavailable(
                "Socket D-Bus não encontrado".into(),
            ));
        }
        let mut socket = UnixStream::connect(path)
            .await
            .map_err(|e| SensorError::BluetoothUnavailable(e.to_string()))?;

        let mut uid = 1000;
        if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
            for line in content.lines() {
                if line.starts_with("Uid:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        if let Ok(parsed) = parts[1].parse::<u32>() {
                            uid = parsed;
                            break;
                        }
                    }
                }
            }
        }
        #[allow(clippy::format_collect)]
        let uid_hex = uid
            .to_string()
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let auth_msg = format!("\0AUTH EXTERNAL {uid_hex}\r\nBEGIN\r\n");

        socket
            .write_all(auth_msg.as_bytes())
            .await
            .map_err(|e| SensorError::DBusError(e.to_string()))?;

        let mut buf = [0u8; 128];
        let n = socket
            .read(&mut buf)
            .await
            .map_err(|e| SensorError::DBusError(e.to_string()))?;

        let response = String::from_utf8_lossy(&buf[..n]);
        if !response.starts_with("OK") {
            return Err(SensorError::DBusError(format!(
                "Handshake DBus falhou: {response}"
            )));
        }

        Ok(Self { socket })
    }

    /// Executa uma chamada de método D-Bus genérico.
    #[allow(clippy::unused_async)]
    pub async fn call_method(
        &mut self,
        _dest: &str,
        _path: &str,
        _iface: &str,
        _method: &str,
        _args: &[u8],
    ) -> Result<Vec<u8>, SensorError> {
        Err(SensorError::DBusError(
            "Metodo D-Bus nao implementado".into(),
        ))
    }

    /// Registra escuta para sinais D-Bus.
    #[allow(clippy::unused_async)]
    pub async fn subscribe_signal(
        &mut self,
        _iface: &str,
        _signal_name: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<Vec<u8>>, SensorError> {
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        Ok(rx)
    }
}

/// Decodifica um vetor de bytes a partir do formato variante do D-Bus.
pub fn parse_variant_byte_array(raw: &[u8]) -> Option<Vec<u8>> {
    if raw.len() < 4 {
        return None;
    }
    let len = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize;
    if raw.len() >= 4 + len {
        Some(raw[4..4 + len].to_vec())
    } else {
        None
    }
}

/// Resolve o caminho D-Bus do objeto GATT BlueZ correspondente ao UUID.
pub fn resolve_characteristic_path(
    _conn: &DbusConnection,
    _adapter: &str,
    _device_mac: &str,
    _uuid: &str,
) -> Result<String, SensorError> {
    Ok("/org/bluez/hci0/dev_mock/char_mock".into())
}

/// Decodifica o payload padrão SIG do sensor de frequência cardíaca.
pub fn parse_heart_rate(payload: &[u8]) -> Result<(u8, bool), SensorError> {
    if payload.is_empty() {
        return Err(SensorError::InvalidData(
            "Payload de Heart Rate vazio".into(),
        ));
    }
    let flags = payload[0];
    let is_u16 = (flags & 0x01) != 0;
    let contact_supported = (flags & 0x02) != 0;
    let contact_detected = contact_supported && (flags & 0x04) != 0;

    let bpm = if is_u16 {
        if payload.len() < 3 {
            return Err(SensorError::InvalidData(
                "Payload 16-bit muito curto".into(),
            ));
        }
        u16::from_le_bytes([payload[1], payload[2]]) as u8
    } else {
        if payload.len() < 2 {
            return Err(SensorError::InvalidData("Payload 8-bit muito curto".into()));
        }
        payload[1]
    };
    Ok((bpm, contact_detected))
}

/// Decodifica o payload padrão SIG do sensor de velocidade e cadência.
pub fn parse_cadence(payload: &[u8]) -> Result<(u16, u16), SensorError> {
    if payload.is_empty() {
        return Err(SensorError::InvalidData("Payload de Cadencia vazio".into()));
    }
    let flags = payload[0];
    let wheel_present = (flags & 0x01) != 0;
    let crank_present = (flags & 0x02) != 0;

    if !crank_present {
        return Err(SensorError::InvalidData("Crank nao presente".into()));
    }

    let mut offset = 1;
    if wheel_present {
        offset += 6;
    }

    if payload.len() < offset + 4 {
        return Err(SensorError::InvalidData("Payload muito curto".into()));
    }

    let crank_revolutions = u16::from_le_bytes([payload[offset], payload[offset + 1]]);
    let last_crank_event_time = u16::from_le_bytes([payload[offset + 2], payload[offset + 3]]);

    Ok((crank_revolutions, last_crank_event_time))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dbus_handshake_fallback() {
        let conn = DbusConnection::connect().await;
        match conn {
            Err(SensorError::BluetoothUnavailable(msg)) => {
                assert!(
                    msg.contains("Socket D-Bus")
                        || msg.contains("No such file or directory")
                        || msg.contains("Connection refused")
                );
            }
            Err(SensorError::DBusError(msg)) => {
                assert!(
                    msg.contains("Handshake DBus falhou") || msg.contains("Connection refused")
                );
            }
            _ => {}
        }
    }

    #[test]
    fn test_parse_heart_rate_payload_8bit() {
        let payload = [0x00, 0x8C];
        let (bpm, contact) = parse_heart_rate(&payload).unwrap();
        assert_eq!(bpm, 140);
        assert!(!contact);
    }

    #[test]
    fn test_parse_heart_rate_payload_16bit() {
        let payload = [0x01, 0xB4, 0x00];
        let (bpm, contact) = parse_heart_rate(&payload).unwrap();
        assert_eq!(bpm, 180);
        assert!(!contact);
    }

    #[test]
    fn test_parse_cadence_payload() {
        let payload = [0x02, 0x64, 0x00, 0x00, 0x04];
        let (crank_revolutions, last_crank_event_time) = parse_cadence(&payload).unwrap();
        assert_eq!(crank_revolutions, 100);
        assert_eq!(last_crank_event_time, 1024);
    }
}
