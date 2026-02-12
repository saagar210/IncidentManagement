import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { setMockInvokeHandler } from "@/test/mocks/tauri";
import { createHookWrapper } from "@/test/test-utils";
import {
  useGenerateReport,
  useSaveReport,
  useDiscussionPoints,
  useReportHistory,
  useDeleteReportHistory,
  useGenerateNarrative,
} from "./use-reports";
import type { DiscussionPoint, ReportHistoryEntry } from "@/types/reports";

const mockDiscussionPoints: DiscussionPoint[] = [
  {
    text: "Database experienced P0 incident affecting 500 users",
    trigger: "high_severity",
    severity: "P0",
  },
  {
    text: "API service had SLA breach of 2 hours",
    trigger: "sla_breach",
    severity: "P1",
  },
  {
    text: "Recurring network issues in us-west-2",
    trigger: "recurrence",
    severity: "P2",
  },
];

const mockReportHistoryEntry: ReportHistoryEntry = {
  id: "rpt-001",
  title: "Q1 2025 Incident Report",
  quarter_id: "q1-2025",
  format: "docx",
  file_path: "/home/user/.config/incident-manager/reports/incident_INC-001_2025-01-15.docx",
  file_size_bytes: 125000,
  config_json: '{"sections":["summary","timeline","rcause"]}',
  generated_at: "2025-01-15T10:30:00Z",
};

describe("useGenerateReport", () => {
  it("creates mutation without errors", () => {
    const { result } = renderHook(() => useGenerateReport(), {
      wrapper: createHookWrapper(),
    });

    expect(result.current.mutate).toBeDefined();
    expect(result.current.mutateAsync).toBeDefined();
  });
});

describe("useSaveReport", () => {
  it("creates mutation without errors", () => {
    const { result } = renderHook(() => useSaveReport(), {
      wrapper: createHookWrapper(),
    });

    expect(result.current.mutate).toBeDefined();
    expect(result.current.mutateAsync).toBeDefined();
  });
});

describe("useDiscussionPoints", () => {
  beforeEach(() => {
    setMockInvokeHandler((cmd) => {
      if (cmd === "generate_discussion_points") {
        return mockDiscussionPoints;
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });
  });

  it("generates discussion points for quarter", async () => {
    const { result } = renderHook(
      () => useDiscussionPoints("q1-2025"),
      { wrapper: createHookWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(3);
    expect(result.current.data![0].trigger).toBe("high_severity");
  });

  it("does not fetch when quarter is null", async () => {
    const { result } = renderHook(
      () => useDiscussionPoints(null),
      { wrapper: createHookWrapper() }
    );

    expect(result.current.status).toBe("pending");
  });

  it("does not fetch when quarter is empty string", async () => {
    const { result } = renderHook(
      () => useDiscussionPoints(""),
      { wrapper: createHookWrapper() }
    );

    expect(result.current.status).toBe("pending");
  });

  it("returns discussion points with trigger and severity", async () => {
    const { result } = renderHook(
      () => useDiscussionPoints("q1-2025"),
      { wrapper: createHookWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const points = result.current.data!;
    points.forEach((point) => {
      expect(point.text).toBeTruthy();
      expect(point.trigger).toBeTruthy();
      expect(point.severity).toMatch(/P[0-4]/);
    });
  });
});

describe("useReportHistory", () => {
  beforeEach(() => {
    setMockInvokeHandler((cmd) => {
      if (cmd === "list_report_history") {
        return [mockReportHistoryEntry];
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });
  });

  it("fetches report history", async () => {
    const { result } = renderHook(() => useReportHistory(), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0].title).toContain("Report");
  });

  it("returns empty array when no history", async () => {
    setMockInvokeHandler((cmd) => {
      if (cmd === "list_report_history") {
        return [];
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useReportHistory(), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toEqual([]);
  });
});

describe("useDeleteReportHistory", () => {
  it("creates mutation without errors", () => {
    const { result } = renderHook(() => useDeleteReportHistory(), {
      wrapper: createHookWrapper(),
    });

    expect(result.current.mutate).toBeDefined();
    expect(result.current.mutateAsync).toBeDefined();
  });
});

describe("useGenerateNarrative", () => {
  it("creates mutation without errors", () => {
    const { result } = renderHook(() => useGenerateNarrative(), {
      wrapper: createHookWrapper(),
    });

    expect(result.current.mutate).toBeDefined();
    expect(result.current.mutateAsync).toBeDefined();
  });
});
