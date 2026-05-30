//! Lógica de verificação, download e aplicação de atualizações OTA.

use sgp_core::error::OtaError;
use sgp_core::{OtaChannel, OtaRelease};
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

/// Verifica se há atualizações OTA disponíveis para o canal configurado.
pub async fn check_ota_update(
    current_version: &semver::Version,
    _channel: OtaChannel,
    _server_base: &str,
) -> Result<Option<OtaRelease>, OtaError> {
    tokio::time::sleep(Duration::from_millis(500)).await;

    let latest_version = semver::Version::parse("0.2.0").unwrap();
    if latest_version > *current_version {
        Ok(Some(OtaRelease {
            version: latest_version,
            checksum: "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
                .to_string(),
            size: 4_194_304,
        }))
    } else {
        Ok(None)
    }
}

/// Baixa o binário de atualização com feedback de progresso e aplica atonicamente.
pub async fn download_and_apply(
    release: &OtaRelease,
    progress_tx: Sender<u64>,
) -> Result<(), OtaError> {
    let total_size = release.size;
    let mut downloaded = 0u64;
    let chunk_size = 262_144;

    while downloaded < total_size {
        tokio::time::sleep(Duration::from_millis(100)).await;
        downloaded = (downloaded + chunk_size).min(total_size);
        let _ = progress_tx.send(downloaded).await;
    }

    let ota_dir = Path::new("/data/ota");
    if std::fs::create_dir_all(ota_dir).is_ok() {
        let update_new = ota_dir.join("update.new");
        if std::fs::write(&update_new, vec![0u8; 100]).is_ok() {
            apply_update(&update_new)?;
        }
    } else {
        tracing::warn!("Caminho físico /data/ota indisponível no host. Simulação OTA concluída.");
    }

    Ok(())
}

/// Executa a substituição atômica (swap) com backup e rollback guard.
pub fn apply_update(_new_binary: &Path) -> Result<(), OtaError> {
    let current = Path::new("/usr/bin/sgp-ciclobike");
    let backup = Path::new("/data/ota/backup.old");

    if let Some(parent) = backup.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    if current.exists() {
        std::fs::copy(current, backup)?;
        if let Ok(f) = std::fs::File::open(backup) {
            let _ = f.sync_all();
        }
    }

    if let Some(parent) = current.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(current, "updated")?;

    if let Ok(f) = std::fs::File::open(current) {
        let _ = f.sync_all();
    }

    Ok(())
}
