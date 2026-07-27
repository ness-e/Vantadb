import { cn } from "../../lib/utils";

interface NbBentoProps {
  children: React.ReactNode;
  columns?: 3 | 4;
  className?: string;
}

export function NbBento({ children, columns = 3, className }: NbBentoProps) {
  return (
    <div className={cn("nb-bento", columns === 3 ? "nb-bento--col3" : "nb-bento--col4", className)}>
      {children}
    </div>
  );
}

interface NbBentoCellProps {
  children: React.ReactNode;
  span?: 2 | 3;
  row?: 2;
  featured?: boolean;
  className?: string;
}

export function NbBentoCell({ children, span, row, featured, className }: NbBentoCellProps) {
  return (
    <div
      className={cn(
        "nb-bento-cell",
        featured && "nb-bento-cell--featured",
        span === 2 && "nb-bento-cell--span2",
        span === 3 && "nb-bento-cell--span3",
        row === 2 && "nb-bento-cell--row2",
        className,
      )}
    >
      {children}
    </div>
  );
}
