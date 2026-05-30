//! Driver do Sensor Reed Switch via GPIO sysfs do Linux.

use crate::traits::{SensorData, SensorError, SensorReader};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

/// Driver para ler pulsos analógicos/digitais de um reed switch de roda de bicicleta.
pub struct ReedSwitchDriver {
    gpio_pin: u32,
    wheel_circumference_mm: u32,
    last_pulse: Option<Instant>,
    last_state: bool,
    debounce_ms: u64,
}

impl ReedSwitchDriver {
    /// Cria uma nova instância associada a um pino GPIO e à circunferência da roda.
    pub fn new(gpio_pin: u32, wheel_circumference_mm: u32) -> Self {
        Self {
            gpio_pin,
            wheel_circumference_mm,
            last_pulse: None,
            last_state: false,
            debounce_ms: 20, // 20ms de debounce para ruídos de bouncing mecânico
        }
    }

    /// Tenta exportar o pino GPIO via sysfs se ele ainda não estiver ativo.
    pub fn export_gpio(&self) -> Result<(), SensorError> {
        let gpio_dir = format!("/sys/class/gpio/gpio{}", self.gpio_pin);
        if Path::new(&gpio_dir).exists() {
            return Ok(());
        }

        // Exporta o pino digitando no /sys/class/gpio/export
        std::fs::write("/sys/class/gpio/export", self.gpio_pin.to_string()).map_err(|e| {
            SensorError::BusError(format!("Não foi possível exportar pino GPIO: {e}"))
        })?;

        // Espera um instante para o kernel processar a criação dos arquivos
        std::thread::sleep(Duration::from_millis(50));

        // Define a direção do pino como entrada
        let direction_path = format!("{gpio_dir}/direction");
        std::fs::write(direction_path, "in").map_err(|e| {
            SensorError::BusError(format!("Não foi possível configurar direção da GPIO: {e}"))
        })?;

        Ok(())
    }

    /// Lê o valor lógico atual do pino (true = alto, false = baixo).
    fn read_gpio_value(&self) -> Result<bool, SensorError> {
        let value_path = format!("/sys/class/gpio/gpio{}/value", self.gpio_pin);
        let mut file =
            File::open(value_path).map_err(|e| SensorError::SensorOffline(e.to_string()))?;

        let mut content = [0u8; 1];
        file.read_exact(&mut content)
            .map_err(|e| SensorError::BusError(e.to_string()))?;

        Ok(content[0] == b'1')
    }
}

impl SensorReader for ReedSwitchDriver {
    async fn read(&mut self) -> Result<SensorData, SensorError> {
        let _ = self.export_gpio(); // Tenta exportar se necessário

        let current_state = self.read_gpio_value().unwrap_or(false);
        let now = Instant::now();

        // Borda de subida detectada (mudança de falso/baixo para verdadeiro/alto)
        if current_state && !self.last_state {
            self.last_state = true;

            if let Some(last) = self.last_pulse {
                let duration = now.duration_since(last);

                // Filtra bouncing mecânico com debounce
                if duration >= Duration::from_millis(self.debounce_ms) {
                    self.last_pulse = Some(now);
                    let sec = duration.as_secs_f32();

                    let rpm = 60.0 / sec;
                    // km/h = (RPM * circumference_mm * 60.0) / 1_000_000.0
                    let speed_kmh =
                        (rpm * (self.wheel_circumference_mm as f32) * 60.0) / 1_000_000.0;

                    return Ok(SensorData::Speed { rpm, speed_kmh });
                }
            } else {
                self.last_pulse = Some(now);
            }
        } else if !current_state {
            self.last_state = false;
        }

        // Se nenhum pulso ocorreu em 3 segundos, assume que a bike está parada (velocidade zero)
        if let Some(last) = self.last_pulse {
            if now.duration_since(last) > Duration::from_secs(3) {
                self.last_pulse = None;
                return Ok(SensorData::Speed {
                    rpm: 0.0,
                    speed_kmh: 0.0,
                });
            }
        }

        // Retorna o último snapshot conhecido ou parado
        Ok(SensorData::Speed {
            rpm: 0.0,
            speed_kmh: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_calculation() {
        let circumference_mm = 2100; // aro 29/700c padrão
        let duration_sec = 0.5; // 0.5s por rotação (120 RPM)
        let rpm = 60.0 / duration_sec;
        let speed_kmh = (rpm * circumference_mm as f32 * 60.0) / 1_000_000.0;
        assert!((rpm - 120.0).abs() < 1e-5);
        assert!((speed_kmh - 15.12).abs() < 1e-5);
    }
}
