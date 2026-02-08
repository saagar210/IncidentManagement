import MDEditor from "@uiw/react-md-editor";
import { useTheme } from "@/hooks/use-theme";

interface MarkdownViewerProps {
  content: string;
}

export function MarkdownViewer({ content }: MarkdownViewerProps) {
  const { resolvedTheme } = useTheme();

  if (!content.trim()) {
    return (
      <p className="text-sm text-muted-foreground italic">No content</p>
    );
  }

  return (
    <div data-color-mode={resolvedTheme === "dark" ? "dark" : "light"}>
      <MDEditor.Markdown source={content} />
    </div>
  );
}
