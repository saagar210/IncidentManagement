import { describe, it, expect } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { setMockInvokeHandler } from "@/test/mocks/tauri";
import { createHookWrapper } from "@/test/test-utils";
import { useIncidents, useIncident, useSearchIncidents } from "./use-incidents";
import type { Incident } from "@/types/incident";

const mockIncident: Incident = {
  id: "inc-1",
  title: "Database outage",
  service_id: "svc-1",
  service_name: "AWS RDS",
  severity: "Critical",
  impact: "Critical",
  priority: "P0",
  status: "Resolved",
  started_at: "2025-06-01T10:00:00Z",
  detected_at: "2025-06-01T10:05:00Z",
  responded_at: "2025-06-01T10:10:00Z",
  resolved_at: "2025-06-01T11:00:00Z",
  duration_minutes: 60,
  root_cause: "Connection pool exhaustion",
  resolution: "Increased pool size",
  tickets_submitted: 15,
  affected_users: 200,
  is_recurring: false,
  recurrence_of: null,
  lessons_learned: "Monitor pool metrics",
  action_items: "",
  external_ref: "INC-2025-001",
  notes: "",
  created_at: "2025-06-01T10:00:00Z",
  updated_at: "2025-06-01T11:00:00Z",
};

describe("useIncidents", () => {
  it("fetches incidents with default empty filters", async () => {
    setMockInvokeHandler((cmd, args) => {
      if (cmd === "list_incidents") {
        expect(args?.filters).toEqual({});
        return [mockIncident];
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useIncidents(), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0].title).toBe("Database outage");
  });

  it("passes filters to the backend", async () => {
    const filters = { severity: "Critical", status: "Active" };

    setMockInvokeHandler((cmd, args) => {
      if (cmd === "list_incidents") {
        expect(args?.filters).toEqual(filters);
        return [];
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useIncidents(filters), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual([]);
  });
});

describe("useIncident", () => {
  it("fetches a single incident by ID", async () => {
    setMockInvokeHandler((cmd, args) => {
      if (cmd === "get_incident") {
        expect(args?.id).toBe("inc-1");
        return mockIncident;
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useIncident("inc-1"), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.title).toBe("Database outage");
  });

  it("does not fetch when id is undefined", async () => {
    let invoked = false;
    setMockInvokeHandler(() => {
      invoked = true;
      return mockIncident;
    });

    const { result } = renderHook(() => useIncident(undefined), {
      wrapper: createHookWrapper(),
    });

    // Should remain idle — never fires
    expect(result.current.isFetching).toBe(false);
    expect(invoked).toBe(false);
  });

  it("does not fetch when id is 'new'", async () => {
    let invoked = false;
    setMockInvokeHandler(() => {
      invoked = true;
      return mockIncident;
    });

    const { result } = renderHook(() => useIncident("new"), {
      wrapper: createHookWrapper(),
    });

    expect(result.current.isFetching).toBe(false);
    expect(invoked).toBe(false);
  });
});

describe("useSearchIncidents", () => {
  it("searches when query is non-empty", async () => {
    setMockInvokeHandler((cmd, args) => {
      if (cmd === "search_incidents") {
        expect(args?.query).toBe("database");
        return [mockIncident];
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useSearchIncidents("database"), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(1);
  });

  it("does not search when query is empty", async () => {
    let invoked = false;
    setMockInvokeHandler(() => {
      invoked = true;
      return [];
    });

    const { result } = renderHook(() => useSearchIncidents(""), {
      wrapper: createHookWrapper(),
    });

    expect(result.current.isFetching).toBe(false);
    expect(invoked).toBe(false);
  });
});
