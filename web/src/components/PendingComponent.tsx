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
      <span
        style={{
          fontFamily: "var(--font-display)",
          fontSize: "0.72rem",
          letterSpacing: "0.12em",
          color: "var(--muted)",
        }}
      >
        Loading...
      </span>
    </div>
  );
}
