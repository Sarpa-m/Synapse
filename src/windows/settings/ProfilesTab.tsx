import React, { useState } from "react";
import type { Profile } from "../../types/domain";

interface ProfilesTabProps {
  profiles: Profile[];
  onSaveProfile: (profile: Profile) => Promise<void>;
  onDeleteProfile: (id: string) => Promise<void>;
  onRestoreDefaults?: () => Promise<void>;
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
        className="space-y-3 p-4 rounded-lg bg-slate-800/50 border border-slate-700"
      >
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-white">
            {editingProfile ? `Editar Perfil: ${editingProfile.name}` : "Novo Perfil de IA"}
          </h3>
          {editingProfile && (
            <button
              type="button"
              onClick={resetForm}
              className="text-xs text-slate-400 hover:text-slate-200"
            >
              Cancelar edição
            </button>
          )}
        </div>

        {validationError && (
          <div role="alert" className="text-xs text-red-400">
            {validationError}
          </div>
        )}

        <div className="grid grid-cols-4 gap-3">
          <div className="col-span-2">
            <label className="block text-xs text-slate-300 mb-1">Nome do Perfil</label>
            <input
              type="text"
              aria-label="Nome do Perfil"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-3 py-1.5 rounded bg-slate-900 border border-slate-700 text-sm text-white"
            />
          </div>
          <div>
            <label className="block text-xs text-slate-300 mb-1">Temperatura (0.0 a 2.0)</label>
            <input
              type="number"
              step="0.1"
              aria-label="Temperatura"
              value={temperature}
              onChange={(e) => setTemperature(Number(e.target.value))}
              className="w-full px-3 py-1.5 rounded bg-slate-900 border border-slate-700 text-sm text-white"
            />
          </div>
          <div>
            <label className="block text-xs text-slate-300 mb-1">Atalho (1-9)</label>
            <input
              type="text"
              maxLength={1}
              aria-label="Atalho Rápido"
              placeholder="Ex: 4"
              value={shortcut}
              onChange={(e) => setShortcut(e.target.value.replace(/[^1-9]/g, ""))}
              className="w-full px-3 py-1.5 rounded bg-slate-900 border border-slate-700 text-sm text-white"
            />
          </div>
        </div>

        <div>
          <label className="block text-xs text-slate-300 mb-1">System Prompt</label>
          <textarea
            rows={3}
            aria-label="System Prompt"
            value={systemPrompt}
            onChange={(e) => setSystemPrompt(e.target.value)}
            className="w-full px-3 py-1.5 rounded bg-slate-900 border border-slate-700 text-sm text-white"
          />
        </div>

        <div className="flex items-center gap-2">
          <button
            type="submit"
            className="px-4 py-1.5 rounded bg-indigo-600 hover:bg-indigo-500 text-xs font-medium text-white"
          >
            {editingProfile ? "Salvar Alterações" : "Adicionar Perfil"}
          </button>
          {editingProfile && (
            <button
              type="button"
              onClick={resetForm}
              className="px-4 py-1.5 rounded bg-slate-700 hover:bg-slate-600 text-xs font-medium text-slate-200"
            >
              Cancelar
            </button>
          )}
        </div>
      </form>

      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <h4 className="text-xs font-semibold uppercase tracking-wider text-slate-400">
            Perfis Cadastrados ({profiles.length})
          </h4>
          {onRestoreDefaults && (
            <button
              type="button"
              onClick={onRestoreDefaults}
              className="text-xs text-slate-400 hover:text-indigo-300"
            >
              Restaurar Perfis Padrão
            </button>
          )}
        </div>

        {profiles.map((p) => (
          <div
            key={p.id}
            className={`flex items-center justify-between p-3 rounded border ${
              editingProfile?.id === p.id
                ? "bg-indigo-950/40 border-indigo-500/70"
                : "bg-slate-800/60 border-slate-700/70"
            }`}
          >
            <div className="min-w-0 pr-3">
              <div className="text-sm font-medium text-white flex items-center gap-2">
                <span className="truncate">{p.name}</span>
                <span className="text-xs text-slate-400">
                  (temp: {p.temperature.toFixed(1)})
                </span>
                {p.shortcut && (
                  <kbd className="px-1.5 py-0.5 text-[10px] rounded bg-slate-900 border border-slate-700 text-slate-300">
                    {p.shortcut}
                  </kbd>
                )}
              </div>
              <p className="text-xs text-slate-400 line-clamp-1">{p.system_prompt}</p>
            </div>
            <div className="flex items-center gap-3 shrink-0">
              <button
                type="button"
                onClick={() => startEditing(p)}
                className="text-xs text-indigo-400 hover:text-indigo-300"
              >
                Editar
              </button>
              {!p.is_default && (
                <button
                  type="button"
                  onClick={() => {
                    if (editingProfile?.id === p.id) resetForm();
                    void onDeleteProfile(p.id);
                  }}
                  className="text-xs text-red-400 hover:text-red-300"
                >
                  Excluir
                </button>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
