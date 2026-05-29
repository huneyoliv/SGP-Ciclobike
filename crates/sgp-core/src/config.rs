//! Estruturas de configuração e tipos do domínio do ciclocomputador.

use serde::{Deserialize, Serialize};
use crate::error::ConfigError;

/// Código de idioma no padrão IETF BCP 47.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageCode(String);

impl LanguageCode {
    /// Cria e valida um novo código de idioma.
    pub fn new(code: &str) -> Result<Self, ConfigError> {
        if code.len() >= 2 && code.is_ascii() {
            Ok(Self(code.to_string()))
        } else {
            Err(ConfigError::InvalidLanguageCode(code.to_string()))
        }
    }

    /// Retorna a representação textual do código.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Identificação de país com metadados para chamadas de emergência e mapas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CountryCode {
    /// Código ISO 3166-1 alpha-2.
    pub iso2: String,
    /// Telefone dos serviços locais de emergência (ex: "192" no Brasil).
    pub emergency_number: String,
    /// URL para baixar o mapa off-line da região.
    pub map_url: String,
}

/// Identificador único de um sensor BLE pareado.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SensorId(String);

impl SensorId {
    /// Cria uma nova identificação de sensor.
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    /// Retorna o identificador textual do sensor.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Caminho físico associado à porta serial do modem USB.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UsbModemPath(String);

/// Registro do progresso do usuário no wizard de setup inicial.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct OnboardingProgress {
    /// Flag que indica se todas as etapas obrigatórias foram concluídas.
    pub setup_complete: bool,
    /// Idioma selecionado na etapa 1.
    pub language: Option<LanguageCode>,
    /// País selecionado na etapa 2.
    pub country: Option<CountryCode>,
    /// SSID da rede Wi-Fi conectada na etapa 3.
    pub wifi_ssid: Option<String>,
    /// Confirmação de verificação de atualização OTA da etapa 4.
    pub ota_checked: Option<bool>,
    /// Confirmação de download do mapa regional (iteração futura).
    pub map_downloaded: Option<bool>,
    /// Lista de sensores periféricos pareados (iteração futura).
    pub sensors_paired: Option<Vec<SensorId>>,
}

impl OnboardingProgress {
    /// Valida e finaliza o progresso do wizard se as etapas obrigatórias forem satisfeitas.
    pub fn try_finalize(&mut self) -> Result<(), ConfigError> {
        if self.language.is_none() {
            return Err(ConfigError::IncompleteStep("language"));
        }
        if self.country.is_none() {
            return Err(ConfigError::IncompleteStep("country"));
        }
        if self.wifi_ssid.is_none() {
            return Err(ConfigError::IncompleteStep("wifi"));
        }
        if self.ota_checked.is_none() {
            return Err(ConfigError::IncompleteStep("ota"));
        }
        self.setup_complete = true;
        Ok(())
    }
}

/// Canais de distribuição de atualizações OTA.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum OtaChannel {
    /// Canal estável.
    #[default]
    #[serde(rename = "release")]
    Release,
    /// Canal de testes beta.
    #[serde(rename = "beta")]
    Beta,
}

/// Configuração global persistente do dispositivo.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct BikeConfig {
    /// Progresso atual do assistente de setup.
    pub onboarding: OnboardingProgress,
    /// Canal OTA ativo para busca de atualizações.
    pub ota_channel: OtaChannel,
    /// SSID da rede Wi-Fi doméstica.
    pub home_wifi_ssid: Option<String>,
    /// Token de autenticação OAuth para sincronização com o Strava.
    pub strava_token: Option<String>,
    /// Versão atual instalada do firmware do ciclocomputador.
    pub firmware_version: Option<String>,
    /// Marcador utilizado para identificar e testar novos builds OTA no boot.
    pub rollback_marker: Option<String>,
}

/// Detalhes de um release de atualização OTA disponível.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OtaRelease {
    /// Versão semântica do release.
    pub version: semver::Version,
    /// Hash SHA256 do binário para validação de integridade.
    pub checksum: String,
    /// Tamanho do arquivo binário em bytes.
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::OtaError;

    #[test]
    fn test_partial_toml_serialization() {
        let mut progress = OnboardingProgress::default();
        progress.language = Some(LanguageCode::new("pt-BR").unwrap());
        
        let config = BikeConfig {
            onboarding: progress,
            ..Default::default()
        };

        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("language = \"pt-BR\""));
        assert!(serialized.contains("setup_complete = false"));
        
        let deserialized: BikeConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.onboarding.language.unwrap().as_str(), "pt-BR");
        assert!(deserialized.onboarding.country.is_none());
    }

    #[test]
    fn test_ignore_extra_toml_fields() {
        let toml_data = r#"
            [onboarding]
            setup_complete = false
            language = "en-US"
            unknown_field = "ignored"

            [extra_section]
            foo = "bar"
        "#;

        let config: BikeConfig = toml::from_str(toml_data).unwrap();
        assert_eq!(config.onboarding.language.unwrap().as_str(), "en-US");
    }

    #[test]
    fn test_try_finalize_validation() {
        let mut progress = OnboardingProgress::default();
        assert!(progress.try_finalize().is_err());

        progress.language = Some(LanguageCode::new("pt-BR").unwrap());
        assert!(progress.try_finalize().is_err());

        progress.country = Some(CountryCode {
            iso2: "BR".to_string(),
            emergency_number: "192".to_string(),
            map_url: "https://example.com/map.map".to_string(),
        });
        assert!(progress.try_finalize().is_err());

        progress.wifi_ssid = Some("Home_Network".to_string());
        assert!(progress.try_finalize().is_err());

        progress.ota_checked = Some(true);
        assert!(progress.try_finalize().is_ok());
        assert!(progress.setup_complete);
    }

    #[test]
    fn test_ota_error_transiency() {
        assert!(OtaError::CheckTimeout.is_transient());
        assert!(OtaError::Network("conn down".to_string()).is_transient());
        assert!(!OtaError::AlreadyUpToDate.is_transient());
        assert!(!OtaError::ChecksumMismatch {
            expected: "a".to_string(),
            got: "b".to_string()
        }.is_transient());
    }
}

