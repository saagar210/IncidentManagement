import { vi } from "vitest";

type InvokeHandler = (cmd: string, args?: Record<string, unknown>) => unknown;

let invokeHandler: InvokeHandler = () => {
  throw new Error("No mock invoke handler set. Call setMockInvokeHandler() first.");
};

export function setMockInvokeHandler(handler: InvokeHandler) {
  invokeHandler = handler;
}

export function resetMockInvokeHandler() {
  invokeHandler = () => {
    throw new Error("No mock invoke handler set.");
  };
}

/**
 * Creates a mock invoke handler from a command→response map.
 * Use this for simple cases where you just need static responses.
 */
export function createMockInvokeMap(
  commandMap: Record<string, unknown>
): InvokeHandler {
  return (cmd: string) => {
    if (cmd in commandMap) {
      return commandMap[cmd];
    }
    throw new Error(`Unmocked Tauri command: ${cmd}`);
  };
}

// Mock the @tauri-apps/api/core module
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    return Promise.resolve(invokeHandler(cmd, args));
  }),
}));

// Mock tauri plugin dialog
vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn(),
  open: vi.fn(),
  message: vi.fn(),
  ask: vi.fn(),
  confirm: vi.fn(),
}));

// Mock tauri plugin fs
vi.mock("@tauri-apps/plugin-fs", () => ({
  readTextFile: vi.fn(),
  writeTextFile: vi.fn(),
  exists: vi.fn(),
  copyFile: vi.fn(),
  rename: vi.fn(),
}));

// Mock tauri plugin opener
vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
  openPath: vi.fn(),
}));
