//! Snapshots de telemetria coletados em tempo de atividade.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Registro geográfico e físico unificado em um instante de tempo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackPoint {
    /// Identificador global único da sessão de pedalada ativa.
    pub session_id: Uuid,
    /// Timestamp Unix em milissegundos.
    pub timestamp_ms: i64,
    /// Latitude em graus decimais (opcional se sem sinal de satélite).
    pub lat: Option<f64>,
    /// Longitude em graus decimais (opcional).
    pub lon: Option<f64>,
    /// Altitude em metros (opcional).
    pub altitude_m: Option<f32>,
    /// Velocidade estimada instantânea da bike (em km/h).
    pub speed_kmh: f32,
    /// Rotações por minuto da roda da bike (estimado ou via sensor de cadência).
    pub cadence_rpm: f32,
    /// Força da aceleração inercial máxima registrada (em m/s²).
    pub accel_magnitude: f32,
    /// Frequência cardíaca em BPM obtida via sensor BLE (opcional).
    #[serde(default)]
    pub heart_rate: Option<u8>,
}
