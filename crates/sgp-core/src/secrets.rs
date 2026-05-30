//! Gerenciamento de credenciais compile-time e tokens runtime para o Strava.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Client ID do aplicativo Strava injetado no build.
pub const STRAVA_CLIENT_ID: u64 = match option_env!("STRAVA_CLIENT_ID") {
    Some(val) => match const_str_to_u64(val) {
        Some(v) => v,
        None => 0,
    },
    None => 0,
};

/// Client Secret do aplicativo Strava injetado no build.
pub const STRAVA_CLIENT_SECRET: &str = match option_env!("STRAVA_CLIENT_SECRET") {
    Some(val) => val,
    None => "",
};

const fn const_str_to_u64(s: &str) -> Option<u64> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let mut val = 0u64;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b < b'0' || b > b'9' {
            return None;
        }
        val = val * 10 + (b - b'0') as u64;
        i += 1;
    }
    Some(val)
}

/// Verifica se o aplicativo Strava possui credenciais basicas configuradas no build.
pub fn strava_app_configured() -> bool {
    STRAVA_CLIENT_ID != 0 && !STRAVA_CLIENT_SECRET.is_empty()
}

/// Tokens de acesso do usuario obtidos via OAuth2.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct StravaTokens {
    /// Token de acesso temporario.
    pub access_token: String,
    /// Token de atualizacao persistente.
    pub refresh_token: String,
    /// Unix timestamp de expiracao do access_token.
    pub expires_at: i64,
}

impl StravaTokens {
    /// Caminho principal de persistencia no hardware.
    pub const PATH: &'static str = "/etc/sgp-ciclobike/tokens.toml";
    /// Caminho alternativo local para ambiente de desenvolvimento.
    pub const ALT_PATH: &'static str = "tokens.toml";

    /// Carrega as credenciais a partir do disco.
    pub fn load() -> Option<Self> {
        let content = std::fs::read_to_string(Self::PATH)
            .or_else(|_| std::fs::read_to_string(Self::ALT_PATH))
            .ok()?;
        toml::from_str(&content).ok()
    }

    /// Grava as credenciais em disco no local correto ou no alternativo.
    pub fn save(&self) -> Result<(), String> {
        let content = toml::to_string(self).map_err(|e| e.to_string())?;

        let path = std::path::Path::new(Self::PATH);
        if let Some(parent) = path.parent() {
            if std::fs::create_dir_all(parent).is_ok() {
                if std::fs::write(path, &content).is_ok() {
                    return Ok(());
                }
            }
        }

        std::fs::write(Self::ALT_PATH, &content).map_err(|e| e.to_string())
    }

    /// Retorna verdadeiro se o access_token expirar em menos de 5 minutos.
    pub fn needs_refresh(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.expires_at - now < 300
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_str_to_u64() {
        assert_eq!(const_str_to_u64("12345"), Some(12345));
        assert_eq!(const_str_to_u64("0"), Some(0));
        assert_eq!(const_str_to_u64(""), None);
        assert_eq!(const_str_to_u64("abc"), None);
    }

    #[test]
    fn test_strava_tokens_needs_refresh() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let tokens_expiring = StravaTokens {
            access_token: "abc".into(),
            refresh_token: "def".into(),
            expires_at: now + 100, // expira em 100 segundos (menos de 5 minutos)
        };
        assert!(tokens_expiring.needs_refresh());

        let tokens_valid = StravaTokens {
            access_token: "abc".into(),
            refresh_token: "def".into(),
            expires_at: now + 7200, // expira em 2 horas
        };
        assert!(!tokens_valid.needs_refresh());
    }

    #[test]
    fn test_strava_tokens_roundtrip_toml() {
        let tokens = StravaTokens {
            access_token: "token_123".into(),
            refresh_token: "refresh_456".into(),
            expires_at: 1234567890,
        };
        let _ = tokens.save();
        let loaded = StravaTokens::load().unwrap();
        assert_eq!(tokens, loaded);
        let _ = std::fs::remove_file(StravaTokens::ALT_PATH);
    }
}
