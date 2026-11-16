import React, { useEffect, useRef } from "react";
import type { Profile } from "../../types/domain";
import { KbdBadge } from "../../components/ui/KbdBadge";

interface ProfileListProps {
  profiles: Profile[];
  selectedIndex: number;
  disabled: boolean;
  onSelect: (profile: Profile) => void;
  onHoverIndex: (index: number) => void;
}

export const ProfileList: React.FC<ProfileListProps> = ({
  profiles,
  selectedIndex,
  disabled,
  onSelect,
  onHoverIndex,
}) => {
  const itemRefs = useRef<Array<HTMLLIElement | null>>([]);

  useEffect(() => {
    const activeItem = itemRefs.current[selectedIndex];
    activeItem?.scrollIntoView?.({ block: "nearest" });
  }, [selectedIndex]);

  if (profiles.length === 0) {
    return (
      <div className="px-4 py-4 flex items-center gap-2 text-xs text-zinc-400">
        <span>Pressione</span>
        <KbdBadge keys="Enter" size="xs" active />
        <span>para executar seu prompt livre sobre o texto selecionado.</span>
      </div>
    );
  }

  return (
    <ul
      role="listbox"
      aria-label="Lista de perfis de IA"
      className="py-1.5 max-h-56 overflow-y-auto synapse-scroll"
    >
      {profiles.map((profile, idx) => {
        const isActive = idx === selectedIndex;
        return (
          <li
            key={profile.id}
            ref={(el) => {
              itemRefs.current[idx] = el;
            }}
            role="option"
            aria-selected={isActive}
            onMouseEnter={() => onHoverIndex(idx)}
            onClick={() => !disabled && onSelect(profile)}
            className={`relative flex items-center justify-between gap-3 mx-1.5 px-3 py-2 rounded-lg cursor-pointer synapse-interactive ${
              isActive
                ? "bg-sky-500/15 text-white ring-1 ring-inset ring-sky-400/35"
                : "text-zinc-300 hover:bg-white/[0.04]"
            } ${disabled ? "opacity-50 pointer-events-none" : ""}`}
          >
            <div className="flex items-center gap-3 min-w-0">
              <KbdBadge
                keys={profile.shortcut ?? String(idx + 1)}
                size="xs"
                active={isActive}
              />
              <div className="min-w-0">
                <div className="text-sm font-medium truncate leading-snug">
                  {profile.name}
                </div>
                {isActive && (
                  <div className="text-[11px] text-zinc-400 truncate max-w-[380px] mt-0.5">
                    {profile.system_prompt}
                  </div>
                )}
              </div>
            </div>

            <span
              className={`text-[11px] font-mono shrink-0 px-1.5 py-0.5 rounded ${
                isActive
                  ? "bg-sky-500/15 text-sky-300"
                  : "text-zinc-500"
              }`}
            >
              {profile.temperature.toFixed(1)}
            </span>
          </li>
        );
      })}
    </ul>
  );
};
