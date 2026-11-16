import React from "react";
import { Sparkles, Loader2 } from "lucide-react";
import { KbdBadge } from "../../components/ui/KbdBadge";

interface PromptInputProps {
  inputRef: React.RefObject<HTMLInputElement>;
  value: string;
  isLoading: boolean;
  onChange: (value: string) => void;
}

export const PromptInput: React.FC<PromptInputProps> = ({
  inputRef,
  value,
  isLoading,
  onChange,
}) => {
  return (
    <div className="relative flex items-center gap-3 px-4 py-3 border-b border-white/[0.08] bg-[#12151B]/95">
      {isLoading ? (
        <Loader2
          data-testid="popup-loading-spinner"
          className="w-4 h-4 text-sky-400 animate-spin shrink-0"
        />
      ) : (
        <Sparkles className="w-4 h-4 text-sky-400 shrink-0" />
      )}

      <input
        ref={inputRef}
        type="text"
        aria-label="Prompt ou busca de perfil"
        disabled={isLoading}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Escolha um perfil (1-4, ↑↓) ou digite uma instrução livre..."
        className="w-full bg-transparent text-sm text-zinc-100 placeholder-zinc-500 focus:outline-none disabled:opacity-50"
      />

      <KbdBadge keys="Esc" size="xs" />

      {isLoading && (
        <div className="absolute inset-x-0 bottom-0 h-[2px] overflow-hidden bg-sky-500/15">
          <div className="h-full w-1/3 bg-sky-400 synapse-loading-bar" />
        </div>
      )}
    </div>
  );
};
