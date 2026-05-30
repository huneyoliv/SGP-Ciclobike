//! Codificador de trajetos no formato GPX 1.1 para upload de atividades.

use crate::snapshot::TrackPoint;
use std::fmt::Write;
use std::path::Path;

/// Codificador de TrackPoints para formato XML GPX 1.1.
pub struct GpxEncoder;

impl GpxEncoder {
    /// Converte uma lista de TrackPoints em uma String XML contendo um GPX 1.1 valido.
    pub fn encode(points: &[TrackPoint], activity_name: &str) -> String {
        let escaped_name = escape_xml(activity_name);
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<gpx version=\"1.1\" creator=\"SGP-Ciclobike\"\n");
        xml.push_str("     xmlns=\"http://www.topografix.com/GPX/1/1\"\n");
        xml.push_str(
            "     xmlns:gpxtpx=\"http://www.garmin.com/xmlschemas/TrackPointExtension/v1\">\n",
        );
        xml.push_str("  <trk>\n");
        let _ = writeln!(xml, "    <name>{escaped_name}</name>");
        xml.push_str("    <trkseg>\n");

        for p in points {
            if let (Some(lat), Some(lon)) = (p.lat, p.lon) {
                let _ = writeln!(xml, "      <trkpt lat=\"{lat:.7}\" lon=\"{lon:.7}\">");
                if let Some(ele) = p.altitude_m {
                    let _ = writeln!(xml, "        <ele>{ele:.1}</ele>");
                }
                let time_str = ms_to_iso8601(p.timestamp_ms);
                let _ = writeln!(xml, "        <time>{time_str}</time>");

                let has_extensions = p.cadence_rpm > 0.0 || p.heart_rate.unwrap_or(0) > 0;
                if has_extensions {
                    xml.push_str("        <extensions>\n");
                    xml.push_str("          <gpxtpx:TrackPointExtension>\n");
                    if let Some(hr) = p.heart_rate {
                        if hr > 0 {
                            let _ = writeln!(xml, "            <gpxtpx:hr>{hr}</gpxtpx:hr>");
                        }
                    }
                    if p.cadence_rpm > 0.0 {
                        let cad = p.cadence_rpm as u32;
                        let _ = writeln!(xml, "            <gpxtpx:cad>{cad}</gpxtpx:cad>");
                    }
                    xml.push_str("          </gpxtpx:TrackPointExtension>\n");
                    xml.push_str("        </extensions>\n");
                }
                xml.push_str("      </trkpt>\n");
            }
        }

        xml.push_str("    </trkseg>\n");
        xml.push_str("  </trk>\n");
        xml.push_str("</gpx>\n");
        xml
    }

    /// Grava a lista de TrackPoints codificada como GPX em um arquivo fisico.
    pub fn write_to_file(points: &[TrackPoint], path: &Path, name: &str) -> Result<(), String> {
        let content = Self::encode(points, name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, content).map_err(|e| e.to_string())
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn ms_to_iso8601(timestamp_ms: i64) -> String {
    const SECS_PER_MIN: i64 = 60;
    const SECS_PER_HOUR: i64 = 3600;
    const SECS_PER_DAY: i64 = 86400;

    let seconds = timestamp_ms / 1000;
    let days = seconds / SECS_PER_DAY;
    let rem_seconds = seconds % SECS_PER_DAY;

    let hour = rem_seconds / SECS_PER_HOUR;
    let rem_hour = rem_seconds % SECS_PER_HOUR;
    let minute = rem_hour / SECS_PER_MIN;
    let second = rem_hour % SECS_PER_MIN;

    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_097) / 365;
    let mut y = i64::from(yoe) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };

    if m <= 2 {
        y += 1;
    }
    let year = y;
    let month = m;
    let day = d;

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_gpx_encode_empty_returns_valid_xml() {
        let xml = GpxEncoder::encode(&[], "Pedalada SGP");
        assert!(xml.contains("<name>Pedalada SGP</name>"));
        assert!(xml.contains("</trkseg>"));
    }

    #[test]
    fn test_gpx_encode_single_point() {
        let p = TrackPoint {
            session_id: Uuid::new_v4(),
            timestamp_ms: 1718442000000, // 2024-06-15T09:00:00Z
            lat: Some(-23.5505),
            lon: Some(-46.6333),
            altitude_m: Some(760.5),
            speed_kmh: 20.0,
            cadence_rpm: 85.0,
            accel_magnitude: 1.0,
            heart_rate: Some(145),
        };
        let xml = GpxEncoder::encode(&[p], "Teste & Treino");
        assert!(xml.contains("lat=\"-23.5505000\""));
        assert!(xml.contains("lon=\"-46.6333000\""));
        assert!(xml.contains("<ele>760.5</ele>"));
        assert!(xml.contains("<time>2024-06-15T09:00:00Z</time>"));
        assert!(xml.contains("<gpxtpx:hr>145</gpxtpx:hr>"));
        assert!(xml.contains("<gpxtpx:cad>85</gpxtpx:cad>"));
        assert!(xml.contains("Teste &amp; Treino"));
    }

    #[test]
    fn test_gpx_encode_skips_points_without_coordinates() {
        let p = TrackPoint {
            session_id: Uuid::new_v4(),
            timestamp_ms: 1718442000000,
            lat: None,
            lon: None,
            altitude_m: Some(760.5),
            speed_kmh: 20.0,
            cadence_rpm: 85.0,
            accel_magnitude: 1.0,
            heart_rate: None,
        };
        let xml = GpxEncoder::encode(&[p], "Omitido");
        assert!(!xml.contains("<trkpt"));
    }
}
