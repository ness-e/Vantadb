interface NbBadgeProps {
  children: React.ReactNode;
  variant?: "amber" | "dark" | "cyan" | "pink" | "outline" | "ghost";
  className?: string;
}

export function NbBadge({ children, variant = "amber", className = "" }: NbBadgeProps) {
  return (
    <span className={`nb-badge nb-badge--${variant} ${className}`}>
      {children}
    </span>
  );
}
