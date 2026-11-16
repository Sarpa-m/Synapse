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

/// Retângulo físico de um monitor conectado ao sistema (em pixels físicos do SO).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl MonitorBounds {
    /// Verifica se o ponto físico `(px, py)` está contido neste monitor (suporta coordenadas negativas).
    #[must_use]
    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        let right = self
            .x
            .saturating_add(i32::try_from(self.width).unwrap_or(i32::MAX));
        let bottom = self
            .y
            .saturating_add(i32::try_from(self.height).unwrap_or(i32::MAX));
        px >= self.x && px < right && py >= self.y && py < bottom
    }

    /// Calcula a posição `(x, y)` para centralizar uma janela de tamanho físico `(win_w, win_h)` neste monitor.
    #[must_use]
    pub fn center_window(&self, win_w: u32, win_h: u32) -> (i32, i32) {
        let mon_w = i32::try_from(self.width).unwrap_or(1920);
        let mon_h = i32::try_from(self.height).unwrap_or(1080);
        let w = i32::try_from(win_w).unwrap_or(620);
        let h = i32::try_from(win_h).unwrap_or(340);

        let cx = self.x.saturating_add((mon_w - w) / 2);
        let cy = self.y.saturating_add((mon_h - h) / 2);
        (cx, cy)
    }

    /// Distância quadrática mínima de um ponto `(px, py)` até este monitor (0 se estiver dentro).
    #[must_use]
    pub fn distance_sq_to_point(&self, px: i32, py: i32) -> i64 {
        let right = self
            .x
            .saturating_add(i32::try_from(self.width).unwrap_or(0));
        let bottom = self
            .y
            .saturating_add(i32::try_from(self.height).unwrap_or(0));

        let dx = if px < self.x {
            i64::from(self.x) - i64::from(px)
        } else if px >= right {
            i64::from(px) - i64::from(right) + 1
        } else {
            0
        };

        let dy = if py < self.y {
            i64::from(self.y) - i64::from(py)
        } else if py >= bottom {
            i64::from(py) - i64::from(bottom) + 1
        } else {
            0
        };

        dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy))
    }
}

/// Seleciona o monitor que contém o ponto `(px, py)` (ou o monitor mais próximo caso a janela
/// esteja parcialmente fora da área visível) e retorna as coordenadas `(x, y)` centralizadas nele.
#[must_use]
pub fn center_window_on_focused_monitor(
    monitors: &[MonitorBounds],
    focus_point: (i32, i32),
    window_size: (u32, u32),
) -> Option<(i32, i32)> {
    let (px, py) = focus_point;
    let target = monitors
        .iter()
        .find(|m| m.contains_point(px, py))
        .or_else(|| {
            monitors
                .iter()
                .min_by_key(|m| m.distance_sq_to_point(px, py))
        })?;

    Some(target.center_window(window_size.0, window_size.1))
}

/// Porta de abstração para captura e restauração de foco da janela ativa do Windows (`HWND`).
#[cfg_attr(test, mockall::automock)]
pub trait WindowFocusPort: Send + Sync {
    /// Retorna o identificador opaco da janela atualmente em primeiro plano (`HWND` no Windows).
    fn get_active_window(&self) -> SynapseResult<isize>;

    /// Devolve o foco para a janela original identificada por `hwnd`.
    fn restore_focus(&self, hwnd: isize) -> SynapseResult<()>;

    /// Retorna o ponto central `(x, y)` da janela em foco (ou posição do cursor do mouse como fallback)
    /// em coordenadas físicas da área de trabalho virtual (multi-monitor).
    fn get_focus_screen_point(&self, hwnd: Option<isize>) -> SynapseResult<(i32, i32)> {
        let _ = hwnd;
        Ok((0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_monitor_centers_on_secondary_right_and_left_monitors() {
        let primary = MonitorBounds {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let right_monitor = MonitorBounds {
            x: 1920,
            y: 0,
            width: 2560,
            height: 1440,
        };
        let left_monitor = MonitorBounds {
            x: -1920,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let monitors = [primary, right_monitor, left_monitor];

        // Foco no monitor da direita (ex: x = 2500, y = 500)
        let pos_right =
            center_window_on_focused_monitor(&monitors, (2500, 500), (620, 340)).unwrap();
        assert_eq!(pos_right, (1920 + (2560 - 620) / 2, (1440 - 340) / 2));

        // Foco no monitor da esquerda com coordenadas negativas (ex: x = -800, y = 400)
        let pos_left =
            center_window_on_focused_monitor(&monitors, (-800, 400), (620, 340)).unwrap();
        assert_eq!(pos_left, (-1920 + (1920 - 620) / 2, (1080 - 340) / 2));

        // Foco no monitor principal
        let pos_primary =
            center_window_on_focused_monitor(&monitors, (500, 300), (620, 340)).unwrap();
        assert_eq!(pos_primary, ((1920 - 620) / 2, (1080 - 340) / 2));
    }
}
