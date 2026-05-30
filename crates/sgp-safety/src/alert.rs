//! Gerenciador da janela de alerta pós-queda e timer de emergência de 30 segundos.

use std::time::Duration;
use tokio::sync::{mpsc, watch};

/// Estado atual da janela de alerta do ciclocomputador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertState {
    /// Sem nenhum incidente ativo.
    Idle,
    /// Janela de alerta ativa exibindo contagem regressiva para o envio da chamada de emergência.
    Alerting {
        /// Segundos restantes para o disparo final (inicia em 30).
        seconds_remaining: u8,
    },
    /// A janela de 30s esgotou sem resposta do usuário. Protocolo de emergência disparado!
    EmergencyTriggered,
}

/// Inputs enviados pelo usuário (através da UI) para o gerenciador de alertas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertInput {
    /// O usuário clicou no botão "ESTOU BEM", indicando que a queda foi leve e o alarme deve ser cancelado.
    Cancel,
}

/// Gerenciador assíncrono que processa incidentes e timers de segurança.
pub struct AlertManager {
    input_rx: mpsc::Receiver<AlertInput>,
    state_tx: watch::Sender<AlertState>,
    emergency_tx: mpsc::Sender<()>,
}

impl AlertManager {
    /// Cria uma nova instância com canais de input, sincronismo de estado e trigger de emergência.
    pub fn new(
        input_rx: mpsc::Receiver<AlertInput>,
        state_tx: watch::Sender<AlertState>,
        emergency_tx: mpsc::Sender<()>,
    ) -> Self {
        Self {
            input_rx,
            state_tx,
            emergency_tx,
        }
    }

    /// Loop de processamento assíncrono que gerencia a transição e a contagem regressiva.
    pub async fn run(mut self, mut fall_rx: mpsc::Receiver<()>) {
        loop {
            tokio::select! {
                // Aguarda um novo incidente de queda ser detectado
                Some(()) = fall_rx.recv() => {
                    tracing::warn!("Queda detectada! Iniciando janela de alerta de 30 segundos...");
                    self.handle_incident().await;
                }
            }
        }
    }

    /// Gerencia o countdown de 30 segundos pós-queda.
    async fn handle_incident(&mut self) {
        let mut countdown = 30u8;
        let _ = self.state_tx.send(AlertState::Alerting {
            seconds_remaining: countdown,
        });

        let mut interval = tokio::time::interval(Duration::from_secs(1));
        // Ignora o primeiro trigger imediato do interval
        interval.tick().await;

        loop {
            tokio::select! {
                // Atualiza a cada segundo
                _ = interval.tick() => {
                    countdown -= 1;
                    if countdown == 0 {
                        tracing::error!("Janela de alerta esgotou sem resposta! Disparando protocolo de emergência...");
                        let _ = self.state_tx.send(AlertState::EmergencyTriggered);
                        let _ = self.emergency_tx.send(()).await;
                        break;
                    }
                    let _ = self.state_tx.send(AlertState::Alerting { seconds_remaining: countdown });
                }
                // Recebe cancelamento da UI
                Some(AlertInput::Cancel) = self.input_rx.recv() => {
                    tracing::info!("Alerta de emergência cancelado pelo usuário.");
                    let _ = self.state_tx.send(AlertState::Idle);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_manager_cancel() {
        let (input_tx, input_rx) = mpsc::channel(10);
        let (state_tx, mut state_rx) = watch::channel(AlertState::Idle);
        let (em_tx, mut em_rx) = mpsc::channel(10);
        let (fall_tx, fall_rx) = mpsc::channel(10);

        let manager = AlertManager::new(input_rx, state_tx, em_tx);
        tokio::spawn(manager.run(fall_rx));

        // Simula disparo de queda
        fall_tx.send(()).await.unwrap();

        // Aguarda mudar para Alerting
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(matches!(*state_rx.borrow(), AlertState::Alerting { .. }));

        // Simula o cancelamento imediato vindo da UI
        input_tx.send(AlertInput::Cancel).await.unwrap();

        // Aguarda restaurar para Idle
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(*state_rx.borrow(), AlertState::Idle);
        assert!(em_rx.try_recv().is_err());
    }
}
