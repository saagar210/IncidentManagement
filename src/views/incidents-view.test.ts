import { describe, expect, it } from "vitest";
import type { Incident } from "@/types/incident";
import { matchesIncidentSearch } from "./incidents-view";

function makeIncident(overrides: Partial<Incident> = {}): Incident {
  return {
    id: "inc-1",
    title: "Database outage",
    service_id: "svc-1",
    service_name: "Billing API",
    severity: "High",
    impact: "High",
    priority: "P1",
    status: "Active",
    started_at: "2026-01-01T00:00:00Z",
    detected_at: "2026-01-01T00:05:00Z",
    acknowledged_at: null,
    first_response_at: null,
    mitigation_started_at: null,
    responded_at: null,
    resolved_at: null,
    reopened_at: null,
    reopen_count: 0,
    duration_minutes: null,
    root_cause: "",
    resolution: "",
    tickets_submitted: 0,
    affected_users: 0,
    is_recurring: false,
    recurrence_of: null,
    lessons_learned: "",
    action_items: "",
    external_ref: "",
    notes: "",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

describe("matchesIncidentSearch", () => {
  it("does not throw when external_ref is missing", () => {
    const incident = makeIncident({
      external_ref: undefined as unknown as string,
    });
    expect(() => matchesIncidentSearch(incident, "database")).not.toThrow();
    expect(matchesIncidentSearch(incident, "database")).toBe(true);
  });

  it("matches by external reference when present", () => {
    const incident = makeIncident({ external_ref: "INC-2026-001" });
    expect(matchesIncidentSearch(incident, "2026-001")).toBe(true);
  });
});
