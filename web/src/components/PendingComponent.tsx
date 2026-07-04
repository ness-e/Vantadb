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
      <span className="swiss-eyebrow" style={{ fontSize: "0.72rem" }}>
        Loading...
      </span>
    </div>
  );
}
