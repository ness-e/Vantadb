import { cn } from "../lib/utils";

export interface VsRow {
  feature: string;
  legacy: string | React.ReactNode;
  vantadb: string | React.ReactNode;
  highlight?: boolean;
}

interface VsTableProps {
  title?: string;
  subtitle?: string;
  rows: VsRow[];
  className?: string;
}

export function VsTable({ title, subtitle, rows, className }: VsTableProps) {
  return (
    <section className={cn("vs-table", className)} aria-label={title ?? "Comparison table"}>
      {title && <h2 className="vs-table__title">{title}</h2>}
      {subtitle && <p className="vs-table__subtitle">{subtitle}</p>}
      <div className="vs-table__grid" role="table" aria-label="Feature comparison">
        <div className="vs-table__header vs-table__header--feature" role="columnheader">Feature</div>
        <div className="vs-table__header vs-table__header--legacy" role="columnheader">Legacy</div>
        <div className="vs-table__header vs-table__header--vanta" role="columnheader">VantaDB</div>
        {rows.map((row, i) => (
          <div
            key={`f-${i}`}
            className={cn(
              "vs-table__cell vs-table__cell--feature",
              row.highlight && "vs-table__cell--highlight",
            )}
            role="cell"
          >
            {row.feature}
          </div>
        ))}
        {rows.map((row, i) => (
          <div
            key={`l-${i}`}
            className={cn(
              "vs-table__cell vs-table__cell--legacy",
              row.highlight && "vs-table__cell--highlight",
            )}
            role="cell"
          >
            {row.legacy}
          </div>
        ))}
        {rows.map((row, i) => (
          <div
            key={`v-${i}`}
            className={cn(
              "vs-table__cell vs-table__cell--vanta",
              row.highlight && "vs-table__cell--highlight",
            )}
            role="cell"
          >
            {row.vantadb}
          </div>
        ))}
      </div>
    </section>
  );
}
