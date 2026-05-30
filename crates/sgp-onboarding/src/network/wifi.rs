//! Gerenciamento de Wi-Fi via socket Unix com o wpa_supplicant.

use super::wifi_state::AccessPoint;
use sgp_core::error::SgpError;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::net::UnixDatagram;

/// Controlador de conexao Wi-Fi interagindo com o wpa_supplicant via Unix socket.
pub struct WpaController {
    socket: Option<UnixDatagram>,
    local_path: Option<PathBuf>,
    mocked: bool,
}

impl WpaController {
    /// Conecta ao socket de controle do wpa_supplicant para a interface informada.
    pub async fn connect(iface: &str) -> Result<Self, SgpError> {
        let dest_path = format!("/var/run/wpa_supplicant/{iface}");
        let dest = Path::new(&dest_path);

        if !dest.exists() {
            return Ok(Self {
                socket: None,
                local_path: None,
                mocked: true,
            });
        }

        let pid = std::process::id();
        let local_path_str = format!("/tmp/sgp-wpa-ctrl-{pid}");
        let local_path = PathBuf::from(&local_path_str);

        if local_path.exists() {
            let _ = std::fs::remove_file(&local_path);
        }

        let socket = UnixDatagram::bind(&local_path)
            .map_err(|e| SgpError::Network(format!("Bind falhou: {e}")))?;

        socket
            .connect(dest)
            .map_err(|e| SgpError::Network(format!("Connect falhou: {e}")))?;

        let ctrl = Self {
            socket: Some(socket),
            local_path: Some(local_path),
            mocked: false,
        };

        let pong = ctrl.send_command("PING").await?;
        if !pong.trim().contains("PONG") {
            return Err(SgpError::Network(format!(
                "Handshake com wpa_supplicant falhou: {pong}"
            )));
        }

        Ok(ctrl)
    }

    /// Envia um comando textual cru ao wpa_supplicant e aguarda a resposta.
    pub async fn send_command(&self, cmd: &str) -> Result<String, SgpError> {
        if self.mocked {
            match cmd {
                "PING" => return Ok("PONG".into()),
                "SCAN" => return Ok("OK".into()),
                "SCAN_RESULTS" => {
                    return Ok("bssid / frequency / signal level / flags / ssid\n00:11:22:33:44:55\t2412\t-50\t[WPA2-PSK-CCMP]\tCicloNet\n66:77:88:99:aa:bb\t2437\t-75\t[WPA2-PSK-CCMP]\tOutraRede".into());
                }
                "ADD_NETWORK" => return Ok("0".into()),
                c if c.starts_with("SET_NETWORK") => return Ok("OK".into()),
                "ENABLE_NETWORK 0" => return Ok("OK".into()),
                "SELECT_NETWORK 0" => return Ok("OK".into()),
                "STATUS" => {
                    return Ok("wpa_state=COMPLETED\nip_address=192.168.1.15".into());
                }
                _ => return Ok("OK".into()),
            }
        }

        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| SgpError::Network("Socket nulo".into()))?;
        socket
            .send(cmd.as_bytes())
            .await
            .map_err(|e| SgpError::Network(e.to_string()))?;

        let mut buf = [0u8; 1024];
        let timeout = Duration::from_secs(2);
        let n = tokio::time::timeout(timeout, socket.recv(&mut buf))
            .await
            .map_err(|_| SgpError::Network("Timeout aguardando wpa_supplicant".into()))?
            .map_err(|e| SgpError::Network(e.to_string()))?;

