This is a tutorial project to improve my axum backend skills

Project Name: AI Gateway API

Description: A production-grade reverse proxy and rate-limiter for multiple LLM providers (OpenAI, Anthropic). Streams completions, enforces per-key quotas, logs usage, and exposes OpenTelemetry traces.

Skills covered:

- Middleware
- Auth / JWT RS256
- Rate limiting
- Redis caching
- Observability
- Resilience / retries
- Config (envy)
- Health checks
- Docker multi-stage
- API versioning
- ADRs

Here's the complete breakdown. I'll start with the architecture, then the folder structure, then the full crate manifest.

The gateway sits between any client and multiple upstream LLM providers. Every request passes through a tower of middleware layers before a single byte reaches OpenAI or Anthropic — this is the core architectural idea, and it maps perfectly to `tower`'s layered `Service` model.Now the folder structure and the full crate reference side by side:A few things worth highlighting that aren't obvious from the table:

**The `Provider` trait is the centrepiece of the design.** Define a `Provider` trait in `providers/mod.rs` that returns a `Stream<Item = Result<Bytes>>`. Both `openai.rs` and `anthropic.rs` implement it, and your router selects the right one based on the `model` field in the request body. This is your ADR #001 — document why you chose a trait object over an enum dispatch.

**`tower` is your middleware system, not a custom one.** Each layer in the stack (`auth`, `rate_limit`, `metrics`) implements `tower::Layer` and wraps the inner `Service`. The order they're added to the router _is_ the execution order — this directly maps to roadmap skill #1's note about "you must understand the order of execution."

**Streaming proxy is the hardest part.** Use `reqwest` with `.bytes_stream()` to get a `Stream` from upstream, then pipe it directly into an Axum `Sse` response or a raw `Body::from_stream`. Don't buffer — the whole point of SSE is low latency token delivery.

**For graceful shutdown** (skill #16), `tokio::signal::ctrl_c()` plus a `CancellationToken` lets you drain in-flight requests before the process exits. Axum's `.with_graceful_shutdown()` handles the rest.

**ADRs to write from day one:**

- `001-provider-trait.md` — trait object vs enum, why extensibility wins
- `002-rate-limit-algo.md` — sliding window vs fixed window, why Redis ZADD
- `003-jwt-strategy.md` — RS256 vs HS256, why asymmetric keys for a gateway
