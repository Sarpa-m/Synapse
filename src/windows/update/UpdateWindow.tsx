import React, { useEffect, useState } from "react";
import {
  ArrowRight,
  Download,
  RefreshCw,
  ShieldCheck,
  Sparkles,
  XCircle,
} from "lucide-react";
import type { UpdateProgress, UpdateReleaseInfo } from "../../types/domain";
import { tauriClient } from "../../services/tauriClient";

function formatMegabytes(bytes: number): string {
  if (!bytes || bytes <= 0) return "—";
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function extractErrorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "object" && err !== null && "message" in err) {
    return String((err as { message: unknown }).message);
  }
  return String(err);
}

export const UpdateWindow: React.FC = () => {
  const [currentVersion, setCurrentVersion] = useState<string>("0.1.0");
  const [release, setRelease] = useState<UpdateReleaseInfo | null>(null);
  const [isDownloading, setIsDownloading] = useState<boolean>(false);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let unlistenAvailable: (() => void) | undefined;
    let unlistenProgress: (() => void) | undefined;

    Promise.all([
      tauriClient.getAppVersion(),
      tauriClient.getPendingUpdate(),
    ]).then(([ver, pending]) => {
      setCurrentVersion(ver);
      if (pending) {
        setRelease(pending);
      }
    });

    tauriClient
      .onUpdateAvailable((incoming) => {
        setRelease(incoming);
        setIsDownloading(false);
        setProgress(null);
        setError(null);
      })
      .then((fn) => {
        unlistenAvailable = fn;
      });

    tauriClient
      .onUpdateProgress((prog) => {
        setProgress(prog);
      })
      .then((fn) => {
        unlistenProgress = fn;
      });

    return () => {
      if (unlistenAvailable) unlistenAvailable();
      if (unlistenProgress) unlistenProgress();
    };
  }, []);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !isDownloading) {
        void tauriClient.dismissUpdatePrompt();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [isDownloading]);

  const handleConfirmUpdate = async () => {
    if (!release || isDownloading) return;
    setIsDownloading(true);
    setError(null);
    setProgress({
      downloaded_bytes: 0,
      total_bytes: release.asset_size_bytes,
      percentage: 0,
    });

    try {
      await tauriClient.installUpdate(release);
    } catch (err: unknown) {
      setError(extractErrorMessage(err));
      setIsDownloading(false);
    }
  };

  const handleDismiss = async () => {
    if (isDownloading) return;
    await tauriClient.dismissUpdatePrompt();
  };

  const handleSkipVersion = async () => {
    if (!release || isDownloading) return;
    await tauriClient.skipUpdateVersion(release.version);
  };

  if (!release) {
    return (
      <div className="h-screen bg-[#0B0D10] text-zinc-200 flex flex-col items-center justify-center p-6 text-center">
        <RefreshCw className="w-6 h-6 text-sky-400 animate-spin mb-3" />
        <p className="text-sm text-zinc-400">
          Verificando informações da versão no GitHub...
        </p>
      </div>
    );
  }

  return (
    <div className="h-screen bg-[#0B0D10] text-zinc-100 flex flex-col justify-between p-6 select-none overflow-hidden">
      {/* Cabeçalho e Resumo da Versão */}
      <div className="space-y-4 min-h-0 flex-1 flex flex-col">
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-sky-500/15 border border-sky-400/30 flex items-center justify-center shrink-0">
              <Sparkles className="w-5 h-5 text-sky-400" />
            </div>
            <div>
              <h1 className="text-base font-semibold text-white">
                Nova versão do Synapse disponível
              </h1>
              <p className="text-xs text-zinc-400 mt-0.5">
                Deseja permitir a atualização automática a partir do GitHub?
              </p>
            </div>
          </div>
        </div>

        {/* Comparativo de Versões */}
        <div className="flex items-center justify-between px-4 py-3 rounded-xl bg-[#12151B] border border-white/[0.08]">
          <div className="flex items-center gap-2.5 text-xs">
            <span className="px-2 py-0.5 rounded bg-zinc-800 text-zinc-300 font-mono">
              v{currentVersion}
            </span>
            <ArrowRight className="w-3.5 h-3.5 text-zinc-500" />
            <span className="px-2 py-0.5 rounded bg-emerald-500/20 border border-emerald-400/30 text-emerald-300 font-mono font-semibold">
              v{release.version}
            </span>
          </div>
          <div className="flex items-center gap-1.5 text-[11px] text-zinc-400">
            <ShieldCheck className="w-3.5 h-3.5 text-sky-400" />
            <span>{formatMegabytes(release.asset_size_bytes)}</span>
          </div>
        </div>

        {/* Notas de Lançamento */}
        <div className="flex-1 min-h-0 flex flex-col">
          <span className="text-[11px] font-medium uppercase tracking-wider text-zinc-400 mb-1.5">
            Notas da versão ({release.tag_name})
          </span>
          <div className="flex-1 overflow-y-auto rounded-lg bg-[#12151B] border border-white/[0.06] p-3 text-xs text-zinc-300 whitespace-pre-wrap select-text leading-relaxed">
            {release.release_notes ||
              "Melhorias de desempenho, estabilidade e correções gerais."}
          </div>
        </div>

        {/* Barra de Progresso Transparente */}
        {isDownloading && progress && (
          <div
            role="status"
            aria-label="Progresso da atualização"
            className="space-y-1.5 px-3 py-2.5 rounded-lg bg-sky-950/30 border border-sky-500/30"
          >
            <div className="flex items-center justify-between text-xs text-sky-200">
              <span>Baixando e preparando instalador...</span>
              <span className="font-mono">{progress.percentage}%</span>
            </div>
            <div className="w-full h-2 rounded-full bg-slate-800 overflow-hidden">
              <div
                className="h-full bg-sky-400 transition-all duration-150"
                style={{ width: `${progress.percentage}%` }}
              />
            </div>
            <div className="text-[11px] text-zinc-400 text-right font-mono">
              {formatMegabytes(progress.downloaded_bytes)} /{" "}
              {formatMegabytes(progress.total_bytes)}
            </div>
          </div>
        )}

        {/* Erro amigável */}
        {error && (
          <div
            role="alert"
            className="flex items-center gap-2 px-3 py-2 rounded-lg bg-red-950/50 border border-red-500/40 text-xs text-red-200"
          >
            <XCircle className="w-4 h-4 text-red-400 shrink-0" />
            <span className="truncate">{error}</span>
          </div>
        )}
      </div>

      {/* Rodapé com Ações de Consentimento */}
      <div className="pt-4 mt-3 border-t border-white/[0.08] flex items-center justify-between gap-2">
        <button
          type="button"
          disabled={isDownloading}
          onClick={handleSkipVersion}
          className="px-3 py-2 rounded-lg text-xs text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.04] disabled:opacity-40"
        >
          Ignorar esta versão
        </button>

        <div className="flex items-center gap-2">
          <button
            type="button"
            disabled={isDownloading}
            onClick={handleDismiss}
            className="px-3.5 py-2 rounded-lg text-xs font-medium bg-zinc-800 hover:bg-zinc-700 text-zinc-200 disabled:opacity-40"
          >
            Mais tarde
          </button>

          <button
            type="button"
            disabled={isDownloading}
            onClick={handleConfirmUpdate}
            className="flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-semibold bg-sky-600 hover:bg-sky-500 text-white disabled:opacity-50"
          >
            <Download className="w-3.5 h-3.5" />
            {isDownloading ? "Atualizando..." : "Permitir e Atualizar"}
          </button>
        </div>
      </div>
    </div>
  );
};
