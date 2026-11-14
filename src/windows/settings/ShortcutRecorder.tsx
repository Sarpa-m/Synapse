import React, { useState } from "react";

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
    <div>
      <label className="block text-xs font-medium text-slate-300 mb-1">
        {label}
      </label>
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
          className={`flex-1 text-left px-3 py-2 rounded border text-sm transition-colors ${
            isRecording
              ? "bg-indigo-950/60 border-indigo-400 text-indigo-200"
              : "bg-slate-800 border-slate-700 text-white hover:border-slate-500"
          }`}
        >
          {isRecording
            ? "Pressione a combinação de teclas... (Esc cancela)"
            : value || "Clique para definir um atalho"}
        </button>
        <button
          type="button"
          onClick={() => {
            setIsRecording((prev) => !prev);
            setHint(null);
          }}
          className="px-3 py-2 rounded bg-slate-800 hover:bg-slate-700 border border-slate-700 text-xs text-slate-200"
        >
          {isRecording ? "Cancelar" : "Gravar"}
        </button>
      </div>
      {hint && <p className="mt-1 text-xs text-amber-400">{hint}</p>}
    </div>
  );
};
