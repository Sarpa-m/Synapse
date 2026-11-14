#[cfg(not(target_os = "windows"))]
use std::sync::{Arc, Mutex};

use crate::domain::errors::{SynapseError, SynapseResult};
use crate::ports::os_ports::ClipboardPort;

/// Adaptador de Clipboard do Sistema Operacional (`arboard` no Windows, fallback seguro em memória).
#[derive(Clone, Default)]
pub struct SystemClipboardAdapter {
    #[cfg(not(target_os = "windows"))]
    memory_clipboard: Arc<Mutex<Option<String>>>,
}

impl SystemClipboardAdapter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_os = "windows"))]
            memory_clipboard: Arc::new(Mutex::new(None)),
        }
    }
}

impl ClipboardPort for SystemClipboardAdapter {
    fn get_text(&self) -> SynapseResult<Option<String>> {
        #[cfg(target_os = "windows")]
        {
            let mut board = arboard::Clipboard::new()
                .map_err(|e| SynapseError::ClipboardError(e.to_string()))?;
            match board.get_text() {
                Ok(text) if !text.is_empty() => Ok(Some(text)),
                // RN-03: Quando o usuário seleciona um arquivo no Explorer ou imagem,
                // `get_text` retorna `ContentNotAvailable`. Tratamos como `Ok(None)` para abortar sem erro fatal.
                Ok(_) | Err(arboard::Error::ContentNotAvailable) => Ok(None),
                Err(e) => Err(SynapseError::ClipboardError(e.to_string())),
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let guard = self
                .memory_clipboard
                .lock()
                .map_err(|_| SynapseError::ClipboardError("lock do clipboard corrompido".into()))?;
            Ok(guard.clone())
        }
    }

    fn set_text(&self, text: &str) -> SynapseResult<()> {
        #[cfg(target_os = "windows")]
        {
            let mut board = arboard::Clipboard::new()
                .map_err(|e| SynapseError::ClipboardError(e.to_string()))?;
            board
                .set_text(text.to_owned())
                .map_err(|e| SynapseError::ClipboardError(e.to_string()))
        }

        #[cfg(not(target_os = "windows"))]
        {
            let mut guard = self
                .memory_clipboard
                .lock()
                .map_err(|_| SynapseError::ClipboardError("lock do clipboard corrompido".into()))?;
            *guard = Some(text.to_string());
            Ok(())
        }
    }

    fn clear(&self) -> SynapseResult<()> {
        #[cfg(target_os = "windows")]
        {
            let mut board = arboard::Clipboard::new()
                .map_err(|e| SynapseError::ClipboardError(e.to_string()))?;
            board
                .clear()
                .map_err(|e| SynapseError::ClipboardError(e.to_string()))
        }

        #[cfg(not(target_os = "windows"))]
        {
            let mut guard = self
                .memory_clipboard
                .lock()
                .map_err(|_| SynapseError::ClipboardError("lock do clipboard corrompido".into()))?;
            *guard = None;
            Ok(())
        }
    }
}
