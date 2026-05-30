use serde::{Deserialize, Serialize};
use crate::error::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageCode(String);

impl LanguageCode {
    pub fn new(code: &str) -> Result<Self, ConfigError> {
        if code.len() >= 2 && code.is_ascii() {
            Ok(Self(code.to_string()))
        } else {
            Err(ConfigError::InvalidLanguageCode(code.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CountryCode {
    pub iso2: String,
    pub emergency_number: String,
    pub map_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SensorId(String);

impl SensorId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UsbModemPath(String);

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct OnboardingProgress {
    pub setup_complete: bool,
    pub language: Option<LanguageCode>,
    pub country: Option<CountryCode>,
    pub wifi_ssid: Option<String>,
    pub ota_checked: Option<bool>,
    pub map_downloaded: Option<bool>,
    pub sensors_paired: Option<Vec<SensorId>>,
}

impl OnboardingProgress {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum OtaChannel {
    #[default]
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "beta")]
    Beta,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct BikeConfig {
    pub onboarding: OnboardingProgress,
    pub ota_channel: OtaChannel,
    pub home_wifi_ssid: Option<String>,
    pub strava_token: Option<String>,
    pub firmware_version: Option<String>,
    pub wheel_circumference_mm: Option<u32>,
    pub rollback_marker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OtaRelease {
    pub version: semver::Version,
    pub checksum: String,
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

