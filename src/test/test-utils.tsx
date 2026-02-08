import React, { type ReactElement } from "react";
import { render, type RenderOptions } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

export function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
      mutations: {
        retry: false,
      },
    },
  });
}

interface WrapperOptions {
  queryClient?: QueryClient;
  initialRoute?: string;
}

function createWrapper(options: WrapperOptions = {}) {
  const queryClient = options.queryClient ?? createTestQueryClient();
  const initialEntries = [options.initialRoute ?? "/"];

  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={initialEntries}>
          {children}
        </MemoryRouter>
      </QueryClientProvider>
    );
  };
}

export function renderWithProviders(
  ui: ReactElement,
  options: WrapperOptions & Omit<RenderOptions, "wrapper"> = {}
) {
  const { queryClient, initialRoute, ...renderOptions } = options;
  return render(ui, {
    wrapper: createWrapper({ queryClient, initialRoute }),
    ...renderOptions,
  });
}

/**
 * Creates a wrapper for testing hooks with renderHook.
 */
export function createHookWrapper(options: WrapperOptions = {}) {
  return createWrapper(options);
}
