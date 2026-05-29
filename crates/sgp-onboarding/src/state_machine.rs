//! Máquina de estados tipada para o wizard de onboarding.

use sgp_core::{CountryCode, LanguageCode};

/// Estados que compõem o assistente de onboarding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnboardingState {
    /// Etapa 1: Seleção de idioma.
    SelectLanguage,
    /// Etapa 2: Seleção do país de operação.
    SelectCountry {
        /// Idioma selecionado na etapa anterior.
        language: LanguageCode,
    },
    /// Etapa 3: Conexão com a rede Wi-Fi doméstica.
    ConnectWifi {
        /// Idioma selecionado.
        language: LanguageCode,
        /// País selecionado.
        country: CountryCode,
    },
    /// Etapa 4: Verificação de atualizações de firmware via OTA.
    CheckOtaUpdate {
        /// Idioma selecionado.
        language: LanguageCode,
        /// País selecionado.
        country: CountryCode,
        /// SSID da rede Wi-Fi conectada.
        wifi_ssid: String,
    },
    /// Onboarding totalmente concluído.
    Complete,
}

impl OnboardingState {
    /// Retorna o nome textual estático do estado.
    pub fn name(&self) -> &'static str {
        match self {
            Self::SelectLanguage => "SelectLanguage",
            Self::SelectCountry { .. } => "SelectCountry",
            Self::ConnectWifi { .. } => "ConnectWifi",
            Self::CheckOtaUpdate { .. } => "CheckOtaUpdate",
            Self::Complete => "Complete",
        }
    }
}

/// Eventos acionados pela UI ou pelo sistema para transitar de estado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnboardingEvent {
    /// O idioma foi selecionado pelo usuário.
    LanguageSelected(LanguageCode),
    /// O país foi selecionado pelo usuário.
    CountrySelected(CountryCode),
    /// A rede Wi-Fi foi conectada com sucesso.
    WifiConnected {
        /// SSID da rede Wi-Fi conectada.
        ssid: String,
    },
    /// A rotina de validação OTA foi finalizada.
    OtaCheckDone {
        /// Indica se alguma atualização foi aplicada.
        update_applied: bool,
    },
    /// Evento genérico para retornar à tela anterior.
    Back,
}

/// Erro gerado ao tentar realizar uma transição de estado inválida.
#[derive(thiserror::Error, Debug)]
pub enum TransitionError {
    /// A transição solicitada não é válida para o estado atual.
    #[error("Evento '{event}' inválido para o estado '{state}'")]
    InvalidTransition {
        /// Nome do evento tentado.
        event: String,
        /// Nome do estado atual.
        state: String,
    },
}

/// Executa a transição da máquina de estados do onboarding.
///
/// Retorna o novo estado em caso de sucesso. Se a transição for inválida,
/// retorna o estado original e o erro correspondente.
pub fn transition(
    state: OnboardingState,
    event: OnboardingEvent,
) -> Result<OnboardingState, (OnboardingState, TransitionError)> {
    match (state, event) {
        (OnboardingState::SelectLanguage, OnboardingEvent::LanguageSelected(lang)) => {
            Ok(OnboardingState::SelectCountry { language: lang })
        }
        (OnboardingState::SelectCountry { language }, OnboardingEvent::CountrySelected(country)) => {
            Ok(OnboardingState::ConnectWifi { language, country })
        }
        (OnboardingState::ConnectWifi { language, country }, OnboardingEvent::WifiConnected { ssid }) => {
            Ok(OnboardingState::CheckOtaUpdate {
                language,
                country,
                wifi_ssid: ssid,
            })
        }
        (OnboardingState::CheckOtaUpdate { .. }, OnboardingEvent::OtaCheckDone { .. }) => {
            Ok(OnboardingState::Complete)
        }
        (OnboardingState::SelectCountry { .. }, OnboardingEvent::Back) => {
            Ok(OnboardingState::SelectLanguage)
        }
        (OnboardingState::ConnectWifi { language, .. }, OnboardingEvent::Back) => {
            Ok(OnboardingState::SelectCountry { language })
        }
        (OnboardingState::CheckOtaUpdate { language, country, .. }, OnboardingEvent::Back) => {
            Ok(OnboardingState::ConnectWifi { language, country })
        }
        (state, event) => {
            let event_name = format!("{event:?}");
            let state_name = state.name().to_string();
            Err((
                state,
                TransitionError::InvalidTransition {
                    event: event_name,
                    state: state_name,
                },
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions_flow() {
        let state = OnboardingState::SelectLanguage;
        
        let lang = LanguageCode::new("pt-BR").unwrap();
        let state = transition(state, OnboardingEvent::LanguageSelected(lang.clone())).unwrap();
        assert!(matches!(state, OnboardingState::SelectCountry { .. }));

        let country = CountryCode {
            iso2: "BR".to_string(),
            emergency_number: "192".to_string(),
            map_url: "url".to_string(),
        };
        let state = transition(state, OnboardingEvent::CountrySelected(country.clone())).unwrap();
        assert!(matches!(state, OnboardingState::ConnectWifi { .. }));

        let state = transition(state, OnboardingEvent::WifiConnected { ssid: "wifi".to_string() }).unwrap();
        assert!(matches!(state, OnboardingState::CheckOtaUpdate { .. }));

        let state = transition(state, OnboardingEvent::OtaCheckDone { update_applied: false }).unwrap();
        assert_eq!(state, OnboardingState::Complete);
    }

    #[test]
    fn test_invalid_transitions() {
        let state = OnboardingState::SelectLanguage;
        let res = transition(state, OnboardingEvent::Back);
        assert!(res.is_err());
        let (original_state, err) = res.unwrap_err();
        assert_eq!(original_state, OnboardingState::SelectLanguage);
        assert!(err.to_string().contains("Back"));
    }

    #[test]
    fn test_back_transitions() {
        let lang = LanguageCode::new("pt-BR").unwrap();
        let state = OnboardingState::SelectCountry { language: lang.clone() };
        let state = transition(state, OnboardingEvent::Back).unwrap();
        assert_eq!(state, OnboardingState::SelectLanguage);

        let country = CountryCode {
            iso2: "BR".to_string(),
            emergency_number: "192".to_string(),
            map_url: "url".to_string(),
        };
        let state = OnboardingState::ConnectWifi {
            language: lang.clone(),
            country: country.clone(),
        };
        let state = transition(state, OnboardingEvent::Back).unwrap();
        assert_eq!(state, OnboardingState::SelectCountry { language: lang });
    }
}
