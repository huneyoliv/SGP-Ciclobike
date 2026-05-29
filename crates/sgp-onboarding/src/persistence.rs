//! Lógica de persistência e recuperação do progresso de onboarding.

use std::path::{Path, PathBuf};
use sgp_core::{BikeConfig, ConfigError, OnboardingProgress};
use crate::state_machine::OnboardingState;

/// Caminho oficial do arquivo de configuração persistente.
pub const CONFIG_PATH: &str = "/etc/bike_config.toml";

/// Protetor (Guard) RAII para manipulação segura e atômica da configuração global.
pub struct ConfigGuard {
    progress: OnboardingProgress,
    path: PathBuf,
    dirty: bool,
}

impl ConfigGuard {
    /// Carrega a configuração do disco. Retorna estado padrão se o arquivo não existir ou for corrompido.
    pub fn load_or_default(path: &Path) -> Self {
        match Self::try_load(path) {
            Ok(guard) => guard,
            Err(_) => Self {
                progress: OnboardingProgress::default(),
                path: path.to_path_buf(),
                dirty: false,
            },
        }
    }

    fn try_load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: BikeConfig = toml::from_str(&content)?;
        Ok(Self {
            progress: config.onboarding,
            path: path.to_path_buf(),
            dirty: false,
        })
    }

    /// Atualiza o progresso usando uma função mutadora e persiste atonicamente em disco.
    pub fn save_step(&mut self, mutate: impl FnOnce(&mut OnboardingProgress)) -> Result<(), ConfigError> {
        mutate(&mut self.progress);
        self.dirty = true;
        self.flush_atomic()
    }

    /// Retorna uma referência imutável ao progresso atual.
    pub fn progress(&self) -> &OnboardingProgress {
        &self.progress
    }

    fn flush_atomic(&mut self) -> Result<(), ConfigError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp_path = self.path.with_extension("toml.tmp");
        let config = BikeConfig {
            onboarding: self.progress.clone(),
            ..BikeConfig::default()
        };
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&tmp_path, content)?;
        std::fs::rename(&tmp_path, &self.path)?;
        self.dirty = false;
        Ok(())
    }
}

impl Drop for ConfigGuard {
    fn drop(&mut self) {
        if self.dirty {
            let _ = self.flush_atomic();
        }
    }
}

/// Determina o estado inicial do onboarding a partir do progresso acumulado em disco.
pub fn resume_state(progress: &OnboardingProgress) -> OnboardingState {
    if progress.setup_complete {
        return OnboardingState::Complete;
    }
    let Some(language) = progress.language.clone() else {
        return OnboardingState::SelectLanguage;
    };
    let Some(country) = progress.country.clone() else {
        return OnboardingState::SelectCountry { language };
    };
    let Some(wifi_ssid) = progress.wifi_ssid.clone() else {
        return OnboardingState::ConnectWifi { language, country };
    };
    if progress.ota_checked.is_none() {
        return OnboardingState::CheckOtaUpdate {
            language,
            country,
            wifi_ssid,
        };
    }
    OnboardingState::Complete
}

#[cfg(test)]
mod tests {
    use super::*;
    use sgp_core::{CountryCode, LanguageCode};

    #[test]
    fn test_resume_state_logic() {
        let mut progress = OnboardingProgress::default();
        assert_eq!(resume_state(&progress), OnboardingState::SelectLanguage);

        let lang = LanguageCode::new("pt-BR").unwrap();
        progress.language = Some(lang.clone());
        assert_eq!(
            resume_state(&progress),
            OnboardingState::SelectCountry { language: lang.clone() }
        );

        let country = CountryCode {
            iso2: "BR".to_string(),
            emergency_number: "192".to_string(),
            map_url: "url".to_string(),
        };
        progress.country = Some(country.clone());
        assert_eq!(
            resume_state(&progress),
            OnboardingState::ConnectWifi {
                language: lang.clone(),
                country: country.clone()
            }
        );

        progress.wifi_ssid = Some("Home_SSID".to_string());
        assert_eq!(
            resume_state(&progress),
            OnboardingState::CheckOtaUpdate {
                language: lang.clone(),
                country: country.clone(),
                wifi_ssid: "Home_SSID".to_string()
            }
        );

        progress.ota_checked = Some(true);
        assert_eq!(resume_state(&progress), OnboardingState::Complete);
    }

    #[test]
    fn test_atomic_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bike_config.toml");
        
        let mut guard = ConfigGuard::load_or_default(&path);
        let lang = LanguageCode::new("pt-BR").unwrap();
        
        guard.save_step(|p| {
            p.language = Some(lang);
        }).unwrap();

        assert!(path.exists());
        
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("language = \"pt-BR\""));
    }
}
