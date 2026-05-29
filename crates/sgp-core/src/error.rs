//! Hierarquia de erros tipados para o SGP-Ciclobike.

/// Erros relacionados à persistência e validação de configurações.
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    /// Arquivo de configuração não encontrado.
    #[error("Arquivo não encontrado: {0}")]
    NotFound(String),
    /// Erro de I/O do sistema de arquivos.
    #[error("Falha de I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Falha na desserialização do TOML.
    #[error("TOML inválido: {0}")]
    Parse(#[from] toml::de::Error),
    /// Falha na serialização do TOML.
    #[error("Falha ao serializar: {0}")]
    Serialize(#[from] toml::ser::Error),
    /// O código de idioma especificado é inválido.
    #[error("Código de idioma inválido: '{0}'")]
    InvalidLanguageCode(String),
    /// Uma etapa obrigatória do wizard não foi preenchida.
    #[error("Etapa incompleta: '{0}' não foi concluída")]
    IncompleteStep(&'static str),
}

/// Erros relacionados ao processo de atualização OTA.
#[derive(thiserror::Error, Debug)]
pub enum OtaError {
    /// Tempo limite esgotado ao verificar atualizações.
    #[error("Timeout ao verificar atualizações")]
    CheckTimeout,
    /// O dispositivo já está na última versão de firmware.
    #[error("Já na versão mais recente")]
    AlreadyUpToDate,
    /// Falha genérica de rede.
    #[error("Erro de rede: {0}")]
    Network(String),
    /// Divergência de checksum SHA256 do binário baixado.
    #[error("SHA256 inválido: esperado={expected}, obtido={got}")]
    ChecksumMismatch {
        /// Hash esperado.
        expected: String,
        /// Hash obtido.
        got: String,
    },
    /// Falha de escrita no armazenamento durante o download.
    #[error("Falha ao escrever arquivo de atualização: {0}")]
    WriteError(#[from] std::io::Error),
    /// Formato de versão semântica inválido.
    #[error("Versão inválida: {0}")]
    InvalidVersion(String),
}

impl OtaError {
    /// Indica se o erro é temporário e permite continuar o fluxo normal.
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::CheckTimeout | Self::Network(_))
    }
}

/// Enumerador envelope para erros globais da aplicação.
#[derive(thiserror::Error, Debug)]
pub enum SgpError {
    /// Erro de configuração.
    #[error("Configuração: {0}")]
    Config(#[from] ConfigError),
    /// Erro de OTA.
    #[error("OTA: {0}")]
    Ota(#[from] OtaError),
    /// Erro na interface gráfica.
    #[error("UI: {0}")]
    Ui(String),
    /// Erro de rede ou comunicação física.
    #[error("Rede: {0}")]
    Network(String),
}
