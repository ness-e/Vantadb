import React from "react";

interface VsTableRow {
  label: string;
  legacy: string;
  vanta: string;
}

interface VsTableProps {
  rows: VsTableRow[];
  title?: string;
  className?: string;
}

const tableStyle: React.CSSProperties = {
  width: "100%",
  borderCollapse: "collapse",
  fontFamily: "var(--font-mono)",
  fontSize: "var(--text-code, 0.82rem)",
};

const headerStyle: React.CSSProperties = {
  textAlign: "left",
  padding: "16px 12px",
  borderBottom: "1px solid var(--border)",
  fontWeight: 700,
  fontSize: "var(--text-label, 0.72rem)",
  letterSpacing: "0.14em",
  textTransform: "uppercase" as const,
};

const cellStyle: React.CSSProperties = {
  padding: "14px 12px",
  borderBottom: "1px solid var(--border)",
  verticalAlign: "top",
};

const legacyStyle: React.CSSProperties = {
  ...cellStyle,
  color: "var(--steel)",
};

const vantaStyle: React.CSSProperties = {
  ...cellStyle,
  color: "var(--foreground)",
  fontWeight: 600,
};

const labelStyle: React.CSSProperties = {
  ...cellStyle,
  fontWeight: 600,
  color: "var(--foreground)",
  whiteSpace: "nowrap" as const,
};

export const VsTable = React.memo(function VsTable({ rows, title, className = "" }: VsTableProps) {
  return (
    <div className={className} style={{ overflowX: "auto" }}>
      {title && (
        <h3
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "var(--text-label, 0.72rem)",
            fontWeight: 600,
            letterSpacing: "0.14em",
            textTransform: "uppercase",
            marginBottom: "16px",
            color: "var(--foreground)",
          }}
        >
          {title}
        </h3>
      )}
      <table style={tableStyle}>
        <thead>
          <tr>
            <th style={headerStyle}></th>
            <th style={{ ...headerStyle, color: "var(--steel)" }}>Legacy Stack</th>
            <th style={{ ...headerStyle, color: "var(--amber)" }}>VantaDB</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row, i) => (
            <tr key={i}>
              <td style={labelStyle}>{row.label}</td>
              <td style={legacyStyle}>{row.legacy}</td>
              <td style={vantaStyle}>{row.vanta}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
});
