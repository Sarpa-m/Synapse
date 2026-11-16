use std::path::Path;
use std::process::Command;

use crate::domain::errors::{SynapseError, SynapseResult};
use crate::ports::update_port::UpdateInstallerPort;

/// Adaptador Windows para execução do instalador ou substituição do executável portátil.
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowsInstallerAdapter;

impl WindowsInstallerAdapter {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn is_msi(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("msi"))
    }

    fn is_portable_standalone_exe(path: &Path) -> bool {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let is_exe = Path::new(&name)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "exe");
        is_exe && !name.contains("setup") && !name.contains("install")
    }
}

impl UpdateInstallerPort for WindowsInstallerAdapter {
    fn launch_installer(&self, installer_path: &Path) -> SynapseResult<()> {
        if !installer_path.exists() {
            return Err(SynapseError::UpdateInstallFailed(format!(
                "arquivo de instalação não encontrado em {}",
                installer_path.display()
            )));
        }

        if Self::is_msi(installer_path) {
            Command::new("msiexec")
                .arg("/i")
                .arg(installer_path)
                .spawn()
                .map_err(|e| SynapseError::UpdateInstallFailed(e.to_string()))?;
            return Ok(());
        }

        if Self::is_portable_standalone_exe(installer_path) {
            if let Ok(current_exe) = std::env::current_exe() {
                let script_path = installer_path.with_extension("cmd");
                let script_content = format!(
                    "@echo off\r\ntimeout /t 2 /nobreak > NUL\r\nmove /Y \"{}\" \"{}\"\r\nstart \"\" \"{}\"\r\ndel \"%~f0\"\r\n",
                    installer_path.display(),
                    current_exe.display(),
                    current_exe.display()
                );
                if std::fs::write(&script_path, script_content).is_ok() {
                    let mut cmd = Command::new("cmd");
                    cmd.arg("/C").arg(&script_path);
                    #[cfg(windows)]
                    {
                        use std::os::windows::process::CommandExt;
                        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
                        cmd.creation_flags(CREATE_NO_WINDOW);
                    }
                    cmd.spawn()
                        .map_err(|e| SynapseError::UpdateInstallFailed(e.to_string()))?;
                    return Ok(());
                }
            }
        }

        Command::new(installer_path)
            .spawn()
            .map_err(|e| SynapseError::UpdateInstallFailed(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_rejects_missing_file() {
        let adapter = WindowsInstallerAdapter::new();
        let missing = Path::new("C:/caminho/inexistente/Synapse-setup.exe");
        let err = adapter.launch_installer(missing).unwrap_err();
        assert!(matches!(err, SynapseError::UpdateInstallFailed(_)));
    }

    #[test]
    fn test_asset_classification_helpers() {
        assert!(WindowsInstallerAdapter::is_msi(Path::new(
            "Synapse_0.2.0_x64.msi"
        )));
        assert!(!WindowsInstallerAdapter::is_portable_standalone_exe(
            Path::new("synapse-0.2.0-Synapse Desktop_0.2.0_x64-setup.exe")
        ));
        assert!(WindowsInstallerAdapter::is_portable_standalone_exe(
            Path::new("synapse-0.2.0-synapse-desktop.exe")
        ));
    }
}
