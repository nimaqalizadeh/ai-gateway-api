# AI Gateway API — Axum Mentor Roadmap

## Context

You're building **AI Gateway API**: a production-grade reverse proxy in front of OpenAI and Anthropic. It streams completions back to callers, enforces per-key quotas via Redis, authenticates clients with RS256 JWTs, and exports OpenTelemetry traces. The repo today is an empty `cargo new` (`src/main.rs` prints hello).

This is a learning vehicle for the 11 skills listed in [overview.md](overview.md#L7-L19) and, opportunistically, for several of the broader roadmap skills in [backend_skills.md](backend_skills.md) that fall out naturally (testing, error handling, API docs, graceful shutdown, security, scalability).

Audience profile for this plan (your answers):

- **Rust level:** comfortable basics, new-ish to the tokio/tower/axum triangle — so each phase leads with the ecosystem concept before the code.
- **Execution mode:** roadmap-only — you'll work each phase on your own and come back with questions; nothing in this plan assumes I'm typing alongside you.
- **Testing rigor:** TDD-lite — tests land in the same phase as the feature, not a big catch-up pass at the end.

Estimate: 15–25 focused evenings to hit "production-grade end-to-end". The phases are ordered so every merge leaves the service runnable — you should never have a week where `cargo run` is broken.

---

## How to use this doc

Each phase is structured:

> **Goal** — what you'll have at the end.
> **Skills** — which roadmap items it lands.
> **Concepts to learn first** — read these before touching a keyboard.
> **Build steps** — terse checklist.
> **Tests (TDD-lite)** — written alongside, not after.
> **Done when** — objective verification.

Commit at the end of every phase. Tag phases you might want to revisit (`git tag phase-04-tracing`). Write the ADR **in the same PR** as the decision it documents — ADRs written after the fact are a lie.

---

## Phase 0 — Bootstrap & tooling

**Goal:** a clean, lintable, formatted workspace with CI-friendly tooling and an empty skeleton of the target folder structure.

**Skills:** prerequisites (rustfmt/clippy/pre-commit), #24 ADR (create the `docs/adr/` folder and write ADR `000-bootstrap.md`).

**Concepts to learn first:**

- Cargo workspaces vs single-crate layout — you're single-crate; don't over-engineer.
- `rust-toolchain.toml` to pin the compiler version; why reproducibility matters.
- `cargo-deny` vs `cargo-audit` — both are fine; `cargo-deny` is the superset.

**Build steps:**

1. Fix the crate name: `Cargo.toml` → `name = "ai-gateway"` (the `apai` typo will haunt you in logs otherwise). Bump `edition = "2024"` to `edition = "2021"` unless you have a reason — 2024 is fine but some of the async-fn-in-trait story is still settling.
2. Add `rust-toolchain.toml` pinning stable.
3. Create the folder layout from [project_structure.md](./project_structure.md) with empty `mod.rs` files so `cargo check` still passes.
4. Add `rustfmt.toml` (max_width, imports_granularity = "Crate") and `clippy.toml` (msrv).
5. Add a `justfile` with at minimum: `just fmt`, `just lint`, `just test`, `just run`, `just docker-build`.
6. Add a `.pre-commit-config.yaml` running `cargo fmt --check`, `cargo clippy -- -D warnings`, and `typos`.
7. Write `docs/adr/000-bootstrap.md` — template: Context / Decision / Consequences.
8. Create placeholder `Dockerfile` (empty `FROM scratch` is fine — it gets real in Phase 9) and `docker-compose.yml` stub declaring the future services: `gateway`, `redis`, `jaeger`, `prometheus`, `grafana`.

**Tests:** none yet — this phase is pure scaffolding.

**Done when:** `just lint && just test && cargo build` all green on a clean checkout; ADR 000 committed.

---

## Phase 1 — Minimal Axum server + error type + first test

**Goal:** a live `/ping` endpoint with a typed error type and an in-process integration test using `axum-test`.

**Skills:** #15 error handling, #5 integration tests, groundwork for #1.

**Concepts to learn first:**

- Axum handlers are plain async fns. Arguments are **extractors**; the return value is **anything that impls `IntoResponse`**. Read the "extractor" and "IntoResponse" chapters of the Axum docs — do not skim.
- `tower::Service<Request> → Response` is the abstraction Axum's router is built from. You don't need to implement `Service` yet, but know it exists.
- Why a single `AppError` enum is the idiomatic move in Rust: `?` propagation + one central `IntoResponse` impl = consistent error envelopes for free.

**Build steps:**

1. Add deps: `axum`, `tokio` (full), `tower`, `tower-http` (trace, timeout), `serde`, `serde_json`, `thiserror`, `anyhow` (only for main.rs).
2. `src/errors/mod.rs` — define `pub enum AppError` with variants `Internal(anyhow::Error)`, `BadRequest(String)`, `Unauthorized`, `RateLimited`, `Upstream(StatusCode, String)`, `NotFound`. `impl IntoResponse` mapping each to a JSON body `{"error": "...", "code": "..."}` and the right `StatusCode`. Add `pub type AppResult<T> = Result<T, AppError>`.
3. `src/routes/health.rs` — `pub async fn ping() -> &'static str { "pong" }`. Stub readiness/liveness; real probes come in Phase 8.
4. `src/main.rs` — build a `Router`, mount `/ping`, bind `0.0.0.0:8080`, `serve` with tokio.
5. Don't add middleware yet — keep this phase dumb.

**Tests (TDD-lite):**

- `tests/integration/health.rs` using `axum-test::TestServer`. Assert `GET /ping` returns 200 + `"pong"`.
- Unit test one `AppError` variant's `IntoResponse` to lock in the JSON shape.

**Done when:** `curl localhost:8080/ping` returns `pong`; `just test` green; `IntoResponse` covered by a test.

---

## Phase 2 — Config via envy + Pydantic-style settings

**Goal:** typed config loaded from env, no hard-coded values anywhere.

**Skills:** #12 config management.

**Concepts to learn first:**

- `envy` deserializes env vars into a struct via serde. Nested structs use prefixes — read `envy::prefixed`.
- `OnceCell`/`OnceLock` vs passing `Arc<Config>` through `axum::Extension` / `State`. Prefer `State` — it's the idiomatic path and keeps tests able to inject overrides.

**Build steps:**

1. `src/config.rs` — `#[derive(Debug, Deserialize, Clone)] pub struct Settings { pub server: ServerSettings, pub redis: RedisSettings, pub openai: ProviderSettings, pub anthropic: ProviderSettings, pub jwt: JwtSettings, pub otel: OtelSettings }`. Nested `...Settings` structs with sensible defaults via `#[serde(default)]`.
2. `Settings::from_env() -> Result<Self, envy::Error>` — called once in `main.rs`.
3. Wire `AppState { settings: Arc<Settings>, ... }` and pass with `Router::with_state`.
4. Add a `.env.example` with every key. Never commit `.env`.
5. Write ADR `004-config-strategy.md`: why envy over `config-rs` (simpler, env-only fits 12-factor, no YAML sprawl).

**Tests:** set env vars in a test, call `Settings::from_env()`, assert the struct. Use `temp_env::with_vars` to isolate.

**Done when:** removing any env var fails fast with a clear error; no magic strings remain in Phase 1 code.

---

## Phase 3 — Structured tracing (JSON logs + per-request spans)

**Goal:** every log line is JSON; every request produces one root span with a `request_id` field.

**Skills:** #11 observability (logging half), foundation for #1 middleware.

**Concepts to learn first:**

- `tracing` is not a logger — it's a structured diagnostics framework. `tracing-subscriber` turns events into output. Read "span vs event" until it clicks.
- `tower_http::trace::TraceLayer` auto-emits a span per request; you customize `.make_span_with` to add the request id once Phase 4 generates one.

**Build steps:**

1. Add deps: `tracing`, `tracing-subscriber` (env-filter, json features), `tower-http` already in.
2. `src/telemetry.rs` — `pub fn init(settings: &OtelSettings)` that installs a subscriber with a JSON layer + `EnvFilter::from_default_env()`. Return a guard if you adopt non-blocking later.
3. Call `telemetry::init()` as the **first line** of `main` (before any `tracing::info!`).
4. Mount `TraceLayer::new_for_http()` on the router. Verify with `RUST_LOG=info cargo run` and a `curl` — you should see a JSON log per request.

**Tests:** none directly — tracing is observed behavior. Add a smoke assertion in an integration test that `tracing_subscriber::fmt::TestWriter` captures at least one span for a request (optional, skip if it feels like testing the framework).

**Done when:** logs are valid JSON parseable by `jq`; each request emits a `span` containing method + uri + latency.

---

## Phase 4 — Request-ID middleware (your first tower::Layer)

**Goal:** every request gets a `x-request-id` header (honoring an incoming one if valid, generating one if not). The ID flows into the tracing span and the response header.

**Skills:** #1 middleware (the big one — **order of execution** is the theme), foundation for everything that follows.

**Concepts to learn first:**

- `tower::Layer` is a factory: `Layer::layer(inner_service) -> wrapped_service`. The wrapped service is what actually runs per request.
- Middleware order is **outermost-registered = outermost-executed**. `router.layer(A).layer(B)` means A wraps B wraps handler. Prove this to yourself with two `dbg!`-layers before moving on — this is roadmap #1's "critical" point.
- For anything stateless, prefer `axum::middleware::from_fn` — it's 10 lines; full `Layer`/`Service` is for when you need a builder with config.

**Build steps:**

1. `src/middleware/request_id.rs` — implement as `axum::middleware::from_fn`. Pull `x-request-id` header if present & UUID-shaped, else generate a `uuid::Uuid::new_v4()`. Attach to the request's extensions and to the response headers.
2. Update `TraceLayer` in `telemetry.rs` to pull request_id from extensions in `.make_span_with`.
3. Mount `request_id` **before** `TraceLayer` so the span has access to the ID.
4. Write ADR `005-middleware-order.md` documenting the intended stack order (auth → rate_limit → request_id → trace → handler — read outer-to-inner).

**Tests:** integration test — send request with no header, assert response has a UUID `x-request-id`. Send again with a custom ID, assert it round-trips.

**Done when:** you can `grep` a request_id across logs from entry to exit; ADR 005 merged.

---

## Phase 5 — Provider trait + OpenAI + Anthropic adapters (non-streaming first)

**Goal:** a `Provider` trait with two implementations. Routing by `model` field. No streaming yet — return a buffered `ChatCompletion`. This phase is where the ADR 001 decision lives.

**Skills:** trait design, foundation for streaming in Phase 6. Architecture skill #25 (lightly).

**Concepts to learn first:**

- `async fn` in traits is stable but using `Box<dyn Provider>` requires `#[async_trait]` or the `async-trait` crate for dyn compat. You need dyn dispatch because your router chooses at runtime — this is exactly what ADR 001 is about.
- `reqwest::Client` is cheap to clone but expensive to create — one per process, stored in state.

**Build steps:**

1. Add deps: `reqwest` (json, stream, rustls-tls), `async-trait`, `bytes`, `futures` (for `Stream`).
2. `src/providers/mod.rs`:
   ```rust
   #[async_trait]
   pub trait Provider: Send + Sync {
       fn supports(&self, model: &str) -> bool;
       async fn complete(&self, req: ChatRequest) -> AppResult<ChatResponse>;
       // add `complete_stream` in Phase 6
   }
   ```
3. `src/providers/openai.rs`, `src/providers/anthropic.rs` — implement the trait. Keep the request/response types in `providers/types.rs`; accept the OpenAI JSON shape as your gateway's canonical shape (it's the de-facto standard).
4. Route `POST /v1/chat/completions` in `src/routes/v1/chat.rs` — iterate `AppState.providers: Vec<Arc<dyn Provider>>`, pick the first that `.supports(model)`, return 400 if none.
5. Write ADR `001-provider-trait.md` — trait object vs enum dispatch, why extensibility wins even at the cost of one vtable indirection per request.

