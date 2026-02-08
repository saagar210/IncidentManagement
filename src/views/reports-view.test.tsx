import { describe, expect, it } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import { setMockInvokeHandler } from "@/test/mocks/tauri";
import { renderWithProviders } from "@/test/test-utils";
import { ReportsView } from "./reports-view";

describe("ReportsView", () => {
  it("shows 0 B for zero-byte report history entries", async () => {
    setMockInvokeHandler((cmd) => {
      if (cmd === "get_quarter_configs") return [];
      if (cmd === "list_report_history") {
        return [
          {
            id: "rep-1",
            title: "Zero Byte Report",
            quarter_id: null,
            format: "docx",
            generated_at: "2026-01-01T00:00:00Z",
            file_path: "/tmp/report.docx",
            config_json: "{}",
            file_size_bytes: 0,
          },
        ];
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    renderWithProviders(<ReportsView />);

    await waitFor(() =>
      expect(screen.getByText("Zero Byte Report")).toBeInTheDocument()
    );
    expect(screen.getByText("0 B")).toBeInTheDocument();
  });
});
