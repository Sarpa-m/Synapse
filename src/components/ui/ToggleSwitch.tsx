import React from "react";

interface ToggleSwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  description?: string;
  disabled?: boolean;
}

export const ToggleSwitch: React.FC<ToggleSwitchProps> = ({
  checked,
  onChange,
  label,
  description,
  disabled = false,
}) => {
  return (
    <div className="flex items-center justify-between gap-4">
      <div className="min-w-0">
        <span className="block text-sm font-medium text-zinc-200">{label}</span>
        {description && (
          <span className="block text-xs text-zinc-400 mt-0.5">{description}</span>
        )}
      </div>

      <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-label={label}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border p-0.5 synapse-interactive focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-400/60 disabled:opacity-50 ${
          checked
            ? "bg-sky-500/90 border-sky-400"
            : "bg-white/10 border-white/15 hover:bg-white/15"
        }`}
      >
        <span
          className="pointer-events-none block h-4 w-4 rounded-full bg-white shadow-sm"
          style={{
            transform: checked ? "translateX(20px)" : "translateX(0px)",
            transition: "transform 160ms var(--ease-out)",
          }}
        />
      </button>
    </div>
  );
};