**Tests:**

- **Contract** test with `wiremock` for each provider: stub `/v1/chat/completions`, assert request body shape, return a canned response, assert the parsed `ChatResponse`.
- **Unit** test router selection: given `model: "gpt-4o"`, the OpenAI impl wins; given `claude-3-opus`, Anthropic wins; given garbage, 400.

**Done when:** two `wiremock`-backed contract tests pass; `POST /v1/chat/completions` works end-to-end against real OpenAI with a real key (manual smoke); ADR 001 merged.

---

## Phase 6 — Streaming proxy (the hardest phase)

**Goal:** `POST /v1/chat/completions` with `stream: true` pipes upstream SSE tokens back to the caller byte-for-byte, with no buffering.

**Skills:** advanced async/streams, core architectural skill for this project, #28 latency optimization.

**Concepts to learn first:**

- `futures::Stream` — read the signature and work through one hand-written impl. Understand `Pin<&mut Self>` and why it's there. You don't need to master it, but you should stop being afraid of it.
- `reqwest::Response::bytes_stream()` returns `impl Stream<Item = Result<Bytes>>`.
- Axum's `Body::from_stream` wraps any `Stream<Item = Result<Bytes, E>>` as a response body. For SSE specifically, `axum::response::sse::Sse::new(stream)` handles headers and keep-alives. **Decision point**: if upstream already sends SSE-formatted frames (OpenAI does), pass them through as a raw body — wrapping in `Sse` re-formats. If you want clean typed events, parse and re-emit via `Sse`. Start with raw pass-through; document the choice in a short note in `docs/adr/006-streaming-passthrough.md`.
- **Backpressure** is free here: `reqwest` reads from the socket as fast as axum writes to the client. Don't `.collect()` — that defeats the entire point.

