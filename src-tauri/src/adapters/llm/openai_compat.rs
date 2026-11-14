use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::domain::errors::{SynapseError, SynapseResult};
use crate::ports::llm_port::{LlmProviderPort, LlmRequest};

/// Cliente HTTP compatível com a especificação OpenAI (`/v1/chat/completions`),
/// suportando DeepSeek, OpenAI, Groq e Ollama local (RN-04).
#[derive(Clone)]
pub struct OpenAiCompatClient {
    http_client: Client,
    custom_timeout_override: Option<Duration>,
}

#[derive(Debug, Serialize)]
struct ChatCompletionPayload<'a> {
    model: &'a str,
    temperature: f32,
    messages: [ChatMessage<'a>; 2],
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: String,
}

impl OpenAiCompatClient {
    /// Cria um novo cliente HTTP com `connect_timeout` de 5s e `timeout` padrão de 10s (RN-04).
    ///
    /// # Errors
    /// Retorna [`SynapseError::LlmNetwork`] caso a inicialização TLS falhe.
    pub fn new() -> SynapseResult<Self> {
        let http_client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| SynapseError::LlmNetwork(e.to_string()))?;

        Ok(Self {
            http_client,
            custom_timeout_override: None,
        })
    }

    /// Permite sobrescrever o timeout em milissegundos (útil em testes de integração com `wiremock`).
    #[must_use]
    pub fn with_timeout_override(mut self, timeout: Duration) -> Self {
        self.custom_timeout_override = Some(timeout);
        self
    }

    fn build_endpoint(base_url: &str) -> String {
        let trimmed = base_url.trim().trim_end_matches('/');
        if trimmed.ends_with("/chat/completions") {
            trimmed.to_string()
        } else {
            format!("{trimmed}/chat/completions")
        }
    }
}

impl LlmProviderPort for OpenAiCompatClient {
    fn complete(
        &self,
        request: LlmRequest,
    ) -> Pin<Box<dyn Future<Output = SynapseResult<String>> + Send + 'static>> {
        let client = self.http_client.clone();
        let timeout_override = self.custom_timeout_override;

        Box::pin(async move {
            let endpoint = Self::build_endpoint(&request.base_url);
            let req_timeout = timeout_override
                .unwrap_or_else(|| Duration::from_secs(request.timeout_secs.max(1)));

            let payload = ChatCompletionPayload {
                model: &request.model,
                temperature: request.temperature,
                messages: [
                    ChatMessage {
                        role: "system",
                        content: &request.system_prompt,
                    },
                    ChatMessage {
                        role: "user",
                        content: &request.user_text,
                    },
                ],
            };

            let response = client
                .post(&endpoint)
                .bearer_auth(request.api_key.trim())
                .timeout(req_timeout)
                .json(&payload)
                .send()
                .await
                .map_err(|err| {
                    if err.is_timeout() {
                        SynapseError::LlmTimeout(request.timeout_secs)
                    } else {
                        SynapseError::LlmNetwork(err.to_string())
                    }
                })?;

            let status = response.status();
            if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
                return Err(SynapseError::LlmUnauthorized(format!("HTTP {status}")));
            }
            if status == StatusCode::TOO_MANY_REQUESTS {
                return Err(SynapseError::LlmRateLimit(format!("HTTP {status}")));
            }
            if !status.is_success() {
                let body_snippet = response.text().await.unwrap_or_default();
                return Err(SynapseError::LlmNetwork(format!(
                    "HTTP {status}: {}",
                    body_snippet.chars().take(120).collect::<String>()
                )));
            }

            let parsed: ChatCompletionResponse = response
                .json()
                .await
                .map_err(|e| SynapseError::LlmNetwork(format!("JSON inválido do provedor: {e}")))?;

            let first =
                parsed.choices.into_iter().next().ok_or_else(|| {
                    SynapseError::LlmNetwork("resposta sem choices da LLM".into())
                })?;

            Ok(first.message.content)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_openai_compat_success_request_and_parsing() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("authorization", "Bearer sk-test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-123",
                "choices": [
                    {
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "Texto corrigido com sucesso."
                        }
                    }
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = OpenAiCompatClient::new().unwrap();
        let req = LlmRequest {
            base_url: format!("{}/v1", server.uri()),
            api_key: "sk-test-key".to_string(),
            model: "deepseek-chat".to_string(),
            system_prompt: "Corrija o texto.".to_string(),
            user_text: "Texto errrado.".to_string(),
            temperature: 0.1,
            timeout_secs: 10,
        };

        let output = client.complete(req).await.unwrap();
        assert_eq!(output, "Texto corrigido com sucesso.");
    }

    #[tokio::test]
    async fn test_openai_compat_401_unauthorized() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Invalid API Key"))
            .expect(1)
            .mount(&server)
            .await;

        let client = OpenAiCompatClient::new().unwrap();
        let req = LlmRequest {
            base_url: format!("{}/v1", server.uri()),
            api_key: "sk-bad-key".to_string(),
            model: "deepseek-chat".to_string(),
            system_prompt: "Corrija".to_string(),
            user_text: "Ola".to_string(),
            temperature: 0.1,
            timeout_secs: 10,
        };

        let err = client.complete(req).await.unwrap_err();
        assert!(matches!(err, SynapseError::LlmUnauthorized(_)));
    }

    #[tokio::test]
    async fn test_openai_compat_429_rate_limit() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(429))
            .expect(1)
            .mount(&server)
            .await;

        let client = OpenAiCompatClient::new().unwrap();
        let req = LlmRequest {
            base_url: format!("{}/v1", server.uri()),
            api_key: "sk-key".to_string(),
            model: "deepseek-chat".to_string(),
            system_prompt: "Corrija".to_string(),
            user_text: "Ola".to_string(),
            temperature: 0.1,
            timeout_secs: 10,
        };

        let err = client.complete(req).await.unwrap_err();
        assert!(matches!(err, SynapseError::LlmRateLimit(_)));
    }

    #[tokio::test]
    async fn test_openai_compat_timeout_enforced() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(300))
                    .set_body_json(json!({
                        "choices": [{ "message": { "role": "assistant", "content": "Demorou" } }]
                    })),
            )
            .mount(&server)
            .await;

        let client = OpenAiCompatClient::new()
            .unwrap()
            .with_timeout_override(Duration::from_millis(80));

        let req = LlmRequest {
            base_url: format!("{}/v1", server.uri()),
            api_key: "sk-key".to_string(),
            model: "deepseek-chat".to_string(),
            system_prompt: "Corrija".to_string(),
            user_text: "Ola".to_string(),
            temperature: 0.1,
            timeout_secs: 10,
        };

        let err = client.complete(req).await.unwrap_err();
        assert_eq!(err, SynapseError::LlmTimeout(10));
    }
}
