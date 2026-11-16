import React from "react";

interface KbdBadgeProps {
  keys: string;
  size?: "xs" | "sm";
  active?: boolean;
}

export const KbdBadge: React.FC<KbdBadgeProps> = ({
  keys,
  size = "xs",
  active = false,
}) => {
  const parts = keys
    .split("+")
    .map((p) => p.trim())
    .filter(Boolean);

  const sizeClass =
    size === "xs"
      ? "px-1.5 py-0.5 text-[10px] min-w-[20px]"
      : "px-2 py-0.5 text-[11px] min-w-[24px]";

  return (
    <span className="inline-flex items-center gap-1 shrink-0">
      {parts.map((part, idx) => (
        <React.Fragment key={`${part}-${idx}`}>
          {idx > 0 && <span className="text-[10px] text-zinc-500">+</span>}
          <kbd
            className={`inline-flex items-center justify-center rounded border font-mono font-medium leading-none synapse-interactive ${sizeClass} ${
              active
                ? "bg-sky-500/20 border-sky-400/50 text-sky-200"
                : "bg-white/[0.06] border-white/10 text-zinc-300"
            }`}
          >
            {part}
          </kbd>
        </React.Fragment>
      ))}
    </span>
  );
};
