use serde::{Deserialize, Serialize};

use crate::domain::errors::{SynapseError, SynapseResult};

pub const MAX_PROFILE_NAME_LEN: usize = 60;
pub const MIN_TEMPERATURE: f32 = 0.0;
pub const MAX_TEMPERATURE: f32 = 2.0;

/// Entidade de domínio que representa um Perfil de Assistência de IA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub temperature: f32,
    pub shortcut: Option<String>,
    pub is_default: bool,
}

impl Profile {
    /// Cria e valida um novo perfil de acordo com a regra RN-05.
    ///
    /// # Errors
    /// Retorna [`SynapseError::InvalidProfileName`] se o nome for vazio ou > 60 caracteres,
    /// [`SynapseError::EmptySystemPrompt`] se o prompt for vazio, ou
    /// [`SynapseError::InvalidTemperature`] se a temperatura estiver fora de `0.0..=2.0`.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        system_prompt: impl Into<String>,
        temperature: f32,
        shortcut: Option<String>,
        is_default: bool,
    ) -> SynapseResult<Self> {
        let id = id.into().trim().to_string();
        let name = name.into().trim().to_string();
        let system_prompt = system_prompt.into().trim().to_string();

        Self::validate_fields(&name, &system_prompt, temperature)?;

        let normalized_shortcut = shortcut
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        Ok(Self {
            id: if id.is_empty() {
                Self::slugify_id(&name)
            } else {
                id
            },
            name,
            system_prompt,
            temperature,
            shortcut: normalized_shortcut,
            is_default,
        })
    }

    /// Valida os campos de negócio de um perfil.
    pub fn validate_fields(name: &str, system_prompt: &str, temperature: f32) -> SynapseResult<()> {
        let name_len = name.trim().chars().count();
        if name_len == 0 || name_len > MAX_PROFILE_NAME_LEN {
            return Err(SynapseError::InvalidProfileName);
        }

        if system_prompt.trim().is_empty() {
            return Err(SynapseError::EmptySystemPrompt);
        }

        if temperature.is_nan() || !(MIN_TEMPERATURE..=MAX_TEMPERATURE).contains(&temperature) {
            return Err(SynapseError::InvalidTemperature(format!("{temperature}")));
        }

        Ok(())
    }

    /// Retorna os Perfis Padrão do MVP (RN-05).
    #[must_use]
    pub fn default_seeds() -> Vec<Self> {
        vec![
            Self {
                id: "default-correcao-silenciosa".to_string(),
                name: "Correção Silenciosa".to_string(),
                system_prompt: "Atue como um revisor especialista em Língua Portuguesa (PT-BR). Sua função é interpretar e corrigir textos de um usuário com dislexia, que apresentam severos erros ortográficos, gramaticais, de concordância e de digitação. Diretrizes de Correção: Precisão Gramatical: Corrija ortografia, pontuação e gramática adequando o texto às regras da língua portuguesa brasileira. Expansão de Reduções Coloquiais: Você deve corrigir e expandir todas as abreviações de internet e reduções de fala para a forma completa correta. Exemplos obrigatórios: mude \"tava\" para \"estava\", \"pra\" para \"para\", \"mto\" para \"muito\", \"vc\" para \"você\", \"q\" para \"que\", \"tb\" para \"também\". Preservação de Identidade: Mantenha rigorosamente o tom, o estilo e a intenção original da mensagem. O texto final deve soar natural para quem o escreveu. Vocabulário Contextual: Conserve gírias e expressões populares amplamente aceitas e utilizadas no contexto da frase. Não eleve o nível de formalidade desnecessariamente (não deixe o texto \"robótico\" ou excessivamente culto se a intenção for casual). Formato de Saída Obrigatório: Retorne ÚNICA e EXCLUSIVAMENTE o texto corrigido. É estritamente proibido incluir aspas, saudações, explicações sobre as correções, notas de rodapé ou qualquer texto adicional.".to_string(),
                temperature: 0.2,
                shortcut: Some("1".to_string()),
                is_default: true,
            },
            Self {
                id: "default-reescrita-profissional".to_string(),
                name: "Reescrita Profissional e Executiva".to_string(),
                system_prompt: "Atue como um especialista em comunicação corporativa e redação executiva em Língua Portuguesa (PT-BR). Sua função é transformar textos brutos, informais ou mal estruturados em comunicações de trabalho impecáveis (e-mails, relatórios ou mensagens corporativas). Diretrizes de Reescrita: Tom e Estilo: Mantenha um tom cordial, polido, objetivo e altamente profissional. Elimine gírias, coloquialismos, emoções excessivas ou tons agressivos/passivo-agressivos. Clareza e Concisão:Estruture a informação de forma lógica e clara, facilitando a leitura rápida e a tomada de decisão no ambiente corporativo.  Precisão Gramatical e Vocabulário: Aplique rigorosamente a norma-padrão da língua portuguesa. Substitua termos informais por vocabulário corporativo adequado, sem tornar o texto pedante ou excessivamente complexo. Formato de Saída Obrigatório: Retorne ÚNICA e EXCLUSIVAMENTE o texto reescrito. É estritamente proibido incluir aspas, introduções (ex: \"Aqui está o texto...\"), explicações sobre as mudanças ou qualquer comentário adicional.".to_string(),
                temperature: 0.5,
                shortcut: Some("2".to_string()),
                is_default: true,
            },
            Self {
                id: "default-resumo-tecnico".to_string(),
                name: "Resumo Técnico e Arquitetura".to_string(),
                system_prompt: "Atue como um Especialista em Documentação Técnica e Arquitetura de Sistemas. Sua função é analisar textos, logs, discussões ou relatórios e extrair resumos altamente precisos, preservando a integridade das informações técnicas.\n\nDiretrizes de Síntese:\n\nPreservação de Dados Críticos: Mantenha rigorosamente intactos os jargões da área, siglas, configurações, linhas de comando, parâmetros de rede, métricas, referências a normas de conformidade e especificações de hardware/software. Não simplifique a terminologia técnica.\n\nEstrutura Lógica e Modular: Organize a informação de forma limpa. Utilize bullet points ou tópicos curtos para categorizar dados importantes (ex: Diagnóstico, Causa-Raiz, Configurações Aplicadas, Próximos Passos), facilitando a leitura rápida operacional.\n\nObjetividade Extrema: Elimine qualquer ruído, coloquialismo, emoção ou contexto irrelevante da mensagem original. O foco deve ser puramente analítico, factual e descritivo.\n\nFormato de Saída Obrigatório: Retorne ÚNICA e EXCLUSIVAMENTE o resumo técnico. É estritamente proibido incluir aspas, saudações, explicações sobre o que foi resumido ou qualquer texto adicional.".to_string(),
                temperature: 0.5,
                shortcut: Some("3".to_string()),
                is_default: true,
            },
            Self {
                id: "default-traducao-en-pt".to_string(),
                name: "Tradução EN-PT".to_string(),
                system_prompt: "Se o texto a seguir estiver em inglês (ou outro idioma), traduza-o naturalmente para o português brasileiro. Se já estiver em português, traduza-o para o inglês. Retorne APENAS a tradução.".to_string(),
                temperature: 0.2,
                shortcut: Some("4".to_string()),
                is_default: true,
            },

        ]
    }

    fn slugify_id(name: &str) -> String {
        let slug: String = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        slug.trim_matches('-').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_valid_creation() {
        let p0 = Profile::new("id-1", "Corrigir", "Corrija o texto", 0.0, None, false).unwrap();
        assert_eq!(p0.name, "Corrigir");
        assert!((p0.temperature - 0.0).abs() < f32::EPSILON);

        let p1 = Profile::new(
            "id-2",
            "Formalizar",
            "Formalize o texto",
            0.7,
            Some("Ctrl+1".to_string()),
            false,
        )
        .unwrap();
        assert_eq!(p1.shortcut.as_deref(), Some("Ctrl+1"));

        let p2 = Profile::new(
            "id-3",
            "Criativo",
            "Reescreva",
            2.0,
            Some("  ".to_string()),
            false,
        )
        .unwrap();
        assert!(p2.shortcut.is_none());
    }

    #[test]
    fn test_profile_invalid_temperature() {
        let err_low = Profile::new("1", "Teste", "Prompt", -0.1, None, false).unwrap_err();
        assert!(matches!(err_low, SynapseError::InvalidTemperature(_)));

        let err_high = Profile::new("1", "Teste", "Prompt", 2.1, None, false).unwrap_err();
        assert!(matches!(err_high, SynapseError::InvalidTemperature(_)));

        let err_nan = Profile::new("1", "Teste", "Prompt", f32::NAN, None, false).unwrap_err();
        assert!(matches!(err_nan, SynapseError::InvalidTemperature(_)));
    }

    #[test]
    fn test_profile_empty_or_long_name() {
        let err_empty = Profile::new("1", "", "Prompt", 0.5, None, false).unwrap_err();
        assert_eq!(err_empty, SynapseError::InvalidProfileName);

        let err_spaces = Profile::new("1", "    ", "Prompt", 0.5, None, false).unwrap_err();
        assert_eq!(err_spaces, SynapseError::InvalidProfileName);

        let long_name = "a".repeat(61);
        let err_long = Profile::new("1", long_name, "Prompt", 0.5, None, false).unwrap_err();
        assert_eq!(err_long, SynapseError::InvalidProfileName);
    }

    #[test]
    fn test_profile_empty_prompt_rejected() {
        let err = Profile::new("1", "Nome Válido", "   ", 0.5, None, false).unwrap_err();
        assert_eq!(err, SynapseError::EmptySystemPrompt);
    }

    #[test]
    fn test_default_seeds_are_valid() {
        let seeds = Profile::default_seeds();
        assert_eq!(seeds.len(), 4);
        for seed in seeds {
            assert!(
                Profile::validate_fields(&seed.name, &seed.system_prompt, seed.temperature).is_ok()
            );
            assert!(seed.is_default);
        }
    }
}
