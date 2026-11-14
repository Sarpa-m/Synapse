# Synapse Desktop

Assistente textual de IA em background para **Windows**: selecione um texto em qualquer app, pressione `Alt + Espaço`, escolha um perfil e o resultado é colado no lugar — sem perder o que estava no seu clipboard.

Arquitetura, regras de negócio e roteiro: veja [AGENTS.md](./AGENTS.md).

## Pré-requisitos (Windows 10/11)

1. Microsoft Visual Studio C++ Build Tools ("Desktop development with C++")
2. WebView2 Runtime (já vem no Windows 10/11)
3. Rust stable MSVC: `rustup default stable-x86_64-pc-windows-msvc`
4. Node.js LTS 20+

## Rodando

```powershell
npm install
npm run tauri dev      # desenvolvimento (popup + tray + atalho global)
npm run tauri build    # instalador .msi / .exe (NSIS) em src-tauri/target/release/bundle
```

Na primeira execução, abra **Bandeja do sistema → Configurações → Provedores e API** e salve sua API Key. Ela fica guardada **somente** no Windows Credential Manager (`com.synapse.desktop`).

## Testes

```bash
cd src-tauri && cargo fmt -- --check && cargo clippy --all-targets -- -D warnings && cargo test
npm run typecheck && npm run test
```

O core (domínio, serviços, SQLite, cliente LLM) compila e é testado em qualquer SO. A camada Tauri (`src-tauri/src/desktop.rs`) e os adaptadores Win32 só são compilados em Windows.

## Limitações conhecidas

- **UIPI:** o Synapse rodando como usuário comum não consegue enviar `Ctrl+C`/`Ctrl+V` para apps abertos como Administrador. Rode o Synapse também como admin se precisar disso.
- **Undo rápido (`Ctrl+Alt+Z`)** fica para a V2 (o histórico já é gravado).
