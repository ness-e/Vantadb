import { useCallback, useState } from "react";

export function useSingleOpen(): [number | null, (index: number) => void] {
  const [openIndex, setOpenIndex] = useState<number | null>(null);
  const toggle = useCallback((index: number) => {
    setOpenIndex((prev) => (prev === index ? null : index));
  }, []);
  return [openIndex, toggle];
}
