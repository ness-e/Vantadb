import { cn } from "../../lib/utils";

interface NbDitherImageProps {
  src: string;
  alt: string;
  className?: string;
}

export function NbDitherImage({ src, alt, className }: NbDitherImageProps) {
  return (
    <div className={cn("nb-dither", className)}>
      <img src={src} alt={alt} />
    </div>
  );
}