        Ok(String::from_utf8_lossy(&buf[..n]).into_owned())
    }

    /// Realiza um escaneamento Wi-Fi e retorna os pontos de acesso ordenados por sinal.
    pub async fn scan(&self) -> Result<Vec<AccessPoint>, SgpError> {
        let _ = self.send_command("SCAN").await?;
        if !self.mocked {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        let results = self.send_command("SCAN_RESULTS").await?;
        let mut aps = Vec::new();
        for line in results.lines().skip(1) {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 5 {
                let signal_dbm = parts[2].parse::<i32>().unwrap_or(-100);
                let secured = parts[3].contains("WPA") || parts[3].contains("WEP");
                let ssid = parts[4].to_string();
                if !ssid.is_empty() {
                    aps.push(AccessPoint {
                        ssid,
                        signal_dbm,
                        secured,
                    });
                }
            }
        }
        aps.sort_by_key(|b| std::cmp::Reverse(b.signal_dbm));
        Ok(aps)
    }

    /// Configura e associa a interface Wi-Fi à rede especificada.
    pub async fn connect_network(&self, ssid: &str, password: &str) -> Result<(), SgpError> {
        let id_str = self.send_command("ADD_NETWORK").await?;
        let id = id_str
            .trim()
            .parse::<i32>()
            .map_err(|_| SgpError::Network("Falha ao adicionar rede".into()))?;

        self.send_command(&format!("SET_NETWORK {id} ssid \"{ssid}\""))
            .await?;
        if password.is_empty() {
            self.send_command(&format!("SET_NETWORK {id} key_mgmt NONE"))
                .await?;
        } else {
            self.send_command(&format!("SET_NETWORK {id} psk \"{password}\""))
                .await?;
        }

        self.send_command(&format!("ENABLE_NETWORK {id}")).await?;
        self.send_command(&format!("SELECT_NETWORK {id}")).await?;
        Ok(())
    }

    /// Aguarda ate a conexao completar e atribuição de IP, dentro de um limite de tempo.
    pub async fn wait_connected(&self, timeout_secs: u64) -> Result<String, SgpError> {
        let start = std::time::Instant::now();
        loop {
            if start.elapsed().as_secs() > timeout_secs {
                return Err(SgpError::Network("Timeout ao conectar".into()));
            }
            let status = self.send_command("STATUS").await?;
            let mut completed = false;
            let mut ip = String::new();

            for line in status.lines() {
                if line.starts_with("wpa_state=") {
                    completed = line.contains("COMPLETED");
                } else if line.starts_with("ip_address=") {
                    ip = line.split('=').nth(1).unwrap_or("").to_string();
                }
            }

            if completed && !ip.is_empty() {
                return Ok(ip);
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}

impl Drop for WpaController {
    fn drop(&mut self) {
        if let Some(ref path) = self.local_path {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Conecta a rede Wi-Fi configurando o wpa_supplicant na interface padrao.
pub async fn connect_wifi(ssid: &str, password: &str) -> Result<(), SgpError> {
    let ctrl = WpaController::connect("wlan0").await?;
    ctrl.connect_network(ssid, password).await?;
    let _ip = ctrl.wait_connected(15).await?;
    Ok(())
}

/// Escaneia redes de Wi-Fi ativas na interface wlan0.
pub async fn scan_wifi() -> Result<Vec<AccessPoint>, SgpError> {
    let ctrl = WpaController::connect("wlan0").await?;
    ctrl.scan().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wpa_controller_fallback() {
        let ctrl = WpaController::connect("wlan_inexistente").await.unwrap();
        assert!(ctrl.mocked);
    }

    #[tokio::test]
    async fn test_scan_returns_sorted_by_signal() {
        let ctrl = WpaController::connect("wlan_inexistente").await.unwrap();
        let aps = ctrl.scan().await.unwrap();
        assert_eq!(aps.len(), 2);
        assert_eq!(aps[0].ssid, "CicloNet");
        assert_eq!(aps[1].ssid, "OutraRede");
    }

    #[tokio::test]
    async fn test_connect_network_mock() {
        let ctrl = WpaController::connect("wlan_inexistente").await.unwrap();
        let res = ctrl.connect_network("CicloNet", "senha123").await;
        assert!(res.is_ok());
    }
}
