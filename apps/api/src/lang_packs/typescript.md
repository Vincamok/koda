## TypeScript — Koda conventions

- Strict mode required (`"strict": true` in tsconfig)
- No `any` — use `unknown` + type narrowing
- Prefer `interface` over `type` for object shapes
- Async: always `async/await`, no raw Promise chains
- Error handling: never swallow errors silently; log or rethrow
- Imports: named imports only; no default re-exports from index files
- No `var` — `const` by default, `let` only when reassigned
- Formatting: Prettier enforced, 2-space indent, single quotes
- Testing: Vitest for unit tests, Playwright for E2E
