use crate::domain::errors::SynapseResult;

/// Porta de abstração para leitura, escrita e limpeza da Área de Transferência do SO.
#[cfg_attr(test, mockall::automock)]
pub trait ClipboardPort: Send + Sync {
    /// Lê o texto atual do clipboard.
    /// Retorna `Ok(None)` caso o clipboard esteja vazio ou contenha dado não-texto (ex: imagem/arquivo).
    fn get_text(&self) -> SynapseResult<Option<String>>;

    /// Define um novo texto UTF-8 no clipboard do sistema.
    fn set_text(&self, text: &str) -> SynapseResult<()>;

    /// Limpa o conteúdo atual do clipboard.
    fn clear(&self) -> SynapseResult<()>;
}

/// Porta de abstração para simulação de atalhos de teclado no SO (Windows SendInput / Enigo).
#[cfg_attr(test, mockall::automock)]
pub trait KeyboardSimulatorPort: Send + Sync {
    /// Garante que teclas modificadoras físicas (Alt, Shift, Win) sejam liberadas antes de enviar Ctrl+C,
    /// prevenindo a colisão `Ctrl + Alt + C` quando acionado via `Alt + Espaço`.
    fn release_modifiers(&self) -> SynapseResult<()>;

    /// Simula o atalho `Ctrl + C` na janela em foco.
    fn simulate_copy(&self) -> SynapseResult<()>;

    /// Simula o atalho `Ctrl + V` na janela em foco.
    fn simulate_paste(&self) -> SynapseResult<()>;
}

/// Porta de abstração para captura e restauração de foco da janela ativa do Windows (`HWND`).
#[cfg_attr(test, mockall::automock)]
pub trait WindowFocusPort: Send + Sync {
    /// Retorna o identificador opaco da janela atualmente em primeiro plano (`HWND` no Windows).
    fn get_active_window(&self) -> SynapseResult<isize>;

    /// Devolve o foco para a janela original identificada por `hwnd`.
    fn restore_focus(&self, hwnd: isize) -> SynapseResult<()>;
}
