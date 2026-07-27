"use client";

import { useEffect, useState } from "react";
import { Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";

export function ThemeToggle() {
  const { theme, setTheme, resolvedTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    Promise.resolve().then(() => setMounted(true));
  }, []);

  const current = mounted ? resolvedTheme ?? theme : "light";
  const isDark = current === "dark";

  return (
    <button
      onClick={() => setTheme(isDark ? "light" : "dark")}
      className="press inline-flex h-9 w-9 items-center justify-center border-4 border-black bg-[#FBF9F5] text-black   "
      aria-label={isDark ? "Activar tema claro" : "Activar tema oscuro"}
      aria-pressed={isDark}
      title={isDark ? "Tema claro" : "Tema oscuro"}
    >
      {mounted ? (
        isDark ? (
          <Sun className="h-4 w-4" strokeWidth={2.5} />
        ) : (
          <Moon className="h-4 w-4" strokeWidth={2.5} />
        )
      ) : (
        <span className="h-4 w-4" />
      )}
    </button>
  );
}
