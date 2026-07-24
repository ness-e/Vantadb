import { memo, useId, type ReactNode } from "react";
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
  const id = useId();
  const buttonId = `nb-accordion-btn-${id}`;
  const panelId = `nb-accordion-panel-${id}`;

  return (
    <div className="nb-faq-item">
      <button
        type="button"
        id={buttonId}
        className={cn("nb-faq-q", titleClassName)}
        onClick={onToggle}
        aria-expanded={isOpen}
        aria-controls={panelId}
      >
        <span className={cn("nb-mono-label", labelClassName)}>{label}</span>
        <span className="nb-section-headline">{title}</span>
        <span className={cn("nb-faq-toggle", toggleClassName)} aria-hidden="true">
          {isOpen ? "−" : "+"}
        </span>
      </button>
      {isOpen && (
        <div
          id={panelId}
          role="region"
          aria-labelledby={buttonId}
          className={cn("nb-faq-a", contentClassName)}
        >
          {children}
        </div>
      )}
    </div>
  );
});
