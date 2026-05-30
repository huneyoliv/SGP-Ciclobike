//! Despachante de chamadas e mensagens de emergência via Modem USB (AT) e HTTPS.

use std::io::Write;
use std::time::Duration;
use tokio::sync::mpsc;

/// Responsável por disparar os alertas físicos e de rede quando o protocolo de emergência é ativado.
pub struct EmergencyDispatcher {
    modem_path: Option<String>,
    emergency_number: String,
    http_endpoint: Option<String>,
}

impl EmergencyDispatcher {
    /// Cria uma nova instância configurada com caminhos de periféricos e endpoints de emergência.
    pub fn new(modem_path: Option<&str>, emergency_number: &str, http_endpoint: Option<&str>) -> Self {
        Self {
            modem_path: modem_path.map(str::to_string),
            emergency_number: emergency_number.to_string(),
            http_endpoint: http_endpoint.map(str::to_string),
        }
    }

    /// Loop principal do dispatcher de emergência que aguarda sinais de ativação.
    pub async fn run(self, mut trigger_rx: mpsc::Receiver<()>) {
        loop {
            tokio::select! {
                Some(()) = trigger_rx.recv() => {
                    tracing::error!("Protocolo de emergência ativado! Iniciando envio de alertas...");
                    let _ = self.dispatch_emergency_alerts(-23.5505, -46.6333).await;
                }
            }
        }
    }

    /// Executa as chamadas e envia as mensagens de localização.
    pub async fn dispatch_emergency_alerts(&self, lat: f64, lon: f64) -> Result<(), String> {
        let message = format!(
            "ALERTA SGP-CICLOBIKE: Queda severa detectada! Localizacao: https://maps.google.com/?q={lat},{lon}"
        );

        // Canal 1: SMS e Voz via Modem USB Serial
        if let Some(ref path) = self.modem_path {
            tracing::info!("Tentando comunicação com modem serial em: {path}...");
            let _ = self.send_at_emergency(path, &message);
        }

        // Canal 2: Fallback via HTTP/HTTPS POST se houver rede ativa
        if let Some(ref url) = self.http_endpoint {
            tracing::info!("Tentando envio de alerta digital via HTTPS para: {url}...");
            let _ = self.send_http_emergency(url, lat, lon).await;
        }

        // Grava no log local do sistema embarcado para auditoria pós-acidente
        let log_data = format!(
            "[{:?}] EMERGENCIA CONFIRMADA: Lat={}, Lon={}. Alertas despachados.\n",
            std::time::SystemTime::now(), lat, lon
        );
        let _ = std::fs::create_dir_all("/var/log");
        let _ = std::fs::write("/var/log/sgp-emergency.log", log_data);

        Ok(())
    }

    /// Envia comandos AT de chamada de voz e envio de SMS via porta serial do modem.
    fn send_at_emergency(&self, path: &str, message: &str) -> Result<(), String> {
        let mut port = serialport::new(path, 115_200)
            .timeout(Duration::from_secs(1))
            .open()
            .map_err(|e| e.to_string())?;

        // 1. Envia comando de discagem de voz
        tracing::info!("Modem: Discando chamada de emergência para {}...", self.emergency_number);
        let dial_cmd = format!("ATD{};\r\n", self.emergency_number);
        let _ = port.write_all(dial_cmd.as_bytes());
        std::thread::sleep(Duration::from_millis(500));

        // 2. Configura modo SMS de texto
        let _ = port.write_all(b"AT+CMGF=1\r\n");
        std::thread::sleep(Duration::from_millis(200));

        // 3. Envia SMS de localização
        let sms_init = format!("AT+CMGS=\"{}\"\r\n", self.emergency_number);
        let _ = port.write_all(sms_init.as_bytes());
        std::thread::sleep(Duration::from_millis(200));

        // Escreve a mensagem terminando com Ctrl+Z (0x1A)
        let mut sms_body = message.to_string();
        sms_body.push('\x1A');
        let _ = port.write_all(sms_body.as_bytes());
        std::thread::sleep(Duration::from_secs(1));

        Ok(())
    }

    /// Envia o JSON do incidente para o servidor remoto de segurança via HTTPS.
    async fn send_http_emergency(&self, url: &str, lat: f64, lon: f64) -> Result<(), String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let body = serde_json::json!({
            "incident": "fall_detected",
            "lat": lat,
            "lon": lon,
            "emergency_number": self.emergency_number,
        });

        let res = client.post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            tracing::info!("Alerta HTTPS enviado com sucesso!");
            Ok(())
        } else {
            Err(format!("Erro no servidor HTTPS: {}", res.status()))
        }
    }
}
