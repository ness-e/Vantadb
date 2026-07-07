import type { ReactNode } from "react";
import { memo } from "react";
import { cn } from "../../lib/utils";

interface NbAccordionItemProps {
  label: string;
  title: string;
  isOpen: boolean;
  onToggle: () => void;
  children: ReactNode;
  labelClassName?: string;
  titleClassName?: string;
  toggleClassName?: string;
  contentClassName?: string;
}

export const NbAccordionItem = memo(function NbAccordionItem({
  label,
  title,
  isOpen,
  onToggle,
  children,
  labelClassName,
  titleClassName,
  toggleClassName,
  contentClassName,
}: NbAccordionItemProps) {
  return (
    <div className="nb-faq-item">
      <button
        type="button"
        className={cn("nb-faq-q", titleClassName)}
        onClick={onToggle}
        aria-expanded={isOpen}
      >
        <span className={cn("nb-mono-label", labelClassName)}>{label}</span>
        <span className="nb-section-headline">{title}</span>
        <span className={cn("nb-faq-toggle", toggleClassName)} aria-hidden="true">
          {isOpen ? "−" : "+"}
        </span>
      </button>
      {isOpen && <div className={cn("nb-faq-a", contentClassName)}>{children}</div>}
    </div>
  );
});
