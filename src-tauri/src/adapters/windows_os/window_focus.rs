use crate::domain::errors::{SynapseError, SynapseResult};
use crate::ports::os_ports::WindowFocusPort;

/// Gerenciador de foco de janela nativo do Windows via Win32 API (`GetForegroundWindow` / `SetForegroundWindow`).
#[derive(Default, Clone, Copy)]
pub struct WindowsWindowFocusAdapter;

impl WindowsWindowFocusAdapter {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl WindowFocusPort for WindowsWindowFocusAdapter {
    fn get_active_window(&self) -> SynapseResult<isize> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
            // SAFETY: `GetForegroundWindow` é uma chamada Win32 pura de leitura que retorna o HWND
            // da janela atual em foreground (ou NULL/0 caso não haja). Não desreferencia ponteiros.
            let hwnd = unsafe { GetForegroundWindow() };
            if hwnd.0.is_null() {
                return Err(SynapseError::WindowFocusError(
                    "nenhuma janela ativa detectada no Windows".into(),
                ));
            }
            Ok(hwnd.0 as isize)
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = SynapseError::WindowFocusError;
            Ok(1)
        }
    }

    fn restore_focus(&self, hwnd: isize) -> SynapseResult<()> {
        if hwnd == 0 {
            return Err(SynapseError::WindowFocusError(
                "HWND inválido (zero)".into(),
            ));
        }

        #[cfg(target_os = "windows")]
        {
            use std::ffi::c_void;
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;

            let target = HWND(hwnd as *mut c_void);
            // SAFETY: `SetForegroundWindow` valida internamente no kernel user32 se o handle `HWND`
            // ainda pertence a uma janela válida antes de trocar o foco.
            let ok = unsafe { SetForegroundWindow(target) };
            if !ok.as_bool() {
                return Err(SynapseError::WindowFocusError(
                    "SetForegroundWindow recusou restaurar foco para a janela de origem".into(),
                ));
            }
            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }

    fn get_focus_screen_point(&self, hwnd: Option<isize>) -> SynapseResult<(i32, i32)> {
        #[cfg(target_os = "windows")]
        {
            use std::ffi::c_void;
            use windows::Win32::Foundation::{HWND, POINT, RECT};
            use windows::Win32::UI::WindowsAndMessaging::{
                GetCursorPos, GetForegroundWindow, GetWindowRect,
            };

            let target_hwnd = match hwnd.filter(|&h| h != 0) {
                Some(raw) => HWND(raw as *mut c_void),
                None => unsafe { GetForegroundWindow() },
            };

            if !target_hwnd.0.is_null() {
                let mut rect = RECT::default();
                // SAFETY: `GetWindowRect` preenche `RECT` com as coordenadas físicas na tela virtual.
                if unsafe { GetWindowRect(target_hwnd, &mut rect) }.is_ok()
                    && rect.right > rect.left
                    && rect.bottom > rect.top
                {
                    let center_x = rect.left.saturating_add(rect.right - rect.left) / 2;
                    let _ = center_x;
                    let cx = rect.left + (rect.right - rect.left) / 2;
                    let cy = rect.top + (rect.bottom - rect.top) / 2;
                    return Ok((cx, cy));
                }
            }

            let mut pt = POINT::default();
            // SAFETY: `GetCursorPos` lê a posição física atual do ponteiro do mouse.
            if unsafe { GetCursorPos(&mut pt) }.is_ok() {
                return Ok((pt.x, pt.y));
            }

            Err(SynapseError::WindowFocusError(
                "não foi possível determinar a posição da janela ativa ou do cursor".into(),
            ))
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = hwnd;
            Ok((0, 0))
        }
    }
}
