//! Worker assíncrono periódico para sincronização de telemetria via MQTT ou HTTPS.

use std::path::Path;
use std::time::Duration;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use crate::ring_buffer::DiskRingBuffer;

/// Worker assíncrono periódico de envio e sincronização de telemetria acumulada em disco.
pub struct SyncWorker {
    buffer: DiskRingBuffer,
    mqtt_host: Option<String>,
    mqtt_port: u16,
    http_endpoint: Option<String>,
}

impl SyncWorker {
    /// Inicializa o worker com base em limites de arquivo e endpoints remotos opcionais.
    pub fn new(
        file_path: &Path,
        mqtt_host: Option<&str>,
        mqtt_port: u16,
        http_endpoint: Option<&str>,
    ) -> Self {
        Self {
            buffer: DiskRingBuffer::new(file_path, 2 * 1024 * 1024), // 2MB limite de log
            mqtt_host: mqtt_host.map(str::to_string),
            mqtt_port,
            http_endpoint: http_endpoint.map(str::to_string),
        }
    }

    /// Loop assíncrono principal rodando a cada 30 segundos.
    pub async fn run(self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            tracing::info!("Iniciando ciclo de sincronização de telemetria...");

            match self.buffer.drain_batch(50) {
                Ok(points) => {
                    if points.is_empty() {
                        continue;
                    }

                    tracing::info!("Drenados {} pontos do disco. Tentando envio...", points.len());
                    let mut success = false;

                    // 1. Canal Primário: MQTT
                    if let Some(ref host) = self.mqtt_host {
                        if self.send_via_mqtt(host, self.mqtt_port, &points).await.is_ok() {
                            success = true;
                        }
                    }

                    // 2. Canal Secundário (Fallback): HTTP POST
                    if !success {
                        if let Some(ref url) = self.http_endpoint {
                            if self.send_via_http(url, &points).await.is_ok() {
                                success = true;
                            }
                        }
                    }

                    if success {
                        tracing::info!("Lote de telemetria sincronizado e removido do disco com sucesso!");
                    } else {
                        tracing::warn!("Falha de conexão com os servidores. Devolvendo lote ao disco.");
                        // Devolve os pontos ao buffer inserindo-os novamente
                        for p in points {
                            let _ = self.buffer.push(&p);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Erro de leitura no buffer de telemetria: {e}");
                }
            }
        }
    }

    /// Envia lote via protocolo leve MQTT.
    async fn send_via_mqtt(&self, host: &str, port: u16, points: &[crate::snapshot::TrackPoint]) -> Result<(), String> {
        let mut mqttoptions = MqttOptions::new("sgp-ciclobike-telemetry", host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        
        // Task para processar eventos do broker em background e manter conexão ativa
        tokio::spawn(async move {
            while let Ok(_notification) = eventloop.poll().await {}
        });

        let payload = serde_json::to_string(points).map_err(|e| e.to_string())?;

        // Tenta publicar no tópico de telemetria com QoS 1 (Garante entrega)
        client.publish("sgp/telemetry", QoS::AtLeastOnce, false, payload.as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        client.disconnect().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Envia lote via requisição padrão HTTPS POST.
    async fn send_via_http(&self, url: &str, points: &[crate::snapshot::TrackPoint]) -> Result<(), String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .map_err(|e| e.to_string())?;

        let res = client.post(url)
            .json(points)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(format!("HTTP Error: {}", res.status()))
        }
    }
}
