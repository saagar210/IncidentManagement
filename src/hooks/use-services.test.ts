import { describe, it, expect, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { setMockInvokeHandler } from "@/test/mocks/tauri";
import { createHookWrapper, createTestQueryClient } from "@/test/test-utils";
import {
  useServices,
  useActiveServices,
  useAddServiceDependency,
  useRemoveServiceDependency,
} from "./use-services";
import type { Service } from "@/types/incident";

const mockServices: Service[] = [
  {
    id: "svc-1",
    name: "Slack",
    category: "Communication",
    default_severity: "High",
    default_impact: "High",
    description: "Team messaging",
    owner: "Platform Team",
    tier: "T1",
    runbook: "",
    is_active: true,
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
  },
  {
    id: "svc-2",
    name: "Legacy Tool",
    category: "Other",
    default_severity: "Low",
    default_impact: "Low",
    description: "Deprecated",
    owner: "",
    tier: "T4",
    runbook: "",
    is_active: false,
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
  },
];

describe("useServices", () => {
  it("fetches all services via list_services command", async () => {
    setMockInvokeHandler((cmd) => {
      if (cmd === "list_services") return mockServices;
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useServices(), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data).toHaveLength(2);
    expect(result.current.data![0].name).toBe("Slack");
    expect(result.current.data![1].is_active).toBe(false);
  });

  it("handles empty service list", async () => {
    setMockInvokeHandler((cmd) => {
      if (cmd === "list_services") return [];
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useServices(), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(0);
  });
});

describe("useActiveServices", () => {
  it("fetches only active services via list_active_services command", async () => {
    const activeOnly = mockServices.filter((s) => s.is_active);
    setMockInvokeHandler((cmd) => {
      if (cmd === "list_active_services") return activeOnly;
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useActiveServices(), {
      wrapper: createHookWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toHaveLength(1);
    expect(result.current.data![0].name).toBe("Slack");
  });
});

describe("service dependency mutations", () => {
  it("invalidates service-specific dependency caches after add", async () => {
    const queryClient = createTestQueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    setMockInvokeHandler((cmd) => {
      if (cmd === "add_service_dependency") {
        return {
          id: "sdep-1",
          service_id: "svc-1",
          depends_on_service_id: "svc-2",
          depends_on_service_name: "Legacy Tool",
          dependency_type: "runtime",
          created_at: "2026-01-01T00:00:00Z",
        };
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useAddServiceDependency(), {
      wrapper: createHookWrapper({ queryClient }),
    });

    await result.current.mutateAsync({
      service_id: "svc-1",
      depends_on_service_id: "svc-2",
      dependency_type: "runtime",
    });

    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["service-dependencies", "svc-1"],
    });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["service-dependents", "svc-2"],
    });
  });

  it("invalidates service-specific dependency caches after remove", async () => {
    const queryClient = createTestQueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    setMockInvokeHandler((cmd, args) => {
      if (cmd === "remove_service_dependency") {
        expect(args?.id).toBe("sdep-1");
        return undefined;
      }
      throw new Error(`Unexpected command: ${cmd}`);
    });

    const { result } = renderHook(() => useRemoveServiceDependency(), {
      wrapper: createHookWrapper({ queryClient }),
    });

    await result.current.mutateAsync({
      id: "sdep-1",
      serviceId: "svc-1",
      dependsOnServiceId: "svc-2",
    });

    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["service-dependencies", "svc-1"],
    });
    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ["service-dependents", "svc-2"],
    });
  });
});
