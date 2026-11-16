import React, { useState } from "react";
import { Keyboard } from "lucide-react";
import { KbdBadge } from "../../components/ui/KbdBadge";

interface ShortcutRecorderProps {
  label: string;
  value: string;
  onChange: (shortcut: string) => void;
}

const MODIFIER_KEYS = new Set([
  "Control",
  "Alt",
  "Shift",
  "Meta",
  "AltGraph",
  "OS",
]);

function normalizeMainKey(e: React.KeyboardEvent): string | null {
  if (MODIFIER_KEYS.has(e.key)) return null;

  const { code, key } = e;
  if (code === "Space" || key === " ") return "Space";
  if (code.startsWith("Key") && code.length === 4) return code.slice(3).toUpperCase();
  if (code.startsWith("Digit") && code.length === 6) return code.slice(5);
  if (/^F([1-9]|1[0-2])$/.test(code)) return code;
  if (code.startsWith("Arrow")) return code.slice(5);

  const directMap: Record<string, string> = {
    Enter: "Enter",
    Tab: "Tab",
    Backspace: "Backspace",
    Delete: "Delete",
    Insert: "Insert",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
  };
  if (code in directMap) return directMap[code];
  if (key.length === 1) return key.toUpperCase();
  return null;
}

export const ShortcutRecorder: React.FC<ShortcutRecorderProps> = ({
  label,
  value,
  onChange,
}) => {
  const [isRecording, setIsRecording] = useState(false);
  const [hint, setHint] = useState<string | null>(null);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
    if (!isRecording) return;
    e.preventDefault();
    e.stopPropagation();

    if (e.key === "Escape") {
      setIsRecording(false);
      setHint(null);
      return;
    }

    const mainKey = normalizeMainKey(e);
    if (!mainKey) return;

    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.metaKey) parts.push("Super");

    if (parts.length === 0) {
      setHint("Use ao menos um modificador (Ctrl, Alt ou Shift).");
      return;
    }

    parts.push(mainKey);
    onChange(parts.join("+"));
    setIsRecording(false);
    setHint(null);
  };

  return (
    <div className="space-y-1.5">
      <label className="block text-xs font-medium text-zinc-300">{label}</label>
      <div className="flex items-center gap-2">
        <button
          type="button"
          aria-label={label}
          onClick={() => {
            setIsRecording(true);
            setHint(null);
          }}
          onBlur={() => {
            setIsRecording(false);
            setHint(null);
          }}
          onKeyDown={handleKeyDown}
          className={`flex-1 flex items-center justify-between gap-2 text-left px-3 py-2 rounded-lg border text-sm synapse-interactive focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-400/50 ${
            isRecording
              ? "bg-sky-950/50 border-sky-400/70 text-sky-200"
              : "bg-[#0B0D10] border-white/10 text-zinc-100 hover:border-white/20"
          }`}
        >
          {isRecording ? (
            <span className="inline-flex items-center gap-2 text-xs text-sky-300">
              <span className="w-2 h-2 rounded-full bg-sky-400 animate-pulse" />
              Pressione a combinação de teclas... (Esc cancela)
            </span>
          ) : value ? (
            <KbdBadge keys={value} size="sm" />
          ) : (
            <span className="text-xs text-zinc-500">Clique para definir um atalho</span>
          )}
          <Keyboard className="w-3.5 h-3.5 text-zinc-500 shrink-0" />
        </button>

        <button
          type="button"
          onClick={() => {
            setIsRecording((prev) => !prev);
            setHint(null);
          }}
          className="px-3 py-2 rounded-lg bg-white/[0.05] hover:bg-white/[0.09] border border-white/10 text-xs font-medium text-zinc-200 synapse-pressable"
        >
          {isRecording ? "Cancelar" : "Gravar"}
        </button>
      </div>
      {hint && <p className="text-xs text-amber-400">{hint}</p>}
    </div>
  );
};
