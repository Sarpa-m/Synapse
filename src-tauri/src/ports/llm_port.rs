use std::future::Future;
use std::pin::Pin;

use crate::domain::errors::SynapseResult;

/// Parâmetros de uma requisição de completude enviados ao provedor LLM compatível com OpenAI.
#[derive(Debug, Clone, PartialEq)]
pub struct LlmRequest {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
    pub user_text: String,
    pub temperature: f32,
    pub timeout_secs: u64,
}

/// Porta de comunicação assíncrona com o provedor de IA (OpenAI / DeepSeek / Ollama).
#[cfg_attr(test, mockall::automock)]
pub trait LlmProviderPort: Send + Sync {
    fn complete(
        &self,
        request: LlmRequest,
    ) -> Pin<Box<dyn Future<Output = SynapseResult<String>> + Send + 'static>>;
}
