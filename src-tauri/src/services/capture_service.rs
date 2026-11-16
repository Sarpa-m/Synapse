use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::domain::errors::{SynapseError, SynapseResult};
use crate::ports::os_ports::{ClipboardPort, KeyboardSimulatorPort, WindowFocusPort};

/// Estado capturado no momento em que o usuário aciona o atalho global (`Alt + Espaço`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureSession {
    pub selected_text: String,
    pub source_hwnd: isize,
    pub backup_clipboard: Option<String>,
}

/// Serviço responsável pelo protocolo de Captura Segura e Restauração Imediata do Clipboard (RN-02 e RN-03).
#[derive(Clone)]
pub struct CaptureService {
    clipboard: Arc<dyn ClipboardPort>,
    keyboard: Arc<dyn KeyboardSimulatorPort>,
    window_focus: Arc<dyn WindowFocusPort>,
    poll_interval: Duration,
    max_poll_attempts: usize,
}

impl CaptureService {
    #[must_use]
    pub fn new(
        clipboard: Arc<dyn ClipboardPort>,
        keyboard: Arc<dyn KeyboardSimulatorPort>,
        window_focus: Arc<dyn WindowFocusPort>,
    ) -> Self {
        Self {
            clipboard,
            keyboard,
            window_focus,
            poll_interval: Duration::from_millis(25),
            max_poll_attempts: 6,
        }
    }

    #[must_use]
    pub fn with_polling(mut self, poll_interval: Duration, max_poll_attempts: usize) -> Self {
        self.poll_interval = poll_interval;
        self.max_poll_attempts = max_poll_attempts;
        self
    }

    /// Retorna o ponto central `(x, y)` na tela da janela em foco (ou posição do cursor).
    ///
    /// # Errors
    /// Retorna erro caso o adaptador de janela do SO não consiga obter as coordenadas.
    pub fn get_focus_screen_point(&self, hwnd: Option<isize>) -> SynapseResult<(i32, i32)> {
        self.window_focus.get_focus_screen_point(hwnd)
    }

    /// Executa o fluxo de captura com **Clipboard Guard**:
    /// 1. Captura `HWND` da janela ativa.
    /// 2. Salva backup do clipboard atual.
    /// 3. Limpa o clipboard.
    /// 4. Libera modificadores (`Alt`/`Shift`) e simula `Ctrl+C`.
    /// 5. Faz polling curto até detectar texto válido.
    /// 6. **Restaura imediatamente o `backup_clipboard`** no clipboard do SO antes mesmo de retornar!
    pub async fn capture_selection(&self) -> SynapseResult<CaptureSession> {
        let source_hwnd = self.window_focus.get_active_window()?;
        let backup_clipboard = self.clipboard.get_text().ok().flatten();

        self.clipboard.clear()?;
        self.keyboard.release_modifiers()?;
        self.keyboard.simulate_copy()?;

        let mut captured_text: Option<String> = None;
        let attempts = self.max_poll_attempts.max(1);

        for attempt in 0..attempts {
            if !self.poll_interval.is_zero() && attempt > 0 {
                tokio::time::sleep(self.poll_interval).await;
            }

            if let Ok(Some(text)) = self.clipboard.get_text() {
                if !text.trim().is_empty() {
                    captured_text = Some(text);
                    break;
                }
            }
        }

        // RN-02 (Clipboard Guard): Restaurar o backup do usuário IMEDIATAMENTE após ler a seleção!
        self.restore_backup(&backup_clipboard)?;

        match captured_text {
            Some(selected_text) => {
                tracing::debug!(
                    text_len = selected_text.len(),
                    source_hwnd,
                    "Texto selecionado capturado com sucesso e clipboard original restaurado"
                );
                Ok(CaptureSession {
                    selected_text,
                    source_hwnd,
                    backup_clipboard,
                })
            }
            None => Err(SynapseError::EmptySelection),
        }
    }

    fn restore_backup(&self, backup: &Option<String>) -> SynapseResult<()> {
        match backup {
            Some(original) => self.clipboard.set_text(original),
            None => self.clipboard.clear(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::os_ports::{
        MockClipboardPort, MockKeyboardSimulatorPort, MockWindowFocusPort,
    };
    use mockall::Sequence;

    #[tokio::test]
    async fn test_capture_restores_clipboard_immediately() {
        let mut seq = Sequence::new();
        let mut clip = MockClipboardPort::new();
        let mut kbd = MockKeyboardSimulatorPort::new();
        let mut focus = MockWindowFocusPort::new();

        focus
            .expect_get_active_window()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(4242));

        // 1. Lê o clipboard inicial ("senha_123")
        clip.expect_get_text()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(Some("senha_123".to_string())));

        // 2. Limpa o clipboard antes do Ctrl+C
        clip.expect_clear()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(()));

        // 3. Libera modificadores e envia Ctrl+C
        kbd.expect_release_modifiers()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(()));
        kbd.expect_simulate_copy()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(()));

        // 4. Lê o texto recém-copiado ("ola mundo")
        clip.expect_get_text()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|| Ok(Some("ola mundo".to_string())));

        // 5. Restaura IMEDIATAMENTE "senha_123" no clipboard!
        clip.expect_set_text()
            .withf(|text| text == "senha_123")
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));

        let service = CaptureService::new(Arc::new(clip), Arc::new(kbd), Arc::new(focus))
            .with_polling(Duration::ZERO, 3);

        let session = service.capture_selection().await.unwrap();
        assert_eq!(session.selected_text, "ola mundo");
        assert_eq!(session.source_hwnd, 4242);
        assert_eq!(session.backup_clipboard.as_deref(), Some("senha_123"));
    }

    #[tokio::test]
    async fn test_capture_aborts_on_file_or_image_selection() {
        let mut clip = MockClipboardPort::new();
        let mut kbd = MockKeyboardSimulatorPort::new();
        let mut focus = MockWindowFocusPort::new();

        focus.expect_get_active_window().returning(|| Ok(99));

        // Primeira chamada: backup tinha "meu_backup_importante"
        let mut call_count = 0;
        clip.expect_get_text().returning(move || {
            call_count += 1;
            if call_count == 1 {
                Ok(Some("meu_backup_importante".to_string()))
            } else {
                // Simula arquivo selecionado no Windows Explorer (não é texto)
                Ok(None)
            }
        });

        clip.expect_clear().times(1).returning(|| Ok(()));
        kbd.expect_release_modifiers().times(1).returning(|| Ok(()));
        kbd.expect_simulate_copy().times(1).returning(|| Ok(()));

        // Deve restaurar "meu_backup_importante" mesmo quando aborta!
        clip.expect_set_text()
            .withf(|text| text == "meu_backup_importante")
            .times(1)
            .returning(|_| Ok(()));

        let service = CaptureService::new(Arc::new(clip), Arc::new(kbd), Arc::new(focus))
            .with_polling(Duration::ZERO, 2);

        let err = service.capture_selection().await.unwrap_err();
        assert_eq!(err, SynapseError::EmptySelection);
    }
}
