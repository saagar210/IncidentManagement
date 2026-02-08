import "@testing-library/jest-dom/vitest";
import "./mocks/tauri";
import { resetMockInvokeHandler } from "./mocks/tauri";
import { afterEach } from "vitest";
import { cleanup } from "@testing-library/react";

// Clean up after each test
afterEach(() => {
  cleanup();
  resetMockInvokeHandler();
});
