//! Definição de tipos, contratos e erros unificados para sensores do ciclocomputador.

use serde::{Deserialize, Serialize};

/// Estrutura de dados unificada retornada por qualquer leitura de sensor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SensorData {
    /// Snapshots de aceleração e giroscópio da IMU.
    Imu {
        /// Aceleração linear no eixo X (em m/s²).
        accel_x: f32,
        /// Aceleração linear no eixo Y (em m/s²).
        accel_y: f32,
        /// Aceleração linear no eixo Z (em m/s²).
        accel_z: f32,
        /// Velocidade angular no eixo X (em graus/s).
        gyro_x: f32,
        /// Velocidade angular no eixo Y (em graus/s).
        gyro_y: f32,
        /// Velocidade angular no eixo Z (em graus/s).
        gyro_z: f32,
    },
    /// Snapshot de velocidade instantânea gerada pela roda.
    Speed {
        /// Rotações por minuto da roda da bicicleta.
        rpm: f32,
        /// Velocidade linear convertida para km/h.
        speed_kmh: f32,
    },
    /// Coordenadas e telemetria espacial fornecidas pelo receptor GPS.
    Gps {
        /// Latitude em graus decimais (ex: -23.5505).
        lat: f64,
        /// Longitude em graus decimais (ex: -46.6333).
        lon: f64,
        /// Altitude elipsoidal em metros.
        altitude_m: f32,
        /// Velocidade do movimento estimada em km/h.
        speed_kmh: f32,
        /// Número de satélites rastreados e em uso para o fix atual.
        satellites: u8,
    },
}

/// Hierarquia de erros tipados que podem ocorrer durante a leitura de sensores.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum SensorError {
    /// O barramento I2C/SPI ou o pino GPIO retornou uma falha física de barramento.
    #[error("Falha física no barramento: {0}")]
    BusError(String),
    /// O sensor não pôde ser encontrado no endereço esperado ou não respondeu.
    #[error("Sensor offline ou não inicializado: {0}")]
    SensorOffline(String),
    /// O buffer de entrada serial ou o parser NMEA retornou dados corrompidos.
    #[error("Dados recebidos inválidos ou corrompidos: {0}")]
    InvalidData(String),
    /// O receptor GPS está ligado mas não possui constelação suficiente para fix de sinal.
    #[error("Receptor GPS sem fix de sinal válido")]
    GpsNoFix,
}

/// Contrato assíncrono universal que todos os drivers de sensores do sistema devem implementar.
#[allow(async_fn_in_trait)]
pub trait SensorReader: Send + Sync {
    /// Executa uma leitura assíncrona do sensor físico e retorna o snapshot correspondente.
    async fn read(&mut self) -> Result<SensorData, SensorError>;
}
