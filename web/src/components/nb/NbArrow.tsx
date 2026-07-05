import { cn } from "../../lib/utils";

interface NbArrowProps {
  children: React.ReactNode;
  variant?: "amber" | "white";
  className?: string;
  href?: string;
  onClick?: () => void;
}

export function NbArrow({ children, variant = "amber", className, href, onClick }: NbArrowProps) {
  const classes = cn("nb-arrow", variant === "white" && "nb-arrow--white", className);

  if (href) {
    return (
      <a href={href} className={classes} onClick={onClick}>
        {children}
      </a>
    );
  }

  return (
    <span className={classes} onClick={onClick}>
      {children}
    </span>
  );
}
