use std::fmt;

use serde::{Deserialize, Serialize};

use crate::domain::errors::{SynapseError, SynapseResult};

/// Versão semântica (`major.minor.patch`) para comparação segura de releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AppVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl AppVersion {
    /// Faz o parse de uma string de versão como `"0.1.0"` ou `"v0.2.1"`, ignorando sufixos
    /// de pré-lançamento/build (`-beta.1`, `+20261116`).
    ///
    /// # Errors
    /// Retorna [`SynapseError::InvalidVersionFormat`] caso a string não contenha `major.minor.patch` válidos.
    pub fn parse(raw: &str) -> SynapseResult<Self> {
        let trimmed = raw.trim().trim_start_matches(['v', 'V']);
        let core = trimmed.split(['-', '+']).next().unwrap_or("").trim();

        let mut parts = core.split('.');
        let major_str = parts
            .next()
            .ok_or_else(|| SynapseError::InvalidVersionFormat(raw.to_string()))?;
        let minor_str = parts
            .next()
            .ok_or_else(|| SynapseError::InvalidVersionFormat(raw.to_string()))?;
        let patch_str = parts
            .next()
            .ok_or_else(|| SynapseError::InvalidVersionFormat(raw.to_string()))?;

        if parts.next().is_some() {
            return Err(SynapseError::InvalidVersionFormat(raw.to_string()));
        }

        let major = major_str
            .parse::<u64>()
            .map_err(|_| SynapseError::InvalidVersionFormat(raw.to_string()))?;
        let minor = minor_str
            .parse::<u64>()
            .map_err(|_| SynapseError::InvalidVersionFormat(raw.to_string()))?;
        let patch = patch_str
            .parse::<u64>()
            .map_err(|_| SynapseError::InvalidVersionFormat(raw.to_string()))?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for AppVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Metadados de uma release disponível no GitHub Releases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateReleaseInfo {
    pub version: String,
    pub tag_name: String,
    pub release_notes: String,
    pub published_at: String,
    pub download_url: String,
    pub asset_name: String,
    pub asset_size_bytes: u64,
}

/// Resultado da verificação de atualização.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UpdateCheckResult {
    UpToDate {
        current_version: String,
    },
    UpdateAvailable {
        current_version: String,
        release: UpdateReleaseInfo,
    },
    Skipped {
        current_version: String,
        skipped_version: String,
    },
}

/// Progresso de download do instalador da atualização.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: u8,
}

impl UpdateProgress {
    #[must_use]
    pub fn new(downloaded_bytes: u64, total_bytes: u64) -> Self {
        let percentage = downloaded_bytes
            .saturating_mul(100)
            .checked_div(total_bytes)
            .map_or(0, |ratio| u8::try_from(ratio.min(100)).unwrap_or(100));
        Self {
            downloaded_bytes,
            total_bytes,
            percentage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_version_parses_standard_and_v_prefixed_versions() {
        let v1 = AppVersion::parse("0.1.0").unwrap();
        let v2 = AppVersion::parse("v0.2.1").unwrap();
        let v3 = AppVersion::parse("V1.10.3-beta.1+build42").unwrap();

        assert_eq!(v1.to_string(), "0.1.0");
        assert_eq!(v2.to_string(), "0.2.1");
        assert_eq!(v3.to_string(), "1.10.3");
        assert!(v2 > v1);
        assert!(v3 > v2);
    }

    #[test]
    fn test_app_version_rejects_invalid_strings() {
        assert_eq!(
            AppVersion::parse(""),
            Err(SynapseError::InvalidVersionFormat(String::new()))
        );
        assert_eq!(
            AppVersion::parse("1.0"),
            Err(SynapseError::InvalidVersionFormat("1.0".to_string()))
        );
        assert_eq!(
            AppVersion::parse("1.0.0.1"),
            Err(SynapseError::InvalidVersionFormat("1.0.0.1".to_string()))
        );
        assert_eq!(
            AppVersion::parse("v1.x.0"),
            Err(SynapseError::InvalidVersionFormat("v1.x.0".to_string()))
        );
    }

    #[test]
    fn test_update_progress_percentage_calculation() {
        let p0 = UpdateProgress::new(0, 0);
        assert_eq!(p0.percentage, 0);

        let p50 = UpdateProgress::new(500, 1000);
        assert_eq!(p50.percentage, 50);

        let p100 = UpdateProgress::new(1500, 1000);
        assert_eq!(p100.percentage, 100);
    }
}
