import React, { useState } from "react";
import { Pencil, RotateCcw, Trash2 } from "lucide-react";
import type { Profile } from "../../types/domain";
import { KbdBadge } from "../../components/ui/KbdBadge";
import { StatusBanner } from "../../components/ui/StatusBanner";

interface ProfilesTabProps {
  profiles: Profile[];
  onSaveProfile: (profile: Profile) => Promise<void>;
  onDeleteProfile: (id: string) => Promise<void>;
  onRestoreDefaults?: () => Promise<void>;
}

function describeTemperature(temp: number): string {
  if (temp <= 0.3) return "Preciso";
  if (temp <= 0.9) return "Equilibrado";
  return "Criativo";
}

export const ProfilesTab: React.FC<ProfilesTabProps> = ({
  profiles,
  onSaveProfile,
  onDeleteProfile,
  onRestoreDefaults,
}) => {
  const [editingProfile, setEditingProfile] = useState<Profile | null>(null);
  const [name, setName] = useState("");
  const [systemPrompt, setSystemPrompt] = useState("");
  const [temperature, setTemperature] = useState(0.2);
  const [shortcut, setShortcut] = useState("");
  const [validationError, setValidationError] = useState<string | null>(null);

  const resetForm = () => {
    setEditingProfile(null);
    setName("");
    setSystemPrompt("");
    setTemperature(0.2);
    setShortcut("");
    setValidationError(null);
  };

  const startEditing = (profile: Profile) => {
    setEditingProfile(profile);
    setName(profile.name);
    setSystemPrompt(profile.system_prompt);
    setTemperature(profile.temperature);
    setShortcut(profile.shortcut ?? "");
    setValidationError(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmedName = name.trim();
    const trimmedPrompt = systemPrompt.trim();

    if (trimmedName.length === 0 || trimmedName.length > 60) {
      setValidationError("O nome do perfil deve ter entre 1 e 60 caracteres.");
      return;
    }
    if (trimmedPrompt.length === 0) {
      setValidationError("O System Prompt é obrigatório.");
      return;
    }
    if (Number.isNaN(temperature) || temperature < 0.0 || temperature > 2.0) {
      setValidationError("A temperatura deve estar entre 0.0 e 2.0.");
      return;
    }

    setValidationError(null);
    await onSaveProfile({
      id: editingProfile ? editingProfile.id : "",
      name: trimmedName,
      system_prompt: trimmedPrompt,
      temperature,
      shortcut: shortcut.trim() || null,
      is_default: editingProfile ? editingProfile.is_default : false,
    });

    resetForm();
  };

  return (
    <div className="space-y-6">
      <form
        onSubmit={handleSubmit}
        className="space-y-4 p-5 rounded-xl bg-[#12151B] border border-white/[0.08]"
      >
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-white">
            {editingProfile
              ? `Editar Perfil: ${editingProfile.name}`
              : "Novo Perfil de IA"}
          </h3>
          {editingProfile && (
            <button
              type="button"
              onClick={resetForm}
              className="text-xs text-zinc-400 hover:text-zinc-200 synapse-interactive"
            >
              Cancelar edição
            </button>
          )}
        </div>

        {validationError && (
          <StatusBanner type="error" message={validationError} />
        )}

        <div className="grid grid-cols-12 gap-3">
          <div className="col-span-6">
            <label className="block text-xs font-medium text-zinc-300 mb-1.5">
              Nome do Perfil
            </label>
            <input
              type="text"
              aria-label="Nome do Perfil"
              placeholder="Ex: Resumo Executivo"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-3 py-2 rounded-lg bg-[#0B0D10] border border-white/10 text-sm text-white placeholder-zinc-500 synapse-interactive focus:outline-none focus:border-sky-400/60"
            />
          </div>

          <div className="col-span-4">
            <div className="flex items-center justify-between mb-1.5">
              <label className="text-xs font-medium text-zinc-300">
                Temperatura
              </label>
              <span className="text-[11px] text-sky-300 font-medium">
                {describeTemperature(temperature)}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="range"
                min={0}
                max={2}
                step={0.1}
                aria-label="Controle deslizante de temperatura"
                value={Number.isNaN(temperature) ? 0 : Math.min(2, Math.max(0, temperature))}
                onChange={(e) => setTemperature(Number(e.target.value))}
                className="flex-1 accent-sky-400 cursor-pointer"
              />
              <input
                type="number"
                step="0.1"
                aria-label="Temperatura"
                value={temperature}
                onChange={(e) => setTemperature(Number(e.target.value))}
                className="w-16 px-2 py-1.5 rounded-lg bg-[#0B0D10] border border-white/10 text-xs font-mono text-white text-center focus:outline-none focus:border-sky-400/60"
              />
            </div>
          </div>

          <div className="col-span-2">
            <label className="block text-xs font-medium text-zinc-300 mb-1.5">
              Atalho (1-9)
            </label>
            <input
              type="text"
              maxLength={1}
              aria-label="Atalho Rápido"
              placeholder="1-9"
              value={shortcut}
              onChange={(e) =>
                setShortcut(e.target.value.replace(/[^1-9]/g, ""))
              }
              className="w-full px-3 py-2 rounded-lg bg-[#0B0D10] border border-white/10 text-sm font-mono text-white text-center placeholder-zinc-500 synapse-interactive focus:outline-none focus:border-sky-400/60"
            />
          </div>
        </div>

        <div>
          <label className="block text-xs font-medium text-zinc-300 mb-1.5">
            System Prompt
          </label>
          <textarea
            rows={3}
            aria-label="System Prompt"
            placeholder="Descreva exatamente como a IA deve transformar o texto..."
            value={systemPrompt}
            onChange={(e) => setSystemPrompt(e.target.value)}
            className="w-full px-3 py-2 rounded-lg bg-[#0B0D10] border border-white/10 text-sm text-white placeholder-zinc-500 synapse-interactive focus:outline-none focus:border-sky-400/60"
          />
        </div>

        <div className="flex items-center gap-2 pt-1">
          <button
            type="submit"
            className="px-4 py-2 rounded-lg bg-sky-500 hover:bg-sky-400 text-xs font-semibold text-slate-950 synapse-pressable"
          >
            {editingProfile ? "Salvar Alterações" : "Adicionar Perfil"}
          </button>
          {editingProfile && (
            <button
              type="button"
              onClick={resetForm}
              className="px-4 py-2 rounded-lg bg-white/[0.06] hover:bg-white/[0.1] text-xs font-medium text-zinc-200 synapse-pressable"
            >
              Cancelar
            </button>
          )}
        </div>
      </form>

      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <h4 className="text-xs font-medium text-zinc-400">
            Perfis cadastrados ({profiles.length})
          </h4>
          {onRestoreDefaults && (
            <button
              type="button"
              onClick={onRestoreDefaults}
              className="inline-flex items-center gap-1.5 text-xs text-zinc-400 hover:text-sky-300 synapse-interactive"
            >
              <RotateCcw className="w-3.5 h-3.5" />
              Restaurar Perfis Padrão
            </button>
          )}
        </div>

        <div className="space-y-2">
          {profiles.map((profile) => (
            <div
              key={profile.id}
              className="synapse-card-hover flex items-start justify-between gap-4 p-3.5 rounded-xl bg-[#12151B] border border-white/[0.08]"
            >
              <div className="min-w-0 space-y-1">
                <div className="flex items-center gap-2">
                  {profile.shortcut && (
                    <KbdBadge keys={profile.shortcut} size="xs" active />
                  )}
                  <span className="text-sm font-medium text-white truncate">
                    {profile.name}
                  </span>
                  <span className="text-[11px] font-mono text-zinc-400 px-1.5 py-0.5 rounded bg-white/[0.04]">
                    {profile.temperature.toFixed(1)}
                  </span>
                </div>
                <p className="text-xs text-zinc-400 line-clamp-2 leading-relaxed">
                  {profile.system_prompt}
                </p>
              </div>

              <div className="flex items-center gap-1.5 shrink-0">
                <button
                  type="button"
                  onClick={() => startEditing(profile)}
                  className="inline-flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-white/[0.05] hover:bg-white/[0.1] text-xs text-zinc-200 synapse-pressable"
                >
                  <Pencil className="w-3 h-3" />
                  Editar
                </button>
                <button
                  type="button"
                  onClick={() => onDeleteProfile(profile.id)}
                  className="inline-flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-rose-950/50 hover:bg-rose-900/60 border border-rose-500/25 text-xs text-rose-200 synapse-pressable"
                >
                  <Trash2 className="w-3 h-3" />
                  Excluir
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
