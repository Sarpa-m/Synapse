use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Erros de domínio e infraestrutura mapeados do Synapse Desktop.
/// Segue estritamente a regra de zero `.unwrap()` em produção e mensagens amigáveis para a UI.
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "code", content = "message")]
pub enum SynapseError {
    #[error("nome do perfil inválido: deve ter entre 1 e 60 caracteres")]
    InvalidProfileName,

    #[error("system prompt do perfil não pode estar vazio")]
    EmptySystemPrompt,

    #[error("temperatura inválida ({0}): deve estar entre 0.0 e 2.0")]
    InvalidTemperature(String),

    #[error("intervalo de retenção inválido: deve ser entre 1 e 365 dias")]
    InvalidRetentionDays,

    #[error("timeout inválido: deve ser entre 1 e 60 segundos")]
    InvalidTimeout,

    #[error("nenhum texto válido selecionado na janela ativa")]
    EmptySelection,

    #[error("perfil não encontrado: {0}")]
    ProfileNotFound(String),

    #[error("nenhuma operação anterior disponível para desfazer")]
    NothingToUndo,

    #[error("falha ao interagir com a área de transferência: {0}")]
    ClipboardError(String),

    #[error("falha ao simular atalho de teclado: {0}")]
    KeyboardSimulationError(String),

    #[error("falha ao gerenciar foco da janela do sistema: {0}")]
    WindowFocusError(String),

    #[error("falha no banco de dados local: {0}")]
    DatabaseError(String),

    #[error("falha ao acessar o cofre de credenciais do sistema: {0}")]
    SecretStoreError(String),

    #[error("chave de API não configurada no cofre de credenciais")]
    MissingApiKey,

    #[error("chave de API inválida ou sem permissão no provedor ({0})")]
    LlmUnauthorized(String),

    #[error("limite de requisições atingido no provedor de IA ({0})")]
    LlmRateLimit(String),

    #[error("o provedor de IA excedeu o tempo limite de resposta ({0}s)")]
    LlmTimeout(u64),

    #[error("falha de conexão com o provedor de IA: {0}")]
    LlmNetwork(String),
}

pub type SynapseResult<T> = Result<T, SynapseError>;
