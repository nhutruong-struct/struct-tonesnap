---
paths:
  - "src/**/*.{ts,tsx}"
---

# TypeScript / React code rules

★ = enforced by `invariants.mjs`; (ESLint) = by `eslint.config.js`. The PostToolUse hook reports violations and runs Prettier after each edit.

- ★ No `any`, `@ts-ignore` or `@ts-nocheck`. `@ts-expect-error` needs a description.
- (ESLint) A promise deliberately not awaited is marked `void api.x()`; no `setState` directly in an effect body; complete hook dependencies; a `switch` over a union lists every case instead of `default`; no `console`.
- Formatting is Prettier's (120 columns); never format by hand.
- ★ `invoke` and `listen` only in `src/lib/api.ts`; components call its typed wrappers and `events` subscriptions. Keep each event name a literal in its `listen` call.
- State shared between windows goes through Rust events, not browser storage.
- UI behavior gets a vitest test next to the component, with the backend faked by `src/test/backend.ts` (fixtures in `src/test/fixtures.ts` match the Rust defaults).
