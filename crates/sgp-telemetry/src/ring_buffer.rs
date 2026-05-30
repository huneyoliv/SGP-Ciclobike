//! Buffer circular persistente em disco baseado em NDJSON e rotação atômica.

use crate::snapshot::TrackPoint;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Buffer circular gravado em disco de forma seqüencial usando JSON por linha (NDJSON).
pub struct DiskRingBuffer {
    file_path: PathBuf,
    max_size_bytes: usize,
}

impl DiskRingBuffer {
    /// Inicializa a interface de buffer circular associada a um arquivo em disco.
    pub fn new(file_path: &Path, max_size_bytes: usize) -> Self {
        Self {
            file_path: file_path.to_path_buf(),
            max_size_bytes,
        }
    }

    /// Escreve um novo trackpoint de telemetria no final do arquivo de log.
    ///
    /// Se o tamanho do arquivo exceder o limite, realiza a rotação de log atômica.
    pub fn push(&self, point: &TrackPoint) -> Result<(), String> {
        // Garante que o diretório pai existe
        if let Some(parent) = self.file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Verifica o tamanho atual do arquivo para rotação
        if self.file_path.exists() {
            let meta = std::fs::metadata(&self.file_path).map_err(|e| e.to_string())?;
            if meta.len() as usize >= self.max_size_bytes {
                self.rotate_log()?;
            }
        }

        let serialized = serde_json::to_string(point).map_err(|e| e.to_string())?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|e| e.to_string())?;

        writeln!(file, "{serialized}").map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Roda o log atômico para evitar estouro de armazenamento.
    fn rotate_log(&self) -> Result<(), String> {
        let mut old_path = self.file_path.clone();
        old_path.set_extension("json.old");

        // Substituição atômica via rename do Linux
        std::fs::rename(&self.file_path, &old_path).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Retorna até `limit` trackpoints pendentes para sincronização e os remove do log.
    #[allow(clippy::manual_flatten)]
    pub fn drain_batch(&self, limit: usize) -> Result<Vec<TrackPoint>, String> {
        let mut old_path = self.file_path.clone();
        old_path.set_extension("json.old");

        let mut batch = Vec::new();

        // 1. Processa primeiro o arquivo .old (se existir) para manter a ordem cronológica
        if old_path.exists() {
            let file = File::open(&old_path).map_err(|e| e.to_string())?;
            let reader = BufReader::new(file);
            let mut remaining_lines = Vec::new();

            for (idx, line_res) in reader.lines().enumerate() {
                if let Ok(line) = line_res {
                    if idx < limit && batch.len() < limit {
                        if let Ok(point) = serde_json::from_str::<TrackPoint>(&line) {
                            batch.push(point);
                        }
                    } else {
                        remaining_lines.push(line);
                    }
                }
            }

            if remaining_lines.is_empty() {
                // Todas as linhas do .old foram enviadas: remove o arquivo .old
                let _ = std::fs::remove_file(&old_path);
            } else {
                // Reescreve apenas as linhas restantes que não couberam no lote
                let mut file = File::create(&old_path).map_err(|e| e.to_string())?;
                for line in remaining_lines {
                    writeln!(file, "{line}").map_err(|e| e.to_string())?;
                }
            }
        }

        // 2. Se o lote ainda não atingiu o limite, lê do arquivo ativo principal
        if batch.len() < limit && self.file_path.exists() {
            let file = File::open(&self.file_path).map_err(|e| e.to_string())?;
            let reader = BufReader::new(file);
            let mut remaining_lines = Vec::new();

            for line_res in reader.lines() {
                if let Ok(line) = line_res {
                    if batch.len() < limit {
                        if let Ok(point) = serde_json::from_str::<TrackPoint>(&line) {
                            batch.push(point);
                        }
                    } else {
                        remaining_lines.push(line);
                    }
                }
            }

            // Reescreve o arquivo ativo principal apenas com as linhas restantes
            let mut file = File::create(&self.file_path).map_err(|e| e.to_string())?;
            for line in remaining_lines {
                writeln!(file, "{line}").map_err(|e| e.to_string())?;
            }
        }

        Ok(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_push_and_drain_batch() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("telemetry_test.json");
        let _ = std::fs::remove_file(&path);

        let buffer = DiskRingBuffer::new(&path, 1024); // 1KB limit

        let id = Uuid::new_v4();
        let p = TrackPoint {
            session_id: id,
            timestamp_ms: 12345,
            lat: Some(1.0),
            lon: Some(2.0),
            altitude_m: Some(10.0),
            speed_kmh: 20.0,
            cadence_rpm: 80.0,
            accel_magnitude: 9.8,
            heart_rate: None,
        };

        buffer.push(&p).unwrap();

        let batch = buffer.drain_batch(10).unwrap();
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].session_id, id);

        // Limpa arquivo
        let _ = std::fs::remove_file(&path);
    }
}
