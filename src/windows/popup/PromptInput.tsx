import React from "react";
import { Sparkles, Loader2 } from "lucide-react";

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
    <div className="flex items-center gap-3 px-4 py-3 border-b border-slate-800/80 bg-slate-900/95">
      {isLoading ? (
        <Loader2
          data-testid="popup-loading-spinner"
          className="w-5 h-5 text-indigo-400 animate-spin shrink-0"
        />
      ) : (
        <Sparkles className="w-5 h-5 text-indigo-400 shrink-0" />
      )}
      <input
        ref={inputRef}
        type="text"
        aria-label="Prompt ou busca de perfil"
        disabled={isLoading}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Escolha um perfil (1-3, ↑↓) ou digite uma instrução livre e aperte Enter..."
        className="w-full bg-transparent text-sm text-slate-100 placeholder-slate-400 focus:outline-none disabled:opacity-50"
      />
    </div>
  );
};
