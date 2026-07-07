import { useCallback, useState } from "react";

export function useClipboard(text: string, timeout = 2000): [boolean, () => Promise<void>] {
  const [copied, setCopied] = useState(false);
  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
    }
    setCopied(true);
    setTimeout(() => setCopied(false), timeout);
  }, [text, timeout]);
  return [copied, handleCopy];
}
