interface NbAvatarProps {
  initials: string;
  variant?: "surface" | "amber" | "cyan" | "pink" | "dark";
  size?: "sm" | "md" | "lg";
  stacked?: boolean;
  className?: string;
}

const BG: Record<string, string> = {
  surface: "",
  amber: "bg-[var(--amber)] text-[var(--text-on-amber)]",
  cyan: "bg-[var(--success)] text-[var(--black)]",
  pink: "bg-[var(--danger)] text-[var(--foreground)]",
  dark: "bg-[var(--black)] text-[var(--foreground)]",
};

export function NbAvatar({
  initials,
  variant = "surface",
  size = "md",
  stacked,
  className = "",
}: NbAvatarProps) {
  const sizeClass = size !== "md" ? `nb-avatar--${size}` : "";
  const stackClass = stacked ? "nb-avatar--stack" : "";
  return (
    <div className={`nb-avatar ${sizeClass} ${stackClass} ${BG[variant]} ${className}`}>
      {initials}
    </div>
  );
}

export function NbAvatarStack({ children }: { children: React.ReactNode }) {
  return <div className="flex items-center">{children}</div>;
}
