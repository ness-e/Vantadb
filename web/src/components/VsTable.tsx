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
    <section className={cn("vs-table", className)}>
      {title && <h2 className="vs-table__title">{title}</h2>}
      {subtitle && <p className="vs-table__subtitle">{subtitle}</p>}
      <div className="vs-table__grid">
        <div className="vs-table__header vs-table__header--feature">Feature</div>
        <div className="vs-table__header vs-table__header--legacy">Legacy</div>
        <div className="vs-table__header vs-table__header--vanta">VantaDB</div>
        {rows.map((row, i) => (
          <div
            key={i}
            className={cn(
              "vs-table__cell vs-table__cell--feature",
              row.highlight && "vs-table__cell--highlight",
            )}
          >
            {row.feature}
          </div>
        ))}
        {rows.map((row, i) => (
          <div
            key={i}
            className={cn(
              "vs-table__cell vs-table__cell--legacy",
              row.highlight && "vs-table__cell--highlight",
            )}
          >
            {row.legacy}
          </div>
        ))}
        {rows.map((row, i) => (
          <div
            key={i}
            className={cn(
              "vs-table__cell vs-table__cell--vanta",
              row.highlight && "vs-table__cell--highlight",
            )}
          >
            {row.vantadb}
          </div>
        ))}
      </div>
    </section>
  );
}
