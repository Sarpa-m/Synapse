import React from "react";
import type { Profile } from "../../types/domain";

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
  if (profiles.length === 0) {
    return (
      <div className="px-4 py-3 text-xs text-slate-400">
        Pressione <kbd className="px-1.5 py-0.5 bg-slate-800 rounded text-slate-200">Enter</kbd> para executar seu prompt livre sobre o texto selecionado.
      </div>
    );
  }

  return (
    <ul role="listbox" aria-label="Lista de perfis de IA" className="py-1.5 max-h-56 overflow-y-auto">
      {profiles.map((profile, idx) => {
        const isActive = idx === selectedIndex;
        return (
          <li
            key={profile.id}
            role="option"
            aria-selected={isActive}
            onMouseEnter={() => onHoverIndex(idx)}
            onClick={() => !disabled && onSelect(profile)}
            className={`flex items-center justify-between px-4 py-2.5 cursor-pointer transition-colors ${
              isActive
                ? "bg-indigo-600/25 text-white border-l-2 border-indigo-400"
                : "text-slate-300 hover:bg-slate-800/60"
            } ${disabled ? "opacity-50 pointer-events-none" : ""}`}
          >
            <div className="flex items-center gap-3 min-w-0">
              <span className="inline-flex items-center justify-center w-5 h-5 text-xs font-semibold rounded bg-slate-800 text-indigo-300 border border-slate-700">
                {profile.shortcut ?? idx + 1}
              </span>
              <span className="text-sm font-medium truncate">{profile.name}</span>
            </div>
            <span className="text-[11px] text-slate-400 ml-3 shrink-0">
              temp {profile.temperature.toFixed(1)}
            </span>
          </li>
        );
      })}
    </ul>
  );
};
