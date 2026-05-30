//! Sensor sintético de simulação para testes e ambientes de desenvolvimento no host.

use crate::traits::{SensorData, SensorError, SensorReader};

/// Cenários de simulação de telemetria suportados pelo sensor mock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockScenario {
    /// Simula pedalada estável normal a 20 km/h.
    NormalBiking,
    /// Simula evento de queda brusca (free-fall seguido de impacto).
    FallTrigger,
    /// Simula área sem sinal de satélite para o receptor GPS.
    NoGpsFix,
    /// Simula sensor de frequencia cardiaca BLE.
    HeartRateSensor,
    /// Simula sensor de cadencia BLE.
    CadenceSensor,
}

/// Gerador sintético de dados físicos de sensores.
pub struct MockSensor {
    scenario: MockScenario,
    read_count: u64,
    lat_current: f64,
    lon_current: f64,
}

impl MockSensor {
    /// Cria uma nova instância de simulação com base em um cenário específico.
    pub fn new(scenario: MockScenario) -> Self {
        Self {
            scenario,
            read_count: 0,
            lat_current: -23.5505,
            lon_current: -46.6333,
        }
    }

    /// Altera dinamicamente o cenário ativo de simulação.
    pub fn set_scenario(&mut self, scenario: MockScenario) {
        self.scenario = scenario;
    }
}

impl SensorReader for MockSensor {
    #[allow(clippy::too_many_lines)]
    async fn read(&mut self) -> Result<SensorData, SensorError> {
        self.read_count += 1;

        match self.scenario {
            MockScenario::HeartRateSensor => {
                let bpm = 140 + (self.read_count % 41) as u8;
                return Ok(SensorData::HeartRate {
                    bpm,
                    contact_detected: true,
                });
            }
            MockScenario::CadenceSensor => {
                let crank_revolutions = self.read_count as u16;
                let last_crank_event_time = ((self.read_count * 1024) % 65536) as u16;
                return Ok(SensorData::Cadence {
                    crank_revolutions,
                    last_crank_event_time,
                });
            }
            _ => {}
        }

        // Rotaciona as leituras de tipo de sensor (0 = IMU, 1 = Speed, 2 = GPS)
        let sensor_type = (self.read_count - 1) % 3;

        match sensor_type {
            0 => {
                // --- Simulação de IMU (m/s² e graus/s) ---
                match self.scenario {
                    MockScenario::NormalBiking | MockScenario::NoGpsFix => {
                        // Ruído estocástico de pedalada normal em torno de 1g de gravidade no eixo Z
                        Ok(SensorData::Imu {
                            accel_x: 0.1,
                            accel_y: 0.2,
                            accel_z: 9.8, // gravidade normal
                            gyro_x: 1.0,
                            gyro_y: 2.0,
                            gyro_z: 0.5,
                        })
                    }
                    MockScenario::FallTrigger => {
                        // Simula padrão de queda livre e impacto sequencial
                        if self.read_count < 15 {
                            // Fase 1: Free Fall (aceleração próxima a zero nos 3 eixos)
                            Ok(SensorData::Imu {
                                accel_x: 0.05,
                                accel_y: 0.05,
                                accel_z: 0.08,
                                gyro_x: 15.0,
                                gyro_y: 25.0,
                                gyro_z: 35.0,
                            })
                        } else if self.read_count < 18 {
                            // Fase 2: Impacto brusco (> 3g de aceleração em múltiplos eixos)
                            Ok(SensorData::Imu {
                                accel_x: 35.0, // impacto
                                accel_y: 12.0,
                                accel_z: 42.0,
                                gyro_x: 180.0,
                                gyro_y: 90.0,
                                gyro_z: 220.0,
                            })
                        } else {
                            // Fase 3: Parado após queda (sem movimento)
                            Ok(SensorData::Imu {
                                accel_x: 0.0,
                                accel_y: 0.0,
                                accel_z: 9.8,
                                gyro_x: 0.0,
                                gyro_y: 0.0,
                                gyro_z: 0.0,
                            })
                        }
                    }
                    _ => Err(SensorError::SensorOffline(
                        "Cenário inválido para IMU".into(),
                    )),
                }
            }
            1 => {
                // --- Simulação de Velocidade (RPM, km/h) ---
                match self.scenario {
                    MockScenario::NormalBiking | MockScenario::NoGpsFix => Ok(SensorData::Speed {
                        rpm: 160.0,
                        speed_kmh: 20.16,
                    }),
                    MockScenario::FallTrigger => {
                        if self.read_count < 18 {
                            Ok(SensorData::Speed {
                                rpm: 120.0,
                                speed_kmh: 15.12,
                            })
                        } else {
                            // Bike parada após o acidente
                            Ok(SensorData::Speed {
                                rpm: 0.0,
                                speed_kmh: 0.0,
                            })
                        }
                    }
                    _ => Err(SensorError::SensorOffline(
                        "Cenário inválido para Velocidade".into(),
                    )),
                }
            }
            _ => {
                // --- Simulação de GPS (lat, lon, altitude, velocidade, satélites) ---
                match self.scenario {
                    MockScenario::NormalBiking => {
                        // Incrementa latitude/longitude simulando deslocamento geográfico
                        self.lat_current += 0.0001;
                        self.lon_current += 0.0001;
                        Ok(SensorData::Gps {
                            lat: self.lat_current,
                            lon: self.lon_current,
                            altitude_m: 760.5,
                            speed_kmh: 20.0,
                            satellites: 9,
                        })
                    }
                    MockScenario::FallTrigger => Ok(SensorData::Gps {
                        lat: self.lat_current,
                        lon: self.lon_current,
                        altitude_m: 760.5,
                        speed_kmh: 0.0,
                        satellites: 8,
                    }),
                    MockScenario::NoGpsFix => Err(SensorError::GpsNoFix),
                    _ => Err(SensorError::SensorOffline(
                        "Cenário inválido para GPS".into(),
                    )),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_flow() {
        let mut mock = MockSensor::new(MockScenario::NormalBiking);
        let d1 = mock.read().await.unwrap();
        assert!(matches!(d1, SensorData::Imu { .. }));

        let d2 = mock.read().await.unwrap();
        assert!(matches!(d2, SensorData::Speed { .. }));

        let d3 = mock.read().await.unwrap();
        assert!(matches!(d3, SensorData::Gps { .. }));
    }

    #[tokio::test]
    async fn test_mock_ble_heart_rate_scenario() {
        let mut mock = MockSensor::new(MockScenario::HeartRateSensor);
        let d = mock.read().await.unwrap();
        if let SensorData::HeartRate {
            bpm,
            contact_detected,
        } = d
        {
            assert!(bpm >= 140 && bpm <= 180);
            assert!(contact_detected);
        } else {
            panic!("Expected HeartRate");
        }
    }

    #[tokio::test]
    async fn test_mock_ble_cadence_scenario() {
        let mut mock = MockSensor::new(MockScenario::CadenceSensor);
        let d1 = mock.read().await.unwrap();
        let d2 = mock.read().await.unwrap();
        if let (
            SensorData::Cadence {
                crank_revolutions: r1,
                ..
            },
            SensorData::Cadence {
                crank_revolutions: r2,
                ..
            },
        ) = (d1, d2)
        {
            assert_eq!(r1, 1);
            assert_eq!(r2, 2);
        } else {
            panic!("Expected Cadence");
        }
    }
}
