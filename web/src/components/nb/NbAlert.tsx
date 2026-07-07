import type { ReactNode } from "react";

interface NbAlertProps {
  variant: "warning" | "success" | "error";
  icon: string;
  heading: string;
  children: ReactNode;
}

const _ICON_BG: Record<string, string> = {
  warning: "nb-alert-icon",
  success: "nb-alert-icon",
  error: "nb-alert-icon",
};

export function NbAlert({ variant, icon, heading, children }: NbAlertProps) {
  return (
    <div className={`nb-alert nb-alert--${variant}`}>
      <span className="nb-alert-icon">{icon}</span>
      <div>
        <p className="nb-alert-heading">{heading}</p>
        <p className="nb-alert-desc">{children}</p>
      </div>
    </div>
  );
}
