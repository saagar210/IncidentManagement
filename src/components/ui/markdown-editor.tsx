import { useCallback } from "react";
import MDEditor from "@uiw/react-md-editor";
import { useTheme } from "@/hooks/use-theme";

interface MarkdownEditorProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  height?: number;
}

export function MarkdownEditor({
  value,
  onChange,
  placeholder,
  height = 200,
}: MarkdownEditorProps) {
  const { resolvedTheme } = useTheme();

  const handleChange = useCallback(
    (val: string | undefined) => {
      onChange(val ?? "");
    },
    [onChange]
  );

  return (
    <div data-color-mode={resolvedTheme === "dark" ? "dark" : "light"}>
      <MDEditor
        value={value}
        onChange={handleChange}
        height={height}
        preview="edit"
        textareaProps={{ placeholder }}
      />
    </div>
  );
}
