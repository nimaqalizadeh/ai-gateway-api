# AI Gateway — Project Structure & Rust Crates Reference

## Folder structure

```
ai-gateway/
├── src/
│   ├── main.rs              # server boot, signal handler
│   ├── config.rs            # envy + serde settings
│   ├── telemetry.rs         # tracing + OTEL init
│   ├── errors.rs            # AppError → HTTP status
│   ├── middleware.rs        # declares middleware submodules
│   ├── middleware/
│   │   ├── auth.rs          # JWT RS256 tower layer
│   │   ├── rate_limit.rs    # Redis sliding window
│   │   ├── request_id.rs
│   │   └── metrics.rs       # Prometheus counters
│   ├── routes.rs            # declares route submodules
│   ├── routes/
│   │   ├── health.rs        # readiness + liveness
│   │   ├── v1.rs            # declares v1 submodules
│   │   └── v1/
│   │       ├── chat.rs      # POST /v1/chat/completions
│   │       └── models.rs
│   ├── providers.rs         # Provider trait + submodule declarations
│   ├── providers/
│   │   ├── openai.rs
│   │   └── anthropic.rs
│   ├── cache.rs             # declares cache submodules
│   └── cache/
│       └── redis.rs         # cache-aside helpers
├── tests/
│   ├── integration/
│   └── contract/            # wiremock stubs
├── docs/
│   ├── roadmap.md
|   ├── project_structure/ # reference docs for this project
│   └── adr/
│       ├── 000-bootstrap.md
│       ├── 001-provider-trait.md
│       ├── 002-rate-limit-algo.md
│       └── 003-jwt-strategy.md
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml      # pins compiler version + components
├── rustfmt.toml
├── clippy.toml
├── justfile                 # task runner (fmt, lint, test, run)
├── .pre-commit-config.yaml
├── .gitignore
├── Dockerfile
└── docker-compose.yml
```

> Uses the modern (edition 2018+) module layout — a `foo.rs` file paired with a `foo/` folder, instead of `foo/mod.rs`. Fewer ambiguously-named tabs in your editor; module name matches file name.

## Core crates

| Crate                                     | Role · Roadmap skill                                                            |
| ----------------------------------------- | ------------------------------------------------------------------------------- |
| `axum`                                    | Router, extractors, SSE streaming — **framework**                               |
| `tower` / `tower-http`                    | Middleware layers, timeout, CORS, compression — **middleware #1**, **CORS #2**  |
| `jsonwebtoken`                            | JWT decode + RS256 public key validation — **auth #3**                          |
| `deadpool-redis`                          | Async Redis pool — rate limits + idempotency keys — **caching #9**              |
| `reqwest`                                 | Upstream HTTP client to LLM providers, stream proxying                          |
| `serde` / `serde_json`                    | Request/response serialisation                                                  |
| `envy`                                    | Env-var config into typed structs (replaces Pydantic Settings) — **config #12** |
| `tracing` + `tracing-subscriber`          | Structured JSON logs, per-request spans — **observability #11**                 |
| `opentelemetry` / `opentelemetry-otlp`    | Distributed tracing export to Jaeger/Tempo — **observability #11**              |
| `metrics` + `metrics-exporter-prometheus` | Prometheus counters/histograms — **benchmarking #6**                            |
| `tokio-retry` / `backoff`                 | Exponential backoff on provider failures — **resilience #17**                   |
| `utoipa` + `utoipa-swagger-ui`            | Auto-generated OpenAPI docs (replaces FastAPI's Swagger) — **API docs #14**     |
| `tokio` (full features)                   | Async runtime — replaces Python's asyncio                                       |

## Testing + tooling crates

| Crate / tool           | Role · Roadmap skill                                                                                               |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `axum-test`            | Spins up a real Axum router in-process — integration tests without a real server socket — **integration tests #5** |
| `wiremock-rs`          | Stubs upstream LLM provider HTTP calls in contract tests — **contract tests #5**                                   |
| `testcontainers-rs`    | Spins up a real Redis container for integration tests — **integration tests #5**                                   |
| `cargo-flamegraph`     | CPU profiling + flamegraph generation (replaces Scalene/perf) — **profiling #8**                                   |
| `k6` (external)        | Load testing perf-sensitive endpoints (same as roadmap) — **benchmarking #6**                                      |
| `clippy` + `rustfmt`   | Linting + formatting (replaces Ruff/Black/Isort)                                                                   |
| `cargo-audit`          | Dependency vulnerability scanning — **security basics #20**                                                        |
| `cargo-watch` + `just` | Dev auto-reload + task runner (replaces Makefile / pre-commit) — **automation**                                    |
