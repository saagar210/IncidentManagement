import { useEffect } from "react";
import { useNavigate } from "react-router-dom";

interface KeyboardShortcutOptions {
  onQuickAdd?: () => void;
  onReport?: () => void;
  onSearch?: () => void;
  onSave?: () => void;
}

export function useKeyboardShortcuts(options: KeyboardShortcutOptions = {}) {
  const navigate = useNavigate();

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      // Don't trigger when typing in inputs
      const target = e.target as HTMLElement;
      const isInput =
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable;

      if (e.metaKey || e.ctrlKey) {
        switch (e.key) {
          case "n":
            e.preventDefault();
            options.onQuickAdd?.();
            break;
          case "r":
            if (e.shiftKey) break; // Don't override browser refresh
            e.preventDefault();
            options.onReport?.();
            break;
          case "k":
            e.preventDefault();
            options.onSearch?.();
            break;
          case "s":
            if (options.onSave) {
              e.preventDefault();
              options.onSave();
            }
            break;
          case "1":
            e.preventDefault();
            navigate("/dashboard");
            break;
          case "2":
            e.preventDefault();
            navigate("/incidents");
            break;
          case "3":
            e.preventDefault();
            navigate("/reports");
            break;
          case "4":
            e.preventDefault();
            navigate("/settings");
            break;
        }
      }

      // "/" to focus search when not in input
      if (e.key === "/" && !isInput) {
        e.preventDefault();
        options.onSearch?.();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [navigate, options]);
}
