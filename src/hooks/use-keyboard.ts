import { useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";

interface KeyboardShortcutOptions {
  onQuickAdd?: () => void;
  onReport?: () => void;
  onSearch?: () => void;
  onSave?: () => void;
}

export function useKeyboardShortcuts(options: KeyboardShortcutOptions = {}) {
  const navigate = useNavigate();
  // Use ref to avoid re-attaching the listener on every render
  const optionsRef = useRef(options);
  optionsRef.current = options;

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
            optionsRef.current.onQuickAdd?.();
            break;
          case "k":
            e.preventDefault();
            optionsRef.current.onSearch?.();
            break;
          case "s":
            if (optionsRef.current.onSave) {
              e.preventDefault();
              optionsRef.current.onSave();
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
        optionsRef.current.onSearch?.();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [navigate]);
}
