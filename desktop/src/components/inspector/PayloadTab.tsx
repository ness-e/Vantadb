// Payload tab (VS-06): preview markdown (react-markdown, escapa HTML por
// defecto — el payload es contenido arbitrario ingerido) con pretty-JSON si
// parsea, ↔ editar JSON con CodeMirror 6 + lint (jsonParseLinter). El texto se
// guarda tal cual (string) vía vantaPut — commit explícito, nunca auto-guardar.
import { useMemo, useState } from "react";
import CodeMirror from "@uiw/react-codemirror";
import { json, jsonParseLinter } from "@codemirror/lang-json";
import { linter, lintGutter } from "@codemirror/lint";
import { oneDark } from "@codemirror/theme-one-dark";
import ReactMarkdown from "react-markdown";

interface Props {
  text: string;
  onChange: (t: string) => void;
  dark: boolean;
}

export default function PayloadTab({ text, onChange, dark }: Props) {
  const [mode, setMode] = useState<"preview" | "edit">("preview");

  const parsedJson = useMemo(() => {
    try {
      const v = JSON.parse(text);
      return typeof v === "object" && v !== null ? v : null;
    } catch {
      return null;
    }
  }, [text]);

  return (
    <div>
      <div className="flex gap-2">
        <button
          type="button"
          onClick={() => setMode("preview")}
          className={`flex-1 border-2 border-foreground px-2 py-1.5 font-tech text-[10px] uppercase tracking-widest ${
            mode === "preview" ? "bg-neon text-background" : "bg-background text-muted-foreground"
          }`}
          aria-pressed={mode === "preview"}
        >
          preview
        </button>
        <button
          type="button"
          onClick={() => setMode("edit")}
          className={`flex-1 border-2 border-foreground px-2 py-1.5 font-tech text-[10px] uppercase tracking-widest ${
            mode === "edit" ? "bg-neon text-background" : "bg-background text-muted-foreground"
          }`}
          aria-pressed={mode === "edit"}
        >
          editar json
        </button>
      </div>

      {mode === "preview" ? (
        parsedJson ? (
          <pre className="mt-2 max-h-[420px] overflow-auto border-2 border-foreground bg-background p-3 font-mono text-[11px]">
            {JSON.stringify(parsedJson, null, 2)}
          </pre>
        ) : (
          <div className="mt-2 max-h-[420px] overflow-auto border-2 border-foreground bg-background p-3 text-sm leading-relaxed">
            <ReactMarkdown>{text}</ReactMarkdown>
          </div>
        )
      ) : (
        <div className="mt-2 border-2 border-foreground">
          <CodeMirror
            value={text}
            onChange={onChange}
            height="320px"
            theme={dark ? oneDark : "light"}
            extensions={[json(), linter(jsonParseLinter()), lintGutter()]}
            basicSetup={{ foldGutter: false, autocompletion: false }}
            style={{ fontSize: 12 }}
          />
        </div>
      )}
    </div>
  );
}