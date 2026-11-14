use serde::{Deserialize, Serialize};

/// Registro de uma operação realizada pelo usuário, armazenado no cofre de histórico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub profile_name: String,
    pub original_text: String,
    pub generated_text: String,
    pub created_at: String,
}

/// Sanitiza a resposta da LLM conforme a regra RN-06:
/// Remove cercas de bloco de código markdown (` ``` `) externas se o texto original
/// selecionado pelo usuário não continha blocos de código.
#[must_use]
pub fn sanitize_llm_output(original_text: &str, raw_output: &str) -> String {
    let trimmed = raw_output.trim();

    // Se o texto original já continha cercas de código, preservamos a saída do modelo intacta (apenas trim).
    if original_text.contains("```") {
        return trimmed.to_string();
    }

    if let Some(rest) = trimmed.strip_prefix("```") {
        if let Some(without_closing) = rest.strip_suffix("```") {
            // Se houver uma primeira linha identificando a linguagem (ex: ```text\nConteúdo),
            // removemos apenas o identificador de linguagem caso haja quebra de linha.
            if let Some((first_line, body)) = without_closing.split_once('\n') {
                if !first_line.contains(' ') && first_line.len() <= 20 {
                    return body.trim().to_string();
                }
            }
            return without_closing.trim().to_string();
        }
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_llm_output_strips_outer_code_fences_for_plain_text() {
        let original = "nois vai na reuniao amanha";
        let llm_raw = "```\nNós vamos à reunião amanhã.\n```";
        assert_eq!(
            sanitize_llm_output(original, llm_raw),
            "Nós vamos à reunião amanhã."
        );

        let llm_with_lang = "```text\nNós vamos à reunião amanhã.\n```";
        assert_eq!(
            sanitize_llm_output(original, llm_with_lang),
            "Nós vamos à reunião amanhã."
        );

        let llm_inline_fence = "```Nós vamos à reunião amanhã.```";
        assert_eq!(
            sanitize_llm_output(original, llm_inline_fence),
            "Nós vamos à reunião amanhã."
        );
    }

    #[test]
    fn test_sanitize_llm_output_preserves_fences_when_original_had_code() {
        let original = "```rust\nfn main() {}\n```";
        let llm_raw = "```rust\nfn main() {\n    println!(\"ok\");\n}\n```";
        assert_eq!(sanitize_llm_output(original, llm_raw), llm_raw);
    }
}
