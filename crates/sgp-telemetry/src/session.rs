//! Gerenciador de ciclo de pedalada ativo e exportação de relatórios estatísticos.

use std::path::Path;
use std::time::{Duration, Instant};
use uuid::Uuid;
use crate::snapshot::TrackPoint;
use crate::ring_buffer::DiskRingBuffer;

/// Sumário final gerado ao término de uma sessão de pedalada.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionSummary {
    /// Identificador global único da sessão encerrada.
    pub session_id: Uuid,
    /// Duração total acumulada da sessão em segundos.
    pub duration_seconds: u32,
    /// Distância total percorrida em quilômetros.
    pub total_distance_km: f32,
    /// Velocidade média mantida durante o movimento (em km/h).
    pub average_speed_kmh: f32,
    /// Velocidade máxima registrada (em km/h).
    pub max_speed_kmh: f32,
    /// Número de trackpoints gerados e gravados em disco.
    pub total_points_recorded: u32,
}

/// Estados lógicos possíveis para o ciclo de vida de uma sessão.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Sem nenhuma atividade ativa.
    Idle,
    /// Pedalada ativa sendo gravada sequencialmente.
    Recording {
        /// UUID associado a este trajeto.
        id: Uuid,
        /// Instante físico do início do pedal.
        started_at: Instant,
    },
    /// Gravação suspensa temporariamente pelo usuário.
    Paused,
}

/// Gerenciador de sessão ativo acoplado ao buffer circular de armazenamento.
pub struct SessionManager {
    state: SessionState,
    buffer: DiskRingBuffer,
    points_recorded: Vec<TrackPoint>,
}

impl SessionManager {
    /// Inicializa a interface de sessão vinculando-a a um arquivo físico em disco.
    pub fn new(file_path: &Path) -> Self {
        Self {
            state: SessionState::Idle,
            buffer: DiskRingBuffer::new(file_path, 2 * 1024 * 1024),
            points_recorded: Vec::new(),
        }
    }

    /// Inicia uma nova gravação gerando um identificador de sessão.
    pub fn start_session(&mut self) -> Uuid {
        let id = Uuid::new_v4();
        self.state = SessionState::Recording {
            id,
            started_at: Instant::now(),
        };
        self.points_recorded.clear();
        tracing::info!("Nova sessão de pedalada iniciada: {id}");
        id
    }

    /// Suspende temporariamente o monitoramento e a gravação.
    pub fn pause_session(&mut self) {
        if matches!(self.state, SessionState::Recording { .. }) {
            self.state = SessionState::Paused;
            tracing::info!("Sessão de pedalada pausada.");
        }
    }

    /// Retoma a gravação de uma sessão anteriormente suspensa.
    pub fn resume_session(&mut self) {
        if self.state == SessionState::Paused {
            self.state = SessionState::Recording {
                id: Uuid::new_v4(),
                started_at: Instant::now(),
            };
            tracing::info!("Sessão de pedalada retomada.");
        }
    }

    /// Captura e grava um snapshot instantâneo de sensores no arquivo circular.
    pub fn record_point(
        &mut self,
        lat: Option<f64>,
        lon: Option<f64>,
        altitude_m: Option<f32>,
        speed_kmh: f32,
        cadence_rpm: f32,
        accel_magnitude: f32,
    ) -> Result<(), String> {
        if let SessionState::Recording { id, .. } = self.state {
            let point = TrackPoint {
                session_id: id,
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or(Duration::ZERO)
                    .as_millis() as i64,
                lat,
                lon,
                altitude_m,
                speed_kmh,
                cadence_rpm,
                accel_magnitude,
            };

            // Salva na RAM para cômputo rápido de estatísticas no final
            self.points_recorded.push(point.clone());
            // Salva no disco de forma circular persistente
            self.buffer.push(&point)?;
        }
        Ok(())
    }

    /// Encerra a pedalada ativa e gera o sumário analítico de telemetria.
    pub fn stop_session(&mut self) -> Result<SessionSummary, String> {
        let (id, duration) = match self.state {
            SessionState::Recording { id, started_at } => (id, started_at.elapsed()),
            _ => return Err("Nenhuma sessão de pedalada ativa para ser parada.".to_string()),
        };

        let total_points = self.points_recorded.len() as u32;
        let mut max_speed = 0.0f32;
        let mut speed_sum = 0.0f32;
        let mut distance_km = 0.0f32;

        let mut last_point: Option<&TrackPoint> = None;

        for p in &self.points_recorded {
            if p.speed_kmh > max_speed {
                max_speed = p.speed_kmh;
            }
            speed_sum += p.speed_kmh;

            if let Some(last) = last_point {
                // Cálculo simplificado de distância acumulada por variação de tempo
                let time_diff_hours = (p.timestamp_ms - last.timestamp_ms) as f32 / 3_600_000.0;
                distance_km += p.speed_kmh * time_diff_hours;
            }
            last_point = Some(p);
        }

        let avg_speed = if total_points > 0 {
            speed_sum / total_points as f32
        } else {
            0.0
        };

        let summary = SessionSummary {
            session_id: id,
            duration_seconds: duration.as_secs() as u32,
            total_distance_km: distance_km,
            average_speed_kmh: avg_speed,
            max_speed_kmh: max_speed,
            total_points_recorded: total_points,
        };

        self.state = SessionState::Idle;
        self.points_recorded.clear();
        tracing::info!("Sessão encerrada com sucesso. Total percorrido: {:.2} km", summary.total_distance_km);

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_flow() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("session_test.json");
        let _ = std::fs::remove_file(&path);

        let mut manager = SessionManager::new(&path);
        let id = manager.start_session();

        manager.record_point(Some(1.0), Some(2.0), Some(10.0), 20.0, 80.0, 9.8).unwrap();
        // Simula passagem de tempo inserindo ponto posterior
        std::thread::sleep(Duration::from_millis(50));
        manager.record_point(Some(1.01), Some(2.01), Some(11.0), 24.0, 85.0, 9.8).unwrap();

        let summary = manager.stop_session().unwrap();
        assert_eq!(summary.session_id, id);
        assert_eq!(summary.total_points_recorded, 2);
        assert!((summary.max_speed_kmh - 24.0).abs() < 1e-5);
        assert!((summary.average_speed_kmh - 22.0).abs() < 1e-5);

        // Limpa arquivo
        let _ = std::fs::remove_file(&path);
    }
}
