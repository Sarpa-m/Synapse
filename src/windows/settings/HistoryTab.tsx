import React from "react";
import type { HistoryEntry } from "../../types/domain";

interface HistoryTabProps {
  entries: HistoryEntry[];
  onClearHistory: () => void;
}

export const HistoryTab: React.FC<HistoryTabProps> = ({
  entries,
  onClearHistory,
}) => {
  const handleCopyOriginal = async (text: string) => {
    if (navigator.clipboard) {
      await navigator.clipboard.writeText(text);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold text-white">
          Últimas Operações ({entries.length})
        </h3>
        <button
          type="button"
          onClick={onClearHistory}
          className="px-3 py-1.5 rounded bg-red-900/50 hover:bg-red-800/60 border border-red-800 text-xs text-red-200"
        >
          Limpar Histórico
        </button>
      </div>

      {entries.length === 0 ? (
        <p className="text-xs text-slate-400">Nenhuma operação registrada ainda.</p>
      ) : (
        <div className="overflow-x-auto border border-slate-800 rounded-lg">
          <table className="w-full text-left text-xs text-slate-300">
            <thead className="bg-slate-800/80 text-slate-400 uppercase">
              <tr>
                <th className="px-3 py-2">Data/Hora</th>
                <th className="px-3 py-2">Perfil</th>
                <th className="px-3 py-2">Texto Original</th>
                <th className="px-3 py-2">Texto Gerado</th>
                <th className="px-3 py-2">Ação</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800">
              {entries.map((item) => (
                <tr key={item.id}>
                  <td className="px-3 py-2 whitespace-nowrap">{item.created_at}</td>
                  <td className="px-3 py-2 font-medium text-indigo-300">
                    {item.profile_name}
                  </td>
                  <td className="px-3 py-2 max-w-[180px] truncate">
                    {item.original_text}
                  </td>
                  <td className="px-3 py-2 max-w-[180px] truncate">
                    {item.generated_text}
                  </td>
                  <td className="px-3 py-2">
                    <button
                      type="button"
                      onClick={() => handleCopyOriginal(item.original_text)}
                      className="px-2 py-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-200"
                    >
                      Copiar Original
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};
