import { useCallback, useEffect, useState } from "react";
import { tauriInvoke } from "@/lib/tauri";

export type Theme = "system" | "light" | "dark";

function getSystemTheme(): "light" | "dark" {
  if (
    typeof window !== "undefined" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
  ) {
    return "dark";
  }
  return "light";
}

function applyTheme(theme: Theme) {
  const resolved = theme === "system" ? getSystemTheme() : theme;
  const root = document.documentElement;
  if (resolved === "dark") {
    root.classList.add("dark");
  } else {
    root.classList.remove("dark");
  }
}

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>("system");
  const [loaded, setLoaded] = useState(false);

  // Load persisted theme on mount
  useEffect(() => {
    tauriInvoke<string | null>("get_setting", { key: "theme" })
      .then((value) => {
        const t = (value as Theme) || "system";
        setThemeState(t);
        applyTheme(t);
        setLoaded(true);
      })
      .catch(() => {
        applyTheme("system");
        setLoaded(true);
      });
  }, []);

  // Listen for system theme changes when in "system" mode
  useEffect(() => {
    if (!loaded) return;

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      if (theme === "system") {
        applyTheme("system");
      }
    };
    mediaQuery.addEventListener("change", handler);
    return () => mediaQuery.removeEventListener("change", handler);
  }, [theme, loaded]);

  const setTheme = useCallback(
    (newTheme: Theme) => {
      setThemeState(newTheme);
      applyTheme(newTheme);
      tauriInvoke<void>("set_setting", {
        key: "theme",
        value: newTheme,
      }).catch(() => {
        // Silently fail - theme is already applied visually
      });
    },
    []
  );

  const resolvedTheme = theme === "system" ? getSystemTheme() : theme;

  return { theme, setTheme, resolvedTheme };
}
