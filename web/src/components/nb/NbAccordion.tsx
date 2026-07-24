import { useCallback, useRef, type ReactNode } from "react";
import { useSingleOpen } from "../../hooks/useSingleOpen";
import { cn } from "../../lib/utils";

interface NbAccordionProps<T> {
  items: T[];
  renderItem: (item: T, index: number, isOpen: boolean, toggle: () => void) => ReactNode;
  className?: string;
}

export function NbAccordion<T>({ items, renderItem, className }: NbAccordionProps<T>) {
  const [openIndex, toggle] = useSingleOpen();
  const containerRef = useRef<HTMLDivElement>(null);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    const target = e.target as HTMLElement;
    if (target.tagName !== "BUTTON" || !containerRef.current) return;

    const buttons = Array.from(
      containerRef.current.querySelectorAll<HTMLButtonElement>(
        ":scope > .nb-faq-item > button",
      ),
    );
    const idx = buttons.indexOf(target as HTMLButtonElement);
    if (idx === -1) return;

    let next: number;
    switch (e.key) {
      case "ArrowDown":
        next = Math.min(idx + 1, buttons.length - 1);
        break;
      case "ArrowUp":
        next = Math.max(idx - 1, 0);
        break;
      case "Home":
        next = 0;
        break;
      case "End":
        next = buttons.length - 1;
        break;
      default:
        return;
    }
    e.preventDefault();
    buttons[next]?.focus();
  }, []);

  return (
    <div
      ref={containerRef}
      className={cn("nb-faq-list", className ?? "")}
      onKeyDown={handleKeyDown}
    >
      {items.map((item, i) => renderItem(item, i, openIndex === i, () => toggle(i)))}
    </div>
  );
}
