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

    /// Retorna os 3 Perfis Padrão obrigatórios do MVP (RN-05).
    #[must_use]
    pub fn default_seeds() -> Vec<Self> {
        vec![
            Self {
                id: "default-correcao-silenciosa".to_string(),
                name: "Correção Silenciosa".to_string(),
                system_prompt: "Corrija erros gramaticais, ortográficos e de pontuação do texto a seguir, mantendo estritamente o tom, o idioma e a intenção original. Retorne APENAS o texto corrigido, sem explicações ou aspas.".to_string(),
                temperature: 0.1,
                shortcut: Some("1".to_string()),
                is_default: true,
            },
            Self {
                id: "default-reescrita-profissional".to_string(),
                name: "Reescrita Profissional".to_string(),
                system_prompt: "Reescreva o texto a seguir tornando-o claro, polido, cordial e adequado para comunicação profissional de trabalho (como e-mails ou mensagens corporativas). Retorne APENAS o texto reescrito, sem comentários adicionais.".to_string(),
                temperature: 0.4,
                shortcut: Some("2".to_string()),
                is_default: true,
            },
            Self {
                id: "default-traducao-en-pt".to_string(),
                name: "Tradução EN-PT".to_string(),
                system_prompt: "Se o texto a seguir estiver em inglês (ou outro idioma), traduza-o naturalmente para o português brasileiro. Se já estiver em português, traduza-o para o inglês. Retorne APENAS a tradução.".to_string(),
                temperature: 0.2,
                shortcut: Some("3".to_string()),
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
        assert_eq!(seeds.len(), 3);
        for seed in seeds {
            assert!(
                Profile::validate_fields(&seed.name, &seed.system_prompt, seed.temperature).is_ok()
            );
            assert!(seed.is_default);
        }
    }
}
