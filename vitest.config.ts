import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Frontend tests. The Rust backend is faked with Tauri's own mocks (see src/test/backend.ts).
export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
    restoreMocks: true,
  },
});
