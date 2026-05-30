//! Cliente HTTP para comunicacao com a API do Strava via OAuth2.

use serde::{Deserialize, Serialize};
use sgp_core::{StravaTokens, STRAVA_CLIENT_ID, STRAVA_CLIENT_SECRET};
use std::time::Duration;

/// Cliente para envio de atividades e refresh de tokens no Strava.
pub struct StravaClient {
    tokens: StravaTokens,
    client: reqwest::Client,
}

/// Identificador do upload retornado pelo Strava.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StravaUploadId(pub u64);

/// Status de processamento do arquivo no Strava.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StravaUploadStatus {
    /// Sincronizacao ainda pendente.
    Pending,
    /// Upload concluido e integrado como atividade.
    Ready {
        /// ID da atividade gerada no Strava.
        activity_id: u64,
    },
    /// Erro de processamento interno no Strava.
    Error(String),
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
}

#[derive(Deserialize)]
struct UploadResponse {
    id: u64,
}

#[derive(Deserialize)]
struct UploadStatusResponse {
    status: String,
    activity_id: Option<u64>,
    error: Option<String>,
}

impl StravaClient {
    /// Inicializa o cliente carregando os tokens a partir das credenciais do usuario.
    pub fn new(tokens: StravaTokens) -> Self {
        Self {
            tokens,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    /// Retorna os tokens vigentes em cache.
    pub fn tokens(&self) -> &StravaTokens {
        &self.tokens
    }

    /// Realiza a renovacao do access_token caso esteja expirado ou proximo de expirar.
    pub async fn refresh_if_needed(&mut self) -> Result<(), String> {
        if !self.tokens.needs_refresh() {
            return Ok(());
        }

        let url = "https://www.strava.com/oauth/token";
        let body = format!(
            "client_id={}&client_secret={}&grant_type=refresh_token&refresh_token={}",
            STRAVA_CLIENT_ID, STRAVA_CLIENT_SECRET, self.tokens.refresh_token
        );

        let res = self
            .client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.as_bytes().to_vec())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("Refresh falhou com status: {}", res.status()));
        }

        let data: TokenResponse = res.json().await.map_err(|e| e.to_string())?;
        self.tokens.access_token = data.access_token;
        self.tokens.refresh_token = data.refresh_token;
        self.tokens.expires_at = data.expires_at;

        let _ = self.tokens.save();
        Ok(())
    }

    /// Realiza o upload do conteudo de um arquivo GPX para o Strava.
    pub async fn upload_gpx(
        &mut self,
        gpx_content: &str,
        name: &str,
    ) -> Result<StravaUploadId, String> {
        self.refresh_if_needed().await?;

        let url = "https://www.strava.com/api/v3/uploads";
        let boundary = format!(
            "---------------------------{}",
            uuid::Uuid::new_v4().to_string().replace('-', "")
        );
        let mut body = Vec::new();

        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"data_type\"\r\n\r\n");
        body.extend_from_slice(b"gpx\r\n");

        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"name\"\r\n\r\n");
        body.extend_from_slice(format!("{name}\r\n").as_bytes());

        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"file\"; filename=\"activity.gpx\"\r\n",
        );
        body.extend_from_slice(b"Content-Type: application/gpx+xml\r\n\r\n");
        body.extend_from_slice(gpx_content.as_bytes());
        body.extend_from_slice(b"\r\n");

        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        let res = self
            .client
            .post(url)
            .bearer_auth(&self.tokens.access_token)
            .header(
                "Content-Type",
                &format!("multipart/form-data; boundary={boundary}"),
            )
            .body(body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("Upload falhou com status: {}", res.status()));
        }

        let data: UploadResponse = res.json().await.map_err(|e| e.to_string())?;
        Ok(StravaUploadId(data.id))
    }

    /// Consulta o processamento do upload informado no Strava.
    pub async fn poll_upload_status(
        &self,
        upload_id: StravaUploadId,
    ) -> Result<StravaUploadStatus, String> {
        let url = format!("https://www.strava.com/api/v3/uploads/{}", upload_id.0);
        let res = self
            .client
            .get(&url)
            .bearer_auth(&self.tokens.access_token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("Consulta falhou com status: {}", res.status()));
        }

        let data: UploadStatusResponse = res.json().await.map_err(|e| e.to_string())?;
        if let Some(err) = data.error {
            return Ok(StravaUploadStatus::Error(err));
        }

        if let Some(act_id) = data.activity_id {
            Ok(StravaUploadStatus::Ready {
                activity_id: act_id,
            })
        } else if data.status.contains("Ready") {
            Ok(StravaUploadStatus::Ready { activity_id: 0 })
        } else if data.status.contains("Error") {
            Ok(StravaUploadStatus::Error(data.status))
        } else {
            Ok(StravaUploadStatus::Pending)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strava_client_new() {
        let tokens = StravaTokens {
            access_token: "123".into(),
            refresh_token: "456".into(),
            expires_at: 0,
        };
        let client = StravaClient::new(tokens.clone());
        assert_eq!(client.tokens().access_token, "123");
    }
}
