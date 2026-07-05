import { useClipboard } from "../../hooks/useClipboard";
import { cn } from "../../lib/utils";

interface NbCopyCommandProps {
  command: string;
  variant?: "default" | "hero";
  showCopy?: boolean;
}

export function NbCopyCommand({
  command,
  variant = "default",
  showCopy = true,
}: NbCopyCommandProps) {
  const [copied, handleCopy] = useClipboard(command);

  const blockClass = cn("nb-cmd-block", variant === "hero" && "nb-cta-command");

  return (
    <>
      <div className={blockClass} onClick={showCopy ? handleCopy : undefined}>
        <span className="nb-cmd-prompt" aria-hidden="true">
          $
        </span>
        <code className="nb-cmd-code">{command}</code>
        <span className="nb-cmd-cursor" aria-hidden="true">
          _
        </span>
        {showCopy && (
          <button
            type="button"
            className="nb-cmd-copy"
            onClick={(e) => {
              e.stopPropagation();
              handleCopy();
            }}
            aria-label="Copy command"
          >
            {copied ? "OK" : "[]"}
          </button>
        )}
      </div>
      {copied && <span className="nb-cmd-feedback">copied to clipboard</span>}
    </>
  );
}
