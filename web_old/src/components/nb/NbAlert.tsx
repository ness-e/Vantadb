import type { ReactNode } from "react";

interface NbAlertProps {
  variant: "warning" | "success" | "error";
  icon: string;
  heading: string;
  children: ReactNode;
}

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
