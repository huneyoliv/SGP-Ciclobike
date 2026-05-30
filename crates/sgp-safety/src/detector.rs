//! Algoritmo inercial de detecção de quedas com base em aceleração tridimensional.

use std::time::{Duration, Instant};

/// Amostra inercial de aceleração tridimensional vinda da IMU.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImuSample {
    /// Timestamp de coleta da amostra física.
    pub timestamp: Instant,
    /// Aceleração linear no eixo X (em m/s²).
    pub accel_x: f32,
    /// Aceleração linear no eixo Y (em m/s²).
    pub accel_y: f32,
    /// Aceleração linear no eixo Z (em m/s²).
    pub accel_z: f32,
}

/// Evento gerado quando uma queda inercial é confirmada.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FallEvent {
    /// Timestamp exato do momento do impacto confirmado.
    pub timestamp: Instant,
    /// Magnitude inercial máxima registrada durante o impacto.
    pub impact_magnitude: f32,
}

/// Estados internos do processamento de queda inercial.
#[derive(Debug, Clone, Copy, PartialEq)]
enum FallState {
    /// Bike em movimento normal (gravidade próxima a 1.0g).
    Idle,
    /// Fase de queda livre (bike ou ciclista caindo).
    FreeFall {
        /// Timestamp de início da queda livre detectada.
        started_at: Instant,
    },
    /// Fase de espera de impacto iminente logo após a queda livre.
    WaitingImpact {
        /// Timestamp do momento em que a queda livre terminou.
        free_fall_ended_at: Instant,
    },
}

/// Processador inercial com máquina de estados integrada para detecção robusta de quedas.
pub struct FallDetector {
    state: FallState,
    free_fall_threshold_mps2: f32,
    free_fall_duration_min: Duration,
    impact_threshold_mps2: f32,
    impact_window_max: Duration,
}

impl Default for FallDetector {
    fn default() -> Self {
        Self {
            state: FallState::Idle,
            // Queda livre: magnitude < 0.3g (2.94 m/s²)
            free_fall_threshold_mps2: 2.94,
            free_fall_duration_min: Duration::from_millis(80),
            // Impacto: magnitude > 3.0g (29.4 m/s²)
            impact_threshold_mps2: 29.4,
            impact_window_max: Duration::from_millis(500),
        }
    }
}

impl FallDetector {
    /// Processa uma nova amostra de aceleração tridimensional.
    ///
    /// Retorna `Some(FallEvent)` apenas quando o padrão de queda + impacto é verificado.
    pub fn feed(&mut self, sample: ImuSample) -> Option<FallEvent> {
        let now = sample.timestamp;

        // Calcula a magnitude vetorial |a| = √(ax² + ay² + az²)
        let magnitude =
            (sample.accel_x.powi(2) + sample.accel_y.powi(2) + sample.accel_z.powi(2)).sqrt();

        match self.state {
            FallState::Idle => {
                if magnitude < self.free_fall_threshold_mps2 {
                    self.state = FallState::FreeFall { started_at: now };
                }
            }
            FallState::FreeFall { started_at } => {
                if magnitude >= self.free_fall_threshold_mps2 {
                    // Queda livre terminou
                    let duration = now.duration_since(started_at);
                    if duration >= self.free_fall_duration_min {
                        // Queda livre longa o suficiente: espera o impacto subsequente
                        self.state = FallState::WaitingImpact {
                            free_fall_ended_at: now,
                        };
                    } else {
                        // Bouncing mecânico ou falso positivo curto: aborta
                        self.state = FallState::Idle;
                    }
                }
            }
            FallState::WaitingImpact { free_fall_ended_at } => {
                if now.duration_since(free_fall_ended_at) > self.impact_window_max {
                    // Sem impacto dentro da janela temporal de 500ms: aborta
                    self.state = FallState::Idle;
                } else if magnitude > self.impact_threshold_mps2 {
                    // Impacto severo confirmado pós queda-livre! Dispara o alarme!
                    self.state = FallState::Idle; // Reseta o detector
                    return Some(FallEvent {
                        timestamp: now,
                        impact_magnitude: magnitude,
                    });
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ideal_fall_pattern() {
        let mut detector = FallDetector::default();
        let start = Instant::now();

        // Amostra 1: Normal (1g no Z)
        let f1 = detector.feed(ImuSample {
            timestamp: start,
            accel_x: 0.0,
            accel_y: 0.0,
            accel_z: 9.8,
        });
        assert!(f1.is_none());

        // Amostra 2: Início de Queda Livre (0.1g)
        let f2 = detector.feed(ImuSample {
            timestamp: start + Duration::from_millis(10),
            accel_x: 0.2,
            accel_y: 0.2,
            accel_z: 0.5,
        });
        assert!(f2.is_none());

        // Amostra 3: Fim de Queda Livre após 100ms
        let f3 = detector.feed(ImuSample {
            timestamp: start + Duration::from_millis(110),
            accel_x: 0.2,
            accel_y: 0.2,
            accel_z: 3.5, // Subiu acima do threshold de queda-livre
        });
        assert!(f3.is_none());

        // Amostra 4: Impacto severo 50ms depois (3.5g = 34.3 m/s²)
        let f4 = detector.feed(ImuSample {
            timestamp: start + Duration::from_millis(160),
            accel_x: 32.0,
            accel_y: 12.0,
            accel_z: 8.0,
        });
        assert!(f4.is_some());
        let event = f4.unwrap();
        assert!(event.impact_magnitude > 29.4);
    }
}
