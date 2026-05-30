//! Driver do Receptor GPS baseado no protocolo serial NMEA 0183.

use crate::traits::{SensorData, SensorError, SensorReader};
use std::io::{BufRead, BufReader};
use std::time::Duration;

/// Driver de leitura de geolocalização e telemetria espacial via receptor serial GPS.
pub struct NmeaGpsDriver {
    port_path: String,
    baud_rate: u32,
}

impl NmeaGpsDriver {
    /// Inicializa uma nova interface associada a um path serial e baudrate.
    pub fn new(port_path: &str, baud_rate: u32) -> Self {
        Self {
            port_path: port_path.to_string(),
            baud_rate,
        }
    }

    /// Converte a representação de latitude/longitude NMEA (DDMM.MMMM) para graus decimais normais.
    fn parse_degrees(nmea_val: &str, direction: &str) -> Option<f64> {
        if nmea_val.is_empty() || direction.is_empty() {
            return None;
        }

        let dot_idx = nmea_val.find('.')?;
        let deg_len = dot_idx.saturating_sub(2);

        let degrees_str = &nmea_val[0..deg_len];
        let minutes_str = &nmea_val[deg_len..];

        let degrees = degrees_str.parse::<f64>().unwrap_or(0.0);
        let minutes = minutes_str.parse::<f64>().unwrap_or(0.0);

        let mut decimal = degrees + (minutes / 60.0);
        if direction == "S" || direction == "W" {
            decimal = -decimal;
        }

        Some(decimal)
    }
}

impl SensorReader for NmeaGpsDriver {
    async fn read(&mut self) -> Result<SensorData, SensorError> {
        let port = serialport::new(&self.port_path, self.baud_rate)
            .timeout(Duration::from_millis(500))
            .open()
            .map_err(|e| SensorError::SensorOffline(e.to_string()))?;

        let mut reader = BufReader::new(port);
        let mut line = String::new();

        // Parâmetros acumulados para preenchimento
        let mut lat = None;
        let mut lon = None;
        let mut speed_kmh = 0.0;
        let mut altitude_m = 0.0;
        let mut satellites = 0;
        let mut has_fix = false;

        // Tenta ler até 10 linhas da porta serial para extrair RMC e GGA simultaneamente
        for _ in 0..10 {
            line.clear();
            if reader.read_line(&mut line).is_err() {
                continue;
            }

            let fields: Vec<&str> = line.split(',').map(str::trim).collect();
            if fields.is_empty() {
                continue;
            }

            let sentence_type = fields[0];

            if sentence_type.contains("RMC") && fields.len() >= 9 {
                // $GPRMC,hhmmss.ss,A,llll.ll,a,yyyyy.yy,a,x.x,x.x,ddmmyy,,,a*hh
                // Status: A = Active (Fix válido), V = Void (Inválido)
                if fields[2] == "A" {
                    has_fix = true;
                    lat = Self::parse_degrees(fields[3], fields[4]);
                    lon = Self::parse_degrees(fields[5], fields[6]);

                    // Nós -> km/h (1 nó = 1.852 km/h)
                    let knots = fields[7].parse::<f32>().unwrap_or(0.0);
                    speed_kmh = knots * 1.852;
                } else {
                    return Err(SensorError::GpsNoFix);
                }
            } else if sentence_type.contains("GGA") && fields.len() >= 10 {
                // $GPGGA,hhmmss.ss,llll.ll,a,yyyyy.yy,a,x,xx,x.x,x.x,M,x.x,M,,*hh
                // Satélites em uso: campo 7. Altitude: campo 9
                satellites = fields[7].parse::<u8>().unwrap_or(0);
                altitude_m = fields[9].parse::<f32>().unwrap_or(0.0);
            }

            if has_fix {
                if let (Some(lat_val), Some(lon_val)) = (lat, lon) {
                    return Ok(SensorData::Gps {
                        lat: lat_val,
                        lon: lon_val,
                        altitude_m,
                        speed_kmh,
                        satellites,
                    });
                }
            }
        }

        Err(SensorError::GpsNoFix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_degrees_lat_lon() {
        // Lat: 2333.0300 S -> -23.5505 (aprox)
        let lat_dec = NmeaGpsDriver::parse_degrees("2333.0300", "S").unwrap();
        assert!((lat_dec - (-23.5505)).abs() < 1e-4);

        // Lon: 4637.9980 W -> -46.6333 (aprox)
        let lon_dec = NmeaGpsDriver::parse_degrees("4637.9980", "W").unwrap();
        assert!((lon_dec - (-46.6333)).abs() < 1e-4);
    }
}
