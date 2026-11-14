import React, { useEffect, useMemo, useRef, useState } from "react";
import { AlertCircle, ClipboardCheck } from "lucide-react";
import type { PopupSessionView, Profile } from "../../types/domain";
import { tauriClient } from "../../services/tauriClient";
import { PromptInput } from "./PromptInput";
import { ProfileList } from "./ProfileList";

function extractErrorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "object" && err !== null && "message" in err) {
    return String((err as { message: unknown }).message);
  }
  return String(err);
}

export const PopupWindow: React.FC = () => {
  const inputRef = useRef<HTMLInputElement>(null);
  const composeRef = useRef<HTMLTextAreaElement>(null);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [session, setSession] = useState<PopupSessionView | null>(null);
  const [query, setQuery] = useState("");
  const [composeText, setComposeText] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const isComposeMode = session?.mode === "compose";

  useEffect(() => {
    let active = true;
    Promise.all([tauriClient.listProfiles(), tauriClient.getPopupSession()])
      .then(([loadedProfiles, currentSession]) => {
        if (!active) return;
        setProfiles(loadedProfiles);
        setSession(currentSession);
        if (currentSession?.mode === "compose") {
          composeRef.current?.focus();
        } else {
          inputRef.current?.focus();
        }
      })
      .catch((err: unknown) => {
        if (!active) return;
        setErrorMsg(extractErrorMessage(err));
      });

    // Popup é pré-carregado e reutilizado: a cada atalho, reseta o estado e recarrega perfis.
    const unlistenPromise = tauriClient.onSessionStarted(async (newSession) => {
      setSession(newSession);
      setQuery("");
      setComposeText("");
      setSelectedIndex(0);
      setErrorMsg(null);
      setIsLoading(false);
      setProfiles(await tauriClient.listProfiles());
      setTimeout(() => {
        if (newSession.mode === "compose") {
          composeRef.current?.focus();
        } else {
          inputRef.current?.focus();
        }
      }, 0);
    });

    return () => {
      active = false;
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // Perder o foco fecha o popup instantaneamente (exceto durante o carregamento da IA).
  const isLoadingRef = useRef(false);
  isLoadingRef.current = isLoading;
  useEffect(() => {
    const onBlur = () => {
      if (!isLoadingRef.current) {
        void tauriClient.closePopup();
      }
    };
    window.addEventListener("blur", onBlur);
    return () => window.removeEventListener("blur", onBlur);
  }, []);

  const filteredProfiles = useMemo(() => {
    if (isComposeMode) return profiles;
    const q = query.trim().toLowerCase();
    if (!q) return profiles;
    return profiles.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        (p.shortcut && p.shortcut.toLowerCase() === q)
    );
  }, [profiles, query, isComposeMode]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query, session?.mode]);

  const handleExecuteProfile = async (profile: Profile) => {
    if (isLoading) return;

    if (isComposeMode) {
      const trimmed = composeText.trim();
      if (!trimmed) {
        setErrorMsg("Digite um texto antes de enviar para o perfil.");
        composeRef.current?.focus();
        return;
      }
      setIsLoading(true);
      setErrorMsg(null);
      try {
        await tauriClient.executeComposeProfile(profile.id, composeText);
        setComposeText("");
        await tauriClient.closePopup();
      } catch (err: unknown) {
        setErrorMsg(extractErrorMessage(err));
      } finally {
        setIsLoading(false);
      }
      return;
    }

    setIsLoading(true);
    setErrorMsg(null);
    try {
      await tauriClient.executeProfile(profile.id);
      setQuery("");
      await tauriClient.closePopup();
    } catch (err: unknown) {
      setErrorMsg(extractErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleExecuteCustom = async (instruction: string) => {
    if (isLoading) return;
    setIsLoading(true);
    setErrorMsg(null);
    try {
      await tauriClient.executeCustomPrompt(instruction);
      setQuery("");
      await tauriClient.closePopup();
    } catch (err: unknown) {
      setErrorMsg(extractErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyDown = async (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Escape") {
      e.preventDefault();
      setErrorMsg(null);
      await tauriClient.closePopup();
      return;
    }

    if (isLoading) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (filteredProfiles.length > 0) {
        setSelectedIndex((prev) => (prev + 1) % filteredProfiles.length);
      }
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      if (filteredProfiles.length > 0) {
        setSelectedIndex(
          (prev) => (prev - 1 + filteredProfiles.length) % filteredProfiles.length
        );
      }
      return;
    }

    if (isComposeMode) {
      if (e.ctrlKey && /^[1-9]$/.test(e.key)) {
        const matched =
          profiles.find((p) => p.shortcut === e.key) ??
          profiles[Number(e.key) - 1];
        if (matched) {
          e.preventDefault();
          await handleExecuteProfile(matched);
          return;
        }
      }

      if (e.ctrlKey && e.key === "Enter") {
        e.preventDefault();
        const targetProfile = filteredProfiles[selectedIndex];
        if (targetProfile) {
          await handleExecuteProfile(targetProfile);
        }
      }
      return;
    }

    // Atalho numérico imediato quando o campo de busca está vazio (1, 2, 3...)
    if (query.trim() === "" && /^[1-9]$/.test(e.key)) {
      const matched =
        profiles.find((p) => p.shortcut === e.key) ??
        profiles[Number(e.key) - 1];
      if (matched) {
        e.preventDefault();
        await handleExecuteProfile(matched);
        return;
      }
    }

    if (e.key === "Enter") {
      e.preventDefault();
      const targetProfile = filteredProfiles[selectedIndex];
      if (targetProfile) {
        await handleExecuteProfile(targetProfile);
      } else if (query.trim().length > 0) {
        await handleExecuteCustom(query.trim());
      }
    }
  };

  return (
    <div
      onKeyDown={handleKeyDown}
      tabIndex={-1}
      className="w-full max-w-xl mx-auto rounded-xl border border-slate-700/80 bg-slate-900/95 shadow-2xl overflow-hidden text-slate-100"
    >
      {isComposeMode ? (
        <div className="p-3 border-b border-slate-800 space-y-2">
          <div className="flex items-center justify-between text-[11px] text-indigo-300">
            <span className="flex items-center gap-1.5 font-medium">
              <ClipboardCheck className="w-3.5 h-3.5" />
              Modo Composição — resultado será copiado para seu Ctrl+V
            </span>
            {isLoading && <span className="text-slate-400">Processando...</span>}
          </div>
          <textarea
            ref={composeRef}
            rows={3}
            disabled={isLoading}
            aria-label="Texto para processar"
            placeholder="Digite seu texto aqui e escolha um perfil abaixo (Ctrl+Enter ou clique)..."
            value={composeText}
            onChange={(e) => setComposeText(e.target.value)}
            className="w-full resize-none rounded-lg bg-slate-950/70 border border-slate-700/80 px-3 py-2 text-sm text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
          />
        </div>
      ) : (
        <PromptInput
          inputRef={inputRef}
          value={query}
          isLoading={isLoading}
          onChange={setQuery}
        />
      )}

      {errorMsg && (
        <div
          role="alert"
          className="flex items-center gap-2 px-4 py-2.5 bg-red-950/90 border-b border-red-800 text-red-200 text-xs"
        >
          <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />
          <span className="flex-1">{errorMsg}</span>
          <span className="text-[11px] text-red-300/80">
            {isComposeMode ? "Ctrl+Enter p/ repetir · Esc p/ sair" : "Enter p/ repetir · Esc p/ sair"}
          </span>
        </div>
      )}

      <ProfileList
        profiles={filteredProfiles}
        selectedIndex={selectedIndex}
        disabled={isLoading}
        onSelect={handleExecuteProfile}
        onHoverIndex={setSelectedIndex}
      />

      <div className="flex items-center justify-between px-4 py-2 bg-slate-950/80 border-t border-slate-800/80 text-[11px] text-slate-400">
        <span className="truncate max-w-[360px]">
          {isComposeMode
            ? `Digitado: ${composeText.length} chars`
            : session
              ? `Selecionado (${session.char_count} chars): "${session.selected_text}"`
              : "Aguardando seleção..."}
        </span>
        <span>
          {isComposeMode
            ? "↑↓ navegar · Ctrl+Enter/Ctrl+1..9 copiar · Esc fechar"
            : "↑↓ navegar · Enter aplicar · Esc fechar"}
        </span>
      </div>
    </div>
  );
};
