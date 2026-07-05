export function PendingComponent() {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        height: "60vh",
        color: "var(--muted)",
      }}
    >
      <span className="nb-label" style={{ fontSize: "0.72rem", marginBottom: 0 }}>
        Loading...
      </span>
    </div>
  );
}