**Build steps:**

1. Extend the `Provider` trait with `async fn complete_stream(&self, req) -> AppResult<BoxStream<'static, AppResult<Bytes>>>`.
2. Implement for OpenAI: call with `stream: true`, return `res.bytes_stream()` mapped to `AppResult<Bytes>`.
3. Same for Anthropic — note their SSE event format differs; either pass raw and document the caveat, or normalize. Pass raw for v1.
4. In the handler, branch on `request.stream`: buffered → existing path; streaming → return `Response::builder().header("content-type", "text/event-stream").body(Body::from_stream(stream))`.

**Tests:**

- Contract test with `wiremock` returning a chunked body; assert you receive ≥2 distinct chunks in the test client's stream (use `axum-test` + `.bytes_stream()` or a raw `hyper` client).
- Manually `curl -N` against the running gateway with a real key to eyeball latency.

**Done when:** first token visible in `curl` within ~300ms of upstream's first token; no buffering; ADR 006 merged.

---

## Phase 7 — JWT RS256 authentication

**Goal:** protect `/v1/*` routes with Bearer JWTs signed by an asymmetric key pair. Public key fetched from a JWKS URL, cached in memory with TTL.

**Skills:** #3 auth, #20 security basics, ADR 003.

**Concepts to learn first:**

