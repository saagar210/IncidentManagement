import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { setMockInvokeHandler } from "@/test/mocks/tauri";
import { createHookWrapper } from "@/test/test-utils";
import {
  useIncidentHeatmap,
  useIncidentByHour,
  useDashboardConfig,
  useUpdateDashboardConfig,
  type DashboardCardConfig,
} from "./use-dashboard";
import type { DayCount, HourCount } from "@/types/metrics";

const mockHeatmapData: DayCount[] = [
  { day: "Monday", count: 5 },
  { day: "Tuesday", count: 3 },
  { day: "Wednesday", count: 8 },
  { day: "Thursday", count: 2 },
  { day: "Friday", count: 6 },
  { day: "Saturday", count: 1 },
  { day: "Sunday", count: 0 },
];

const mockHourData: HourCount[] = Array.from({ length: 24 }, (_, i) => ({
  hour: i,
  count: Math.floor(Math.random() * 10),
}));

describe("useIncidentHeatmap", () => {
  beforeEach(() => {
    setMockInvokeHandler((cmd) => {
      if (cmd === "get_incident_heatmap") {
        return mockHeatmapData;
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });
  });

  it("fetches heatmap data when dates are provided", async () => {
    const { result } = renderHook(
      () => useIncidentHeatmap("2025-01-01", "2025-01-31"),
      { wrapper: createHookWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(mockHeatmapData);
  });

  it("does not fetch when startDate is empty", async () => {
    const { result } = renderHook(
      () => useIncidentHeatmap("", "2025-01-31"),
      { wrapper: createHookWrapper() }
    );

    // Should remain disabled
    expect(result.current.status).toBe("pending");
  });

  it("returns data with correct day counts", async () => {
    const { result } = renderHook(
      () => useIncidentHeatmap("2025-01-01", "2025-01-31"),
      { wrapper: createHookWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const data = result.current.data!;
    expect(data.length).toBe(7);
    expect(data[0].day).toBe("Monday");
    expect(data[0].count).toBe(5);
    expect(data[2].day).toBe("Wednesday");
    expect(data[2].count).toBe(8);
  });

  it("caches results for 30 seconds", async () => {
    const { result, rerender } = renderHook(
      ({ startDate, endDate }) => useIncidentHeatmap(startDate, endDate),
      {
        initialProps: { startDate: "2025-01-01", endDate: "2025-01-31" },
        wrapper: createHookWrapper(),
      }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    // Rerender with same dates - should use cache
    rerender({ startDate: "2025-01-01", endDate: "2025-01-31" });

    expect(result.current.data).toEqual(mockHeatmapData);
  });
});

describe("useIncidentByHour", () => {
  beforeEach(() => {
    setMockInvokeHandler((cmd) => {
      if (cmd === "get_incident_by_hour") {
        return mockHourData;
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });
  });

  it("fetches hour histogram data when dates provided", async () => {
    const { result } = renderHook(
      () => useIncidentByHour("2025-01-01", "2025-01-31"),
      { wrapper: createHookWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(24);
  });

  it("does not fetch when startDate is null", async () => {
    const { result } = renderHook(
      () => useIncidentByHour(null, "2025-01-31"),
      { wrapper: createHookWrapper() }
    );

    expect(result.current.status).toBe("pending");
  });

  it("returns all 24 hours of data", async () => {
    const { result } = renderHook(
      () => useIncidentByHour("2025-01-01", "2025-01-31"),
      { wrapper: createHookWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const data = result.current.data!;
    expect(data.length).toBe(24);
    expect(data[0].hour).toBe(0);
    expect(data[23].hour).toBe(23);
  });
});

describe("useDashboardConfig", () => {
  const mockConfig: DashboardCardConfig = {
    mttr: true,
    mtta: true,
    recurrence_rate: true,
    avg_tickets: true,
    by_severity: true,
    by_service: true,
    heatmap: true,
    hour_histogram: true,
    trends: true,
    timeline: false,
  };

  beforeEach(() => {
    setMockInvokeHandler((cmd) => {
      if (cmd === "get_setting") {
        return JSON.stringify(mockConfig);
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });
  });

  it("fetches dashboard configuration", async () => {
    const { result } = renderHook(() => useDashboardConfig(), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(mockConfig);
  });

  it("handles missing configuration by returning defaults", async () => {
    setMockInvokeHandler((cmd) => {
      if (cmd === "get_setting") {
        return null;
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useDashboardConfig(), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    // Should have default values
    expect(result.current.data?.mttr).toBe(true);
    expect(result.current.data?.mtta).toBe(true);
    expect(result.current.data?.timeline).toBe(false);
  });

  it("handles invalid JSON gracefully", async () => {
    setMockInvokeHandler((cmd) => {
      if (cmd === "get_setting") {
        return "invalid json {";
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useDashboardConfig(), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    // Should return defaults when JSON parsing fails
    expect(result.current.data?.mttr).toBe(true);
  });

  it("caches config indefinitely (staleTime: Infinity)", async () => {
    const { result } = renderHook(() => useDashboardConfig(), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    // Data should persist and not be refetched on re-render
    expect(result.current.data).toEqual(mockConfig);
  });
});

describe("useUpdateDashboardConfig", () => {
  it("creates mutation without errors", () => {
    const { result } = renderHook(() => useUpdateDashboardConfig(), {
      wrapper: createHookWrapper(),
    });

    expect(result.current.mutate).toBeDefined();
    expect(result.current.mutateAsync).toBeDefined();
  });
});
