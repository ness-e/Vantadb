import React from "react";

type LogoVariant = "mark" | "full";
type LogoSize = "xs" | "sm" | "md" | "lg" | "xl";

interface VantaDBLogoProps {
  variant?: LogoVariant;
  size?: LogoSize;
  accentColor?: string;
  noAnimation?: boolean;
  className?: string;
  "aria-label"?: string;
  inverted?: boolean;
}

const SIZE_MAP: Record<LogoSize, number> = {
  xs: 20,
  sm: 28,
  md: 40,
  lg: 56,
  xl: 80,
};

const VantaDBMark: React.FC<{
  size?: number;
  accentColor?: string;
  className?: string;
  inverted?: boolean;
}> = ({ size = 40, accentColor = "var(--amber)", className = "", inverted }) => {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
      className={`vdb-mark ${className}${inverted ? " vdb-mark--inverted" : ""}`}
    >
      <circle cx="32" cy="32" r="24" fill="none" stroke="var(--foreground)" strokeWidth="6" />
      <circle cx="32" cy="32" r="10" fill={accentColor} />
    </svg>
  );
};

export const VantaDBLogoFull: React.FC<{
  size?: LogoSize;
  className?: string;
  inverted?: boolean;
}> = ({ size = "md", className = "", inverted }) => {
  const markPx = SIZE_MAP[size];
  const fontPx = Math.round(markPx * 0.7);

  return (
    <div
      className={`vdb-logo-full ${className}${inverted ? " vdb-logo-full--inverted" : ""}`}
      role="img"
      aria-label="VantaDB"
    >
      <VantaDBMark size={markPx} inverted={inverted} />
      <span className="vdb-wordmark" style={{ fontSize: `${fontPx}px` }} aria-hidden="true">
        <span>Vanta</span>
        <span className="vdb-wordmark-db">DB</span>
      </span>
    </div>
  );
};

const VantaDBLogo: React.FC<VantaDBLogoProps> = React.memo(
  ({
    variant = "full",
    size = "md",
    accentColor,
    className = "",
    "aria-label": ariaLabel = "VantaDB",
    inverted,
  }) => {
    const markPx = SIZE_MAP[size];

    switch (variant) {
      case "mark":
        return (
          <div role="img" aria-label={ariaLabel} className={className}>
            <VantaDBMark size={markPx} accentColor={accentColor} inverted={inverted} />
          </div>
        );
      case "full":
      default:
        return <VantaDBLogoFull size={size} className={className} inverted={inverted} />;
    }
  },
);

VantaDBLogo.displayName = "VantaDBLogo";

export default VantaDBLogo;
