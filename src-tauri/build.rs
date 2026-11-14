fn main() {
    // tauri-build gera o contexto (ícones, manifesto Windows, capabilities) apenas no build Windows.
    #[cfg(windows)]
    tauri_build::build();
}
