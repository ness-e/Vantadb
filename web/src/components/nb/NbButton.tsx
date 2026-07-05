import { cn } from "../../lib/utils";

interface NbButtonProps {
  children: React.ReactNode;
  variant?: "primary" | "ghost" | "ghost-light" | "install";
  size?: "sm" | "md" | "lg";
  className?: string;
  href?: string;
  onClick?: () => void;
  type?: "button" | "submit";
}

export function NbButton({
  children,
  variant = "primary",
  size = "md",
  className,
  href,
  onClick,
  type = "button",
}: NbButtonProps) {
  const classes = cn(
    "nb-btn",
    variant === "ghost" && "nb-btn--ghost",
    variant === "ghost-light" && "nb-btn--ghost-light",
    variant === "install" && "nb-btn--install",
    size === "sm" && "nb-btn--sm",
    size === "lg" && "nb-btn--lg",
    className,
  );

  if (href) {
    return (
      <a href={href} className={classes} onClick={onClick}>
        {children}
      </a>
    );
  }

  return (
    <button type={type} className={classes} onClick={onClick}>
      {children}
    </button>
  );
}
