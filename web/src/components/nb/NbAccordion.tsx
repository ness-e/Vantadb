import type { ReactNode } from "react";
import { useSingleOpen } from "../../hooks/useSingleOpen";
import { cn } from "../../lib/utils";

interface NbAccordionProps<T> {
  items: T[];
  renderItem: (item: T, index: number, isOpen: boolean, toggle: () => void) => ReactNode;
  className?: string;
}

export function NbAccordion<T>({ items, renderItem, className }: NbAccordionProps<T>) {
  const [openIndex, toggle] = useSingleOpen();
  return (
    <div className={cn("nb-faq-list", className ?? "")}>
      {items.map((item, i) => renderItem(item, i, openIndex === i, () => toggle(i)))}
    </div>
  );
}
