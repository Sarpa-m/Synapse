import React from "react";
import { AlertCircle, CheckCircle2, Info } from "lucide-react";

interface StatusBannerProps {
  type: "success" | "error" | "info";
  message: string;
  hint?: string;
}

export const StatusBanner: React.FC<StatusBannerProps> = ({
  type,
  message,
  hint,
}) => {
  const styles = {
    error: {
      container: "bg-rose-950/70 border-rose-500/35 text-rose-200",
      icon: <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />,
      role: "alert" as const,
    },
    success: {
      container: "bg-emerald-950/60 border-emerald-500/35 text-emerald-200",
      icon: <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />,
      role: "status" as const,
    },
    info: {
      container: "bg-sky-950/60 border-sky-500/35 text-sky-200",
      icon: <Info className="w-4 h-4 text-sky-400 shrink-0" />,
      role: "status" as const,
    },
  }[type];

  return (
    <div
      role={styles.role}
      className={`synapse-alert-enter flex items-center gap-2.5 px-3.5 py-2.5 rounded-lg border text-xs ${styles.container}`}
    >
      {styles.icon}
      <span className="flex-1 leading-relaxed">{message}</span>
      {hint && <span className="text-[11px] opacity-75 shrink-0">{hint}</span>}
    </div>
  );
};
