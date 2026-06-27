## Go — Koda conventions

- Go 1.22+ features allowed
- Error handling: always check errors, never `_` on error returns except in tests
- Naming: short variable names in small scopes, descriptive in package-level funcs
- Goroutines: always handle lifecycle (context cancellation, WaitGroup)
- Formatting: `gofmt` enforced
- Tests: `testing` stdlib, table-driven tests preferred
- Logging: `slog` (stdlib) with structured key-value pairs
- Modules: `go.sum` committed, no indirect deps without explanation
