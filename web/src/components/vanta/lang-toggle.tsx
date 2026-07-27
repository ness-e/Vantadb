"use client";

import { useLanguage } from "@/lib/language-provider";
import type { Lang } from "@/lib/dictionaries";

const LANGS: Lang[] = ["es", "en"];

export function LangToggle() {
  const { lang, setLang } = useLanguage();

  return (
    <div className="flex border-4 border-black bg-[#FBF9F5] shadow-[4px_4px_0_0_#000]   ">
      {LANGS.map((l) => (
        <button
          key={l}
          onClick={() => setLang(l)}
          className={`
            px-2 py-1.5 font-tech text-xs font-bold uppercase
            transition-colors
            ${
              lang === l
                ? "bg-black text-[#FBF9F5]  "
                : "bg-[#FBF9F5] text-black hover:bg-[#F2EDE2]   "
            }
          `}
          aria-label={l === "es" ? "Español" : "English"}
          aria-pressed={lang === l}
        >
          {l}
        </button>
      ))}
    </div>
  );
}
