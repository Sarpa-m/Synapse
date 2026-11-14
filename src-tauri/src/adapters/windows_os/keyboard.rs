use crate::domain::errors::{SynapseError, SynapseResult};
use crate::ports::os_ports::KeyboardSimulatorPort;

/// Simulador de Teclado Windows (`SendInput` / `enigo`) com proteção contra colisão de teclas modificadoras (`Alt+Espaço`).
#[derive(Default, Clone, Copy)]
pub struct WindowsKeyboardAdapter;

impl WindowsKeyboardAdapter {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl KeyboardSimulatorPort for WindowsKeyboardAdapter {
    fn release_modifiers(&self) -> SynapseResult<()> {
        #[cfg(target_os = "windows")]
        {
            use enigo::{Direction, Enigo, Key, Keyboard, Settings};
            let mut enigo = Enigo::new(&Settings::default())
                .map_err(|e| SynapseError::KeyboardSimulationError(e.to_string()))?;

            // Libera explicitamente Alt, Shift, Control e Meta (Win) antes de enviar Ctrl+C
            for key in [Key::Alt, Key::Shift, Key::Control, Key::Meta] {
                let _ = enigo.key(key, Direction::Release);
            }
            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = SynapseError::KeyboardSimulationError;
            Ok(())
        }
    }

    fn simulate_copy(&self) -> SynapseResult<()> {
        #[cfg(target_os = "windows")]
        {
            use enigo::{Direction, Enigo, Key, Keyboard, Settings};
            let mut enigo = Enigo::new(&Settings::default())
                .map_err(|e| SynapseError::KeyboardSimulationError(e.to_string()))?;

            enigo
                .key(Key::Control, Direction::Press)
                .map_err(|e| SynapseError::KeyboardSimulationError(e.to_string()))?;
            enigo
                .key(Key::Unicode('c'), Direction::Click)
                .map_err(|e| SynapseError::KeyboardSimulationError(e.to_string()))?;
            enigo
                .key(Key::Control, Direction::Release)
                .map_err(|e| SynapseError::KeyboardSimulationError(e.to_string()))?;
            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }

    fn simulate_paste(&self) -> SynapseResult<()> {
        #[cfg(target_os = "windows")]
        {
            use enigo::{Direction, Enigo, Key, Keyboard, Settings};
            let mut enigo = Enigo::new(&Settings::default())
                .map_err(|e| SynapseError::KeyboardSimulationError(e.to_string()))?;

            enigo
                .key(Key::Control, Direction::Press)
                .map_err(|e| SynapseError::KeyboardSimulationError(e.to_string()))?;
            enigo
                .key(Key::Unicode('v'), Direction::Click)
                .map_err(|e| SynapseError::KeyboardSimulationError(e.to_string()))?;
            enigo
                .key(Key::Control, Direction::Release)
                .map_err(|e| SynapseError::KeyboardSimulationError(e.to_string()))?;
            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }
}
