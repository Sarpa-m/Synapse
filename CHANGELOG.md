# Changelog — Synapse Desktop

Todas as alterações notáveis, novas funcionalidades e correções de bugs de cada versão do **Synapse Desktop** são documentadas neste arquivo e exibidas automaticamente na janela de atualização do aplicativo.

---

## [0.2.2] - 2026-11-16

### Adicionado
- **Suporte Inteligente a Múltiplos Monitores:** Ao acionar o atalho do Popup (Seleção ou Composição), bem como ao abrir Configurações ou Atualizações, o Synapse agora identifica em qual monitor está a janela ativa em foco (`GetWindowRect` no `HWND` de origem, com fallback para a posição do cursor `GetCursorPos`) e centraliza a janela diretamente no monitor em uso, respeitando coordenadas negativas e escalas de DPI diferentes entre telas.

---

## [0.2.1] - 2026-11-16

### Corrigido
- **Tela de Configurações travada em novas instalações:** Corrigida condição de corrida no boot onde a janela oculta `settings` consultava o backend alguns milissegundos antes da criação inicial do banco `synapse.db` em máquinas recém-instaladas.
- **Recarregamento automático ao abrir Configurações:** A janela de Configurações agora executa *retry* automático com backoff durante o boot e recarrega os dados sempre que é aberta pelo System Tray (`synapse://settings-opened`).
- **Tolerância a falhas do Windows Credential Manager (`keyring`):** Caso o cofre de credenciais do Windows retorne erro de plataforma ao verificar `has_api_key` em uma conta nova, o sistema agora registra um aviso no log e carrega a interface normalmente assumindo `has_api_key = false`.

### Adicionado
- **Arquivo de Log em Disco (`synapse.log`):** O aplicativo agora grava automaticamente logs de inicialização, avisos e erros em `%APPDATA%\com.synapse.desktop\synapse.log` para diagnóstico rápido em qualquer máquina.
- **Painel de Diagnóstico na UI:** Caso ocorra qualquer falha persistente ao carregar as configurações, a janela exibe o erro detalhado com botão **"Tentar novamente"**.

---

## [0.2.0] - 2026-11-16

### Adicionado
- **Sistema de Atualização Automática (GitHub Releases):**
  - Verificação assíncrona e silenciosa na inicialização do app comparando a versão local com a última release publicada no GitHub (`Sarpa-m/Synapse`).
  - **Janela dedicada de permissão (`UpdateWindow`):** Solicita consentimento explícito ao usuário antes de atualizar, exibindo comparativo de versões (`vAtual → vNova`), tamanho do pacote, notas de lançamento (*Release Notes*) e barra de progresso em tempo real durante o download.
  - Opções para **"Permitir e Atualizar"**, **"Mais tarde"** ou **"Ignorar esta versão"**.
  - Atalho **"Verificar atualizações..."** no menu do System Tray e seção de controle na aba **Configurações → Geral**.
- **Modos de Entrega do Texto Selecionado:** Suporte a três modos configuráveis (*Salvar no Clipboard*, *Colar e manter no Ctrl+V*, *Colar e restaurar clipboard antigo*).
- **Novo Perfil Padrão:** Adicionado o perfil *"Resumo Técnico e Arquitetura"* aos perfis padrão do sistema.

### Alterado
- **Design System da Interface:** Reestruturação visual do Popup Spotlight e do Painel de Configurações com barra lateral fixa, indicadores de status do cofre de API Key e animações fluidas.

---

## [0.1.0] - 2026-11-14

### Adicionado
- **Lançamento Inicial (MVP) do Synapse Desktop para Windows:**
  - Captura segura de texto selecionado via atalho global (`Alt+Space`) com preservação do clipboard (*Clipboard Guard*).
  - Modo de Composição (`Ctrl+Alt+Space`) para redigir instruções livres e copiar o resultado.
  - Integração HTTP compatível com OpenAI / DeepSeek / Groq / Ollama e cofre seguro no Windows Credential Manager (`keyring`).
  - Banco local SQLite (`rusqlite` bundled) com gerenciamento de perfis, histórico e retenção automática.
