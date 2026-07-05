import { cn } from "../../lib/utils";

interface NbMarqueeProps {
  children: React.ReactNode;
  speed?: "normal" | "slow" | "fast";
  className?: string;
}

export function NbMarquee({ children, speed = "normal", className }: NbMarqueeProps) {
  return (
    <div
      className={cn(
        "nb-marquee",
        speed === "slow" && "nb-marquee--slow",
        speed === "fast" && "nb-marquee--fast",
        className,
      )}
    >
      <div className="nb-marquee-track">
        {children}
        {children}
      </div>
    </div>
  );
}
