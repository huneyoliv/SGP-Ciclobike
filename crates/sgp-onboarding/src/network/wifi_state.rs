//! Estados e eventos do gerenciador de conexão Wi-Fi.

use serde::{Deserialize, Serialize};

/// Estados internos do gerenciador de conexão Wi-Fi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WifiPhase {
    /// Ocioso.
    Idle,
    /// Escaneando redes disponíveis.
    Scanning,
    /// Associando ao ponto de acesso.
    Associating {
        /// SSID da rede alvo.
        ssid: String,
    },
    /// Solicitando IP via DHCP.
    ObtainingIp {
        /// SSID da rede alvo.
        ssid: String,
    },
    /// Conectado ao Wi-Fi com IP válido.
    Connected {
        /// SSID da rede conectada.
        ssid: String,
        /// IP obtido.
        ip: String,
    },
    /// Falha na conexão ou escaneamento.
    Failed {
        /// Causa da falha.
        reason: String,
    },
}

/// Eventos da máquina de estados do Wi-Fi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WifiEvent {
    /// Solicitação de scan.
    ScanRequested,
    /// Scan concluído com sucesso.
    ScanComplete(Vec<AccessPoint>),
    /// Solicitação de conexão.
    ConnectRequested {
        /// SSID da rede alvo.
        ssid: String,
        /// Senha da rede.
        password: String,
    },
    /// Associação e handshake WPA concluídos.
    AssociationComplete,
    /// IP obtido com sucesso.
    IpObtained {
        /// Endereço IP.
        ip: String,
    },
    /// Dispositivo desconectado da rede.
    Disconnected,
    /// Falha em qualquer etapa.
    Error(String),
}

/// Informações de um ponto de acesso Wi-Fi.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessPoint {
    /// Nome da rede.
    pub ssid: String,
    /// Força do sinal em dBm.
    pub signal_dbm: i32,
    /// Indica se é protegida por senha.
    pub secured: bool,
}

impl WifiPhase {
    /// Transiciona o estado com base em um evento.
    #[must_use]
    pub fn next(&self, event: WifiEvent) -> Self {
        match (self, event) {
            (_, WifiEvent::Error(reason)) => WifiPhase::Failed { reason },
            (WifiPhase::Idle | WifiPhase::Failed { .. }, WifiEvent::ScanRequested) => {
                WifiPhase::Scanning
            }
            (WifiPhase::Scanning, WifiEvent::ScanComplete(_)) => WifiPhase::Idle,
            (
                WifiPhase::Idle | WifiPhase::Failed { .. },
                WifiEvent::ConnectRequested { ssid, .. },
            ) => WifiPhase::Associating { ssid },
            (WifiPhase::Associating { ssid }, WifiEvent::AssociationComplete) => {
                WifiPhase::ObtainingIp { ssid: ssid.clone() }
            }
            (WifiPhase::ObtainingIp { ssid }, WifiEvent::IpObtained { ip }) => {
                WifiPhase::Connected {
                    ssid: ssid.clone(),
                    ip,
                }
            }
            (WifiPhase::Connected { .. }, WifiEvent::Disconnected) => WifiPhase::Idle,
            (state, _) => state.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_state_transitions_happy_path() {
        let mut state = WifiPhase::Idle;

        state = state.next(WifiEvent::ScanRequested);
        assert_eq!(state, WifiPhase::Scanning);

        state = state.next(WifiEvent::ScanComplete(vec![]));
        assert_eq!(state, WifiPhase::Idle);

        state = state.next(WifiEvent::ConnectRequested {
            ssid: "CicloNet".into(),
            password: "123".into(),
        });
        assert_eq!(
            state,
            WifiPhase::Associating {
                ssid: "CicloNet".into()
            }
        );

        state = state.next(WifiEvent::AssociationComplete);
        assert_eq!(
            state,
            WifiPhase::ObtainingIp {
                ssid: "CicloNet".into()
            }
        );

        state = state.next(WifiEvent::IpObtained {
            ip: "192.168.1.15".into(),
        });
        assert_eq!(
            state,
            WifiPhase::Connected {
                ssid: "CicloNet".into(),
                ip: "192.168.1.15".into()
            }
        );
    }

    #[test]
    fn test_wifi_state_transition_error() {
        let state = WifiPhase::Idle;
        let next_state = state.next(WifiEvent::Error("Falha DHCP".into()));
        assert_eq!(
            next_state,
            WifiPhase::Failed {
                reason: "Falha DHCP".into()
            }
        );
    }
}
