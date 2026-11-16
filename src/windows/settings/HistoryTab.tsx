import React, { useMemo, useState } from "react";
import { Check, ChevronDown, Copy, Search, Trash2 } from "lucide-react";
import type { HistoryEntry } from "../../types/domain";

interface HistoryTabProps {
  entries: HistoryEntry[];
  onClearHistory: () => void;
}

export const HistoryTab: React.FC<HistoryTabProps> = ({
  entries,
  onClearHistory,
}) => {
  const [search, setSearch] = useState("");
  const [expandedId, setExpandedId] = useState<number | null>(null);
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [confirmClear, setConfirmClear] = useState(false);

  const filteredEntries = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return entries;
    return entries.filter(
      (item) =>
        item.profile_name.toLowerCase().includes(q) ||
        item.original_text.toLowerCase().includes(q) ||
        item.generated_text.toLowerCase().includes(q)
    );
  }, [entries, search]);

  const handleCopy = async (key: string, text: string) => {
    if (navigator.clipboard) {
      await navigator.clipboard.writeText(text);
    }
    setCopiedKey(key);
    setTimeout(() => {
      setCopiedKey((current) => (current === key ? null : current));
    }, 1500);
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-3">
        <div className="relative flex-1">
          <Search className="w-4 h-4 text-zinc-500 absolute left-3 top-1/2 -translate-y-1/2" />
          <input
            type="text"
            aria-label="Filtrar histórico"
            placeholder="Buscar por perfil ou trecho de texto..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-9 pr-3 py-2 rounded-lg bg-[#12151B] border border-white/[0.08] text-xs text-white placeholder-zinc-500 synapse-interactive focus:outline-none focus:border-sky-400/60"
          />
        </div>

        {entries.length > 0 && (
          <div className="flex items-center gap-2 shrink-0">
            {confirmClear ? (
              <>
                <button
                  type="button"
                  onClick={() => {
                    onClearHistory();
                    setConfirmClear(false);
                  }}
                  className="px-3 py-2 rounded-lg bg-rose-600 hover:bg-rose-500 text-xs font-semibold text-white synapse-pressable"
                >
                  Confirmar limpeza
                </button>
                <button
                  type="button"
                  onClick={() => setConfirmClear(false)}
                  className="px-2.5 py-2 rounded-lg bg-white/[0.06] hover:bg-white/[0.1] text-xs text-zinc-300 synapse-pressable"
                >
                  Cancelar
                </button>
              </>
            ) : (
              <button
                type="button"
                onClick={() => setConfirmClear(true)}
                className="inline-flex items-center gap-1.5 px-3 py-2 rounded-lg bg-rose-950/50 hover:bg-rose-900/60 border border-rose-500/25 text-xs text-rose-200 synapse-pressable"
              >
                <Trash2 className="w-3.5 h-3.5" />
                Limpar Histórico
              </button>
            )}
          </div>
        )}
      </div>

      {filteredEntries.length === 0 ? (
        <div className="rounded-xl border border-white/[0.08] bg-[#12151B] p-8 text-center">
          <p className="text-xs text-zinc-400">
            {entries.length === 0
              ? "Nenhuma operação registrada ainda."
              : "Nenhum registro encontrado para essa busca."}
          </p>
        </div>
      ) : (
        <div className="space-y-2.5">
          {filteredEntries.map((item) => {
            const isExpanded = expandedId === item.id;
            const origKey = `orig-${item.id}`;
            const genKey = `gen-${item.id}`;

            return (
              <article
                key={item.id}
                className="synapse-card-hover rounded-xl border border-white/[0.08] bg-[#12151B] p-3.5 space-y-2.5"
              >
                <div className="flex items-center justify-between gap-2">
                  <div className="flex items-center gap-2 min-w-0">
                    <span className="px-2 py-0.5 rounded bg-sky-500/15 text-sky-300 text-[11px] font-medium truncate">
                      {item.profile_name}
                    </span>
                    <span className="text-[11px] font-mono text-zinc-500">
                      {item.created_at}
                    </span>
                  </div>

                  <button
                    type="button"
                    onClick={() =>
                      setExpandedId(isExpanded ? null : item.id)
                    }
                    className="inline-flex items-center gap-1 text-[11px] text-zinc-400 hover:text-zinc-200 synapse-interactive"
                  >
                    <span>{isExpanded ? "Recolher" : "Expandir"}</span>
                    <ChevronDown
                      className="w-3.5 h-3.5"
                      style={{
                        transform: isExpanded ? "rotate(180deg)" : "rotate(0deg)",
                        transition: "transform 160ms var(--ease-out)",
                      }}
                    />
                  </button>
                </div>

                <div className="grid grid-cols-2 gap-3 text-xs">
                  <div className="rounded-lg bg-[#0B0D10] border border-white/[0.06] p-2.5 space-y-1.5">
                    <div className="flex items-center justify-between text-[11px] text-zinc-400">
                      <span>Original</span>
                      <button
                        type="button"
                        onClick={() => handleCopy(origKey, item.original_text)}
                        className="inline-flex items-center gap-1 text-zinc-300 hover:text-sky-300 synapse-interactive"
                      >
                        {copiedKey === origKey ? (
                          <>
                            <Check className="w-3 h-3 text-emerald-400" />
                            <span className="text-emerald-300">Copiado!</span>
                          </>
                        ) : (
                          <>
                            <Copy className="w-3 h-3" />
                            <span>Copiar Original</span>
                          </>
                        )}
                      </button>
                    </div>
                    <p
                      className={`text-zinc-300 leading-relaxed whitespace-pre-wrap ${
                        isExpanded ? "" : "line-clamp-2"
                      }`}
                    >
                      {item.original_text}
                    </p>
                  </div>

                  <div className="rounded-lg bg-[#0B0D10] border border-white/[0.06] p-2.5 space-y-1.5">
                    <div className="flex items-center justify-between text-[11px] text-zinc-400">
                      <span>Gerado</span>
                      <button
                        type="button"
                        onClick={() => handleCopy(genKey, item.generated_text)}
                        className="inline-flex items-center gap-1 text-zinc-300 hover:text-sky-300 synapse-interactive"
                      >
                        {copiedKey === genKey ? (
                          <>
                            <Check className="w-3 h-3 text-emerald-400" />
                            <span className="text-emerald-300">Copiado!</span>
                          </>
                        ) : (
                          <>
                            <Copy className="w-3 h-3" />
                            <span>Copiar Gerado</span>
                          </>
                        )}
                      </button>
                    </div>
                    <p
                      className={`text-zinc-100 leading-relaxed whitespace-pre-wrap ${
                        isExpanded ? "" : "line-clamp-2"
                      }`}
                    >
                      {item.generated_text}
                    </p>
                  </div>
                </div>
              </article>
            );
          })}
        </div>
      )}
    </div>
  );
};
