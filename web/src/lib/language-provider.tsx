"use client";

import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  type ReactNode,
} from "react";
import { dictionaries, type Lang } from "./dictionaries";

type LanguageContextType = {
  lang: Lang;
  setLang: (l: Lang) => void;
  t: (key: string, params?: Record<string, string>) => string;
};

const LanguageContext = createContext<LanguageContextType>({
  lang: "es",
  setLang: () => {},
  t: (key) => key,
});

export function LanguageProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>("es");
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    // setState-in-effect is intentional here: we read localStorage on mount
    // (client-only) to sync the initial language. This is the standard pattern
    // for avoiding SSR hydration mismatches — SSR renders with "es" default,
    // then the effect syncs to the user's stored/browser language after mount.
    // The cascading render is acceptable (runs once on mount).
    const stored = localStorage.getItem("vantadb-lang") as Lang | null;
    if (stored && (stored === "es" || stored === "en")) {
      setLangState(stored);
      document.documentElement.lang = stored;
    } else {
      // Auto-detect from browser
      const browserLang = navigator.language?.slice(0, 2);
      const detected: Lang = browserLang === "en" ? "en" : "es";
      setLangState(detected);
      document.documentElement.lang = detected;
      localStorage.setItem("vantadb-lang", detected);
    }
    setMounted(true);
  }, []);

  const setLang = useCallback((l: Lang) => {
    setLangState(l);
    localStorage.setItem("vantadb-lang", l);
    document.documentElement.lang = l;
  }, []);

  const t = useCallback(
    (key: string, params?: Record<string, string>) => {
      let value = dictionaries[lang]?.[key] || dictionaries["es"]?.[key] || key;
      if (params) {
        Object.entries(params).forEach(([k, v]) => {
          value = value.replace(`{${k}}`, v);
        });
      }
      return value;
    },
    [lang]
  );

  return (
    <LanguageContext.Provider value={{ lang, setLang, t }}>
      {children}
    </LanguageContext.Provider>
  );
}

export function useLanguage() {
  return useContext(LanguageContext);
}