- RS256 vs HS256: gateways should never share the signing secret — they only need the **public key** to verify. That's the entire argument of ADR 003.
- JWKS = JSON Web Key Set. Your auth provider (Auth0/Cognito/custom) publishes keys at `https://issuer/.well-known/jwks.json`. You fetch periodically, cache, and verify against the matching `kid`.

**Build steps:**

1. Add deps: `jsonwebtoken`, `moka` (for the JWKS cache with TTL), `serde`.
2. `src/middleware/auth.rs` — `axum::middleware::from_fn_with_state`. Extract `Authorization: Bearer` → decode header → look up `kid` in cached JWKS → verify → insert `Claims` into request extensions. 401 on any failure with a clear reason (don't leak internals).
3. `src/auth/jwks.rs` — `moka::future::Cache<String, DecodingKey>` with 10-minute TTL; miss fetches from JWKS URL.
4. Mount auth layer on the `v1` sub-router only. `/ping` and `/health/*` stay public.
5. Add a `Claims` extractor (`FromRequestParts`) so handlers can just write `claims: Claims` in their args.
6. Write ADR `003-jwt-strategy.md`.

**Tests:**

- Spin up a test JWKS server with `wiremock`, sign tokens in-test with `jsonwebtoken::encode`, assert valid → 200, expired → 401, bad sig → 401, missing header → 401.
- Verify the cache: count `wiremock` hits, assert second request doesn't re-fetch keys.

**Done when:** `/v1/chat/completions` returns 401 without a token; with a valid token it passes through; JWKS cache demonstrably reduces upstream calls.

---

## Phase 8 — Redis rate limiting (sliding window) + cache-aside

**Goal:** per-subject (JWT `sub` claim) sliding-window rate limit, enforced via Redis `ZADD`/`ZREMRANGEBYSCORE`/`ZCARD`. Plus a cache-aside helper for the models list.

**Skills:** #9 caching, implicit #2 rate limiting, ADR 002.

**Concepts to learn first:**

- Fixed window vs sliding window: fixed is one `INCR`+`EXPIRE`, sliding is a sorted set where scores are timestamps. Fixed has edge-bursts at window boundaries; sliding doesn't. ADR 002 picks sliding and justifies the memory cost.
- `deadpool-redis` pool sizing: start with `max_size = 2 × CPU cores`; tune after Phase 13 benchmarks (skill #19).
- Cache-aside: read cache → miss → read DB/upstream → write cache → return. Never write-through for a gateway — invalidation becomes a nightmare.

**Build steps:**

1. Add deps: `deadpool-redis`, `redis` (tokio-comp).
2. `src/cache/redis.rs` — pool setup, a `cache_aside<T>` generic helper with serde bounds.
3. `src/middleware/rate_limit.rs` — Layer (or `from_fn_with_state`). Key = `rl:{sub}:{route}`. Logic: `ZADD key now now` → `ZREMRANGEBYSCORE key 0 now-window` → `ZCARD key`. If count > limit, 429 with `Retry-After` header; else continue. Do this as an atomic Lua script — `MULTI/EXEC` isn't atomic in cluster mode.
4. Mount `rate_limit` **after** auth on the `v1` sub-router (auth must run first so rate keys are per-subject, not per-IP).
5. Extract limits from `Settings` — defaults of `60 req/min` per subject are fine to start.
6. Write ADR `002-rate-limit-algo.md`.

**Tests:**

- Integration test with `testcontainers-rs` spinning a real Redis. Send N+1 requests in a window; assert the N+1-th is 429. Send another after the window; assert 200.
- Unit test the Lua script's return shape with a direct `redis-cli` script, kept in `tests/fixtures/rate_limit.lua`.

**Done when:** `testcontainers` integration test is green; ADR 002 merged; `Retry-After` header present on 429s.

---

## Phase 9 — Metrics (Prometheus) + OpenTelemetry traces

**Goal:** `/metrics` endpoint exposes Prometheus counters + histograms; every request emits an OTLP span to Jaeger (via docker-compose).

**Skills:** #11 observability (the hard half), #6 benchmarking groundwork.

**Concepts to learn first:**

- `metrics` crate + `metrics-exporter-prometheus` is the lightweight path. `opentelemetry` is the heavier, standards-compliant path for traces. They coexist fine — you use `metrics` for counters and OTel for spans.
- Cardinality kills Prometheus: never include `request_id` or `user_id` as a label. Bucket by route template, method, status class.

**Build steps:**

1. Add deps: `metrics`, `metrics-exporter-prometheus`, `opentelemetry`, `opentelemetry-otlp`, `tracing-opentelemetry`.
2. Extend `telemetry.rs` to also install the OTel layer (`tracing_opentelemetry::layer()`) exporting OTLP to `settings.otel.endpoint`.
3. `src/middleware/metrics.rs` — record `http_requests_total{method,route,status}` counter and `http_request_duration_seconds{route}` histogram. Use the **matched route path**, not the raw URI (`axum::extract::MatchedPath`).
4. Add `/metrics` route returning the Prometheus-formatted text from the exporter handle.
5. Update `docker-compose.yml` to include Jaeger and Prometheus + a Grafana dashboard JSON under `ops/grafana/`.

**Tests:**

- Hit `/ping` in a test, then assert `/metrics` contains `http_requests_total{...} 1`.
- OTel is hard to unit-test — skip it; verify manually in Jaeger.

**Done when:** Grafana shows p50/p95/p99 for `/v1/chat/completions`; Jaeger shows a trace with `auth → rate_limit → provider.complete_stream` as nested spans.

---

## Phase 10 — Resilience: retries, timeouts, circuit breaker

**Goal:** upstream failures are retried with exponential backoff; requests have an end-to-end timeout; repeated upstream failure opens a circuit breaker.

**Skills:** #17 resilience.

**Concepts to learn first:**

- **Idempotency first, retries second.** Only retry on network errors or 5xx — never on 4xx. A retried non-idempotent POST can double-charge someone.
- `tokio-retry` gives you `Retry::spawn(strategy, operation)`. `tower::retry::Retry` is the Service-level version; use the former for upstream HTTP calls, the latter only if you want retry to compose with the rest of the stack.
- Circuit breaker: crates exist (`failsafe-rs`) but a hand-rolled `Arc<Mutex<State>>` with `Closed/Open/HalfOpen` is ~80 lines and teaches you more. For this project, build your own.

**Build steps:**

1. Add deps: `tokio-retry`.
2. Wrap the `reqwest` call in each provider's `complete`/`complete_stream` with retry: 3 attempts, exponential backoff (100ms base, 2x, jitter).
3. Add a `tower_http::timeout::TimeoutLayer` at the top of the stack: 60s for `/v1/chat/completions`, 5s for everything else (`Router::nest` two sub-routers).
4. `src/providers/circuit.rs` — simple breaker keyed per provider. Open after 5 consecutive failures in 30s; half-open after 10s.
5. Contract test with `wiremock` returning 503 twice then 200, assert one successful response.

**Tests:** contract + unit. The circuit breaker state machine is a textbook unit-test target.

**Done when:** chaos test (manually `docker pause` the upstream mock) shows the circuit opens, then recovers.

---

## Phase 11 — Real health checks (readiness vs liveness)

**Goal:** `/health/live` is a cheap "am I a process" probe; `/health/ready` actually pings Redis and returns 503 if it can't.

**Skills:** #18 health checks.

**Concepts to learn first:**

- **Liveness** = "should Kubernetes restart me?" — keep dumb, no deps.
- **Readiness** = "should traffic come to me?" — check every critical dep.
- Don't let readiness check upstream LLM providers — their outages aren't yours.

**Build steps:**

1. Split `health.rs` into `live()` (returns 200 always) and `ready()` (pings Redis via the pool, 200 or 503).
2. Neither is under auth. Both are under `TraceLayer` so you see them in logs, but exclude them from the Prometheus counter (use a path-aware filter) — otherwise readiness probes dominate your metrics.

**Tests:** integration tests for both; readiness test with `testcontainers` Redis (200) and with a pointed-to-nothing Redis URL (503).

**Done when:** k8s-style probes work; metrics aren't polluted by probe traffic.

---

## Phase 12 — Graceful shutdown

**Goal:** SIGTERM stops accepting new connections, waits up to 30s for in-flight requests to finish, drains the Redis pool, exits clean.

**Skills:** #16 graceful shutdown.

**Concepts to learn first:**

- `tokio::signal::ctrl_c()` + Axum's `.with_graceful_shutdown(fut)` is the two-line version. For multi-signal (SIGTERM on Linux), use `tokio::signal::unix::signal(SignalKind::terminate())`.
- `tokio_util::sync::CancellationToken` lets you signal _inside_ handlers too — important for long-running streams.

**Build steps:**

1. In `main.rs`, build a `CancellationToken`; spawn a signal listener that `token.cancel()`s on SIGTERM/SIGINT.
2. Pass a child token into state so streaming handlers can short-circuit new upstream calls once shutdown starts (don't kill in-flight streams — let them finish).
3. After `axum::serve(...).with_graceful_shutdown(token.cancelled())` returns, close the Redis pool (`pool.close()`) and the OTel provider (`shutdown_tracer_provider()`).

**Tests:** spawn the server in a test, send SIGTERM (via `libc::raise` or by awaiting the cancel token directly), assert `serve` returns within 30s with in-flight work complete. This is fiddly — a manual test via `docker stop` is acceptable if the unit version gets hairy.

**Done when:** `docker stop gateway` never loses an in-flight request; logs show clean shutdown order.

---

## Phase 13 — API versioning + utoipa docs

**Goal:** `/v1/*` is the only mounted version; adding `/v2` would be a drop-in sub-router. OpenAPI spec auto-generated; Swagger UI at `/docs`.

**Skills:** #23 versioning, #14 API docs.

**Concepts to learn first:**

- URL versioning (`/v1/chat`) vs header versioning (`Accept: application/vnd.api.v1+json`). URL wins for gateways — it's greppable, cacheable, easier to document. Write ADR `007-api-versioning.md`.
- `utoipa` derives OpenAPI from your handler signatures + Pydantic-style schema structs.

**Build steps:**

1. Add deps: `utoipa`, `utoipa-swagger-ui`.
2. Annotate request/response structs with `#[derive(ToSchema)]` and handlers with `#[utoipa::path(...)]`.
3. Mount `SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi())`.
4. Every `ToSchema` struct gets an `example` via `#[schema(example = ...)]`. Roadmap skill #14 explicitly requires example fields.

**Tests:** one integration test asserting `/api-doc/openapi.json` is valid JSON and contains every route.

**Done when:** `/docs` renders; every request/response shape has an example.

---

## Phase 14 — Multi-stage Dockerfile + full compose

**Goal:** a ~40MB image built in a multi-stage `Dockerfile` using `cargo-chef` for layer caching. `docker-compose up` brings up the whole stack (gateway + redis + jaeger + prometheus + grafana).

**Skills:** #7 containerization.

**Concepts to learn first:**

- `cargo-chef` splits the build into a "cook deps" stage that's cached separately from your source — the difference between 8-minute and 30-second rebuilds.
- `distroless/cc` or `gcr.io/distroless/static` runtime images — small, no shell, safer. If you're using `rustls` + pure Rust TLS, `distroless/static` works.

**Build steps:**

1. Stage 1 (`planner`): `cargo chef prepare --recipe-path recipe.json`.
2. Stage 2 (`cacher`): `cargo chef cook --release --recipe-path recipe.json`.
3. Stage 3 (`builder`): copy source + build the actual binary.
4. Stage 4 (`runtime`): `FROM gcr.io/distroless/cc-debian12`, copy binary, `USER nonroot`, `EXPOSE 8080`, `ENTRYPOINT ["/ai-gateway"]`.
5. `docker-compose.yml` — gateway, redis (7-alpine), jaeger, prometheus, grafana. Healthchecks on each. `depends_on` with `condition: service_healthy`.
6. Verify image size with `docker images`. If > 100MB, something's wrong — investigate.

**Tests:** a smoke script in `scripts/smoke.sh`: `docker compose up -d`, wait for readiness, curl `/ping`, curl `/metrics`, assert both 200, `docker compose down`.

**Done when:** `just docker-build && just compose-up` brings the whole stack up; image < 60MB.

---

## Phase 15 — Load testing (k6) + profiling (cargo-flamegraph)

**Goal:** you can quantify p50/p95/p99 for streaming and non-streaming flows; you've generated one flamegraph and identified at least one micro-optimization.

**Skills:** #6 benchmarking, #8 profiling, foundation for #19 connection pool tuning.

**Concepts to learn first:**

- k6 scripts are JS. Write one scenario for `POST /v1/chat/completions` with a `wiremock` upstream to remove external variance.
- `cargo-flamegraph` wraps `perf` (Linux) or `dtrace` (macOS). On macOS you need to disable SIP for `dtrace` — or run the flamegraph pass inside a Linux container.

**Build steps:**

1. `tests/load/chat.k6.js` — 100 VUs, 60s, ramp. Output JSON to `tests/load/results/`.
2. Add `just bench` and `just flamegraph` targets.
3. Run once, write `docs/perf/baseline.md` with p50/p95/p99 numbers. This is your "before" snapshot; revisit after Phase 16.

**Tests:** none — this phase produces artifacts, not code changes (unless you find a fix).

**Done when:** baseline.md committed; flamegraph PNG committed; at least one insight noted ("80% of time in serde_json — consider `sonic-rs`" is a valid insight; acting on it is optional).

---

## Phase 16 — Security + CI + final ADRs

**Goal:** `cargo audit` clean in CI; secrets never in images; at least 3 ADRs per major decision.

**Skills:** #20 security basics, #24 ADRs, implicit #5 testing in CI.

**Build steps:**

1. `cargo-audit` in CI (GitHub Actions or whatever you use). Block merges on `RUSTSEC-*` advisories.
2. `cargo-deny` config banning duplicate deps, yanked versions, forbidden licenses.
3. CI pipeline: `fmt --check → clippy -D warnings → test → audit → docker build`.
4. Review every `unwrap()` in the codebase. `clippy::unwrap_used` as a warn is a reasonable gate. In production paths, replace with explicit `?` + `AppError::Internal`.
5. Add CSP / HSTS headers via `tower_http::set_header` for any future web UI (low priority now; 2 lines to add).
6. Final ADR pass — make sure every "why" decision has an ADR entry. Minimum set: 001 provider trait, 002 rate-limit algo, 003 JWT RS256, 004 config strategy, 005 middleware order, 006 streaming passthrough, 007 API versioning, plus 000 bootstrap.

**Done when:** CI green end-to-end on a fresh clone; `cargo audit` clean; 8+ ADRs committed.

---

## End-to-end verification (how you know you're done)

Run this sequence on a clean checkout:

```bash
just compose-up             # gateway + redis + jaeger + prometheus + grafana
just test                   # all unit + integration + contract tests green
just bench                  # k6 hits target p99
curl -N -H "Authorization: Bearer $JWT" \
  -d '{"model":"gpt-4o","messages":[{"role":"user","content":"hi"}],"stream":true}' \
  localhost:8080/v1/chat/completions   # streams tokens in real time
curl localhost:8080/metrics | grep http_requests_total
open http://localhost:16686          # Jaeger shows the trace
open http://localhost:3000            # Grafana dashboard shows p50/p95/p99
docker stop $(docker ps -qf name=gateway)   # graceful shutdown, no request loss
```

Every one of those must work without surprises.

---

## Critical files you'll touch (the full map)

- [Cargo.toml](Cargo.toml) — fix the typo, add deps incrementally per phase; never `cargo add` everything on day one.
- [src/main.rs](src/main.rs) — grows slowly; stays under 100 lines by delegating to modules.
- [src/config.rs](src/config.rs) — Phase 2.
- [src/errors/mod.rs](src/errors/mod.rs) — Phase 1, extended every phase.
- [src/telemetry.rs](src/telemetry.rs) — Phases 3, 9, 12.
- [src/middleware/](src/middleware/) — auth (7), rate_limit (8), request_id (4), metrics (9).
- [src/providers/](src/providers/) — trait + openai/anthropic (5, 6, 10).
- [src/cache/redis.rs](src/cache/redis.rs) — Phase 8.
- [src/routes/v1/chat.rs](src/routes/v1/chat.rs) — Phases 5, 6.
- [src/routes/health.rs](src/routes/health.rs) — Phases 1, 11.
- [tests/integration/](tests/integration/) — every phase.
- [tests/contract/](tests/contract/) — Phases 5, 6, 10.
- [docs/adr/](docs/adr/) — one ADR per decision, written in the same PR.
- [Dockerfile](Dockerfile) — Phase 14.
- [docker-compose.yml](docker-compose.yml) — grows from Phase 0 stub to full stack by Phase 14.
- [justfile](justfile) — every phase adds targets.

---

## Ecosystem reading (in priority order)

Before Phase 1: the `axum` README + examples dir.
Before Phase 4: the `tower` "inventing your own Service" post (Tokio blog).
Before Phase 6: `futures::Stream` docs + the `async-stream` crate's README.
Before Phase 9: `tracing`'s "how-it-works" section and one OpenTelemetry spec skim.
Before Phase 14: the `cargo-chef` README.

---

## What this plan deliberately does NOT cover

- Multi-tenant quota billing / usage aggregation — out of scope, add a Phase 17 if you want it.
- A real control plane / admin UI — the gateway is data-plane only.
- Full gRPC or GraphQL variants — roadmap item #29, explicitly future work.
- Kafka / event sourcing — roadmap items #26, #27, future work.

If you decide to extend into any of these, open it as a new plan.
