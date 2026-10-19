// Lint rules for the React/TypeScript code in `src/` and the Node scripts (`scripts/`, this file); `check.mjs`
// runs them. Formatting belongs to Prettier, and project rules that span Rust and TypeScript live in
// scripts/harness/.
import js from "@eslint/js";
import { defineConfig } from "eslint/config";
import reactHooks from "eslint-plugin-react-hooks";
import tseslint from "typescript-eslint";

// Node globals the scripts use; the `globals` package is not a dependency of this project.
const nodeGlobals = Object.fromEntries(
  ["process", "console", "URL", "setTimeout", "clearTimeout", "Buffer"].map((name) => [name, "readonly"]),
);

export default defineConfig(
  {
    files: ["scripts/**/*.mjs", "eslint.config.js"],
    extends: [js.configs.recommended],
    languageOptions: { globals: nodeGlobals },
    rules: { eqeqeq: ["error", "always", { null: "ignore" }] },
  },
  {
    files: ["src/**/*.{ts,tsx}"],
    extends: [js.configs.recommended, tseslint.configs.recommendedTypeChecked, reactHooks.configs.flat.recommended],
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    rules: {
      // Privacy: user text must never reach a log.
      "no-console": "error",
      // A command that is deliberately not awaited is marked with `void`.
      "@typescript-eslint/no-floating-promises": "error",
      "@typescript-eslint/no-misused-promises": "error",
      "@typescript-eslint/switch-exhaustiveness-check": "error",
      eqeqeq: ["error", "always", { null: "ignore" }],
    },
  },
  {
    files: ["src/**/*.test.{ts,tsx}", "src/test/**"],
    rules: {
      // Tauri commands reject with the serialized error value, so fakes do the same.
      "@typescript-eslint/prefer-promise-reject-errors": "off",
    },
  },
);
