// Lint rules for the React/TypeScript code; `check.mjs` runs them on `src/`. Formatting belongs
// to Prettier, and project rules that span Rust and TypeScript live in scripts/harness/.
import js from "@eslint/js";
import { defineConfig } from "eslint/config";
import reactHooks from "eslint-plugin-react-hooks";
import tseslint from "typescript-eslint";

export default defineConfig(
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
