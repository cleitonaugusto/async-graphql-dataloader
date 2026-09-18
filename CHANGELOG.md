# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Announced

- **Licensing change at 0.3.0.** From 0.3.0 this project will be dual-licensed
  under AGPL-3.0-or-later or a paid commercial license. Version 0.2.0 and
  everything before it stay under MIT OR Apache-2.0 permanently — that grant is
  irrevocable and will not be withdrawn. The 0.1.0 dispatch fix therefore ships
  free, under the permissive license, before the change takes effect. See
  `LICENSING.md` and `COMMERCIAL-LICENSE.md`.
- Contributions now require a sign-off (`git commit -s`) granting the
  maintainer the right to license them commercially. See `CONTRIBUTING.md`.

## [0.2.0] - 2026-09-15

### Fixed

- **`DataLoader::load` never returned.** `Batcher::clone` built a fresh, empty
  `pending` queue and `batch_task` slot instead of sharing them, and the
  dispatch task ran against that clone. It therefore drained an empty queue
  while the real `oneshot::Sender`s stayed parked in the original one, so the
  `oneshot::Receiver` never resolved and every `load` awaited forever. This
  affected 0.1.0 in every configuration — batching, and consequently the whole
  crate, did not work. `pending` and `batch_task` are now `Arc`-shared across
  clones.
- A full batch no longer waits for the delay window. `schedule` dispatches
  immediately once `max_batch_size` keys are queued; previously the flush was a
  no-op whenever a timer task already existed, so `max_batch_size` only ever
  capped batch size instead of triggering a dispatch.
- Queues larger than `max_batch_size` are now drained completely. The dispatch
  task loops until the queue is empty; previously the leftover keys waited for
  an unrelated request to re-arm the timer.
- The crate did not compile: `lib.rs` declared the `metrics`, `rate_limiting`
  and `query_cost` modules, but none of the three source files were present.
- README documented features that do not exist in this crate (distributed
  caching, multi-tenant isolation, LGPD/GDPR compliance tooling, audit trails,
  real-time dashboard). The feature list now describes what actually ships.
- `README.pt-BR.md` documented a `Loader<K>` trait that no longer exists; the
  current trait is `BatchLoad`.

### Added

- An integration test suite (`tests/batching.rs`) covering dispatch, batch
  coalescing, caching and `max_batch_size` splitting. Each test bounds its
  `load` in a timeout so a dispatch regression fails the run instead of
  hanging it.
- `RateLimiter` — fixed-window rate limiting, opt-in per loader via
  `DataLoader::with_rate_limiting`.
- `QueryCostAnalyzer` — rule-based query cost estimation, opt-in via
  `DataLoader::with_query_cost_analysis`.
- `TelemetryCollector` and `DataLoaderMetrics` — batch and cache counters.
- Crate-level documentation with runnable doctests.
- This changelog.

### Changed

- **Breaking:** `license` is now `MIT OR Apache-2.0`, matching the `LICENSE-MIT`
  and `LICENSE-APACHE` files that shipped since 0.1.0. Previously the manifest
  declared only `MIT`.
- `DataLoaderError` gained the `RateLimitExceeded` and `QueryCostExceeded`
  variants. Exhaustive matches on this enum need updating.
- `RateLimitError` is now publicly exported.

### Removed

- **Breaking:** the `enterprise` cargo feature and the `enterprise` module.
  The module re-exported types that are already public at the crate root, and
  its `EnterpriseDataLoaderBuilder::with_config` silently discarded the
  configuration it was given at `build()` time. Use `DataLoader` directly.
- **Breaking:** the `enterprise_loader!` macro, for the same reason.
- The unused optional dependencies `redis`, `warp`, `tokio-stream`, `uuid`,
  `thiserror` and `chrono`. None were referenced anywhere in `src/`, but the
  `enterprise` feature pulled all of them in — including a full web framework.

## [0.1.0] - 2025-10-02

### Added

- Initial release: `DataLoader` with automatic batching, `Cache` with optional
  TTL, the `BatchLoad` trait, `sqlx` integration and the optional `graphql`
  feature for `async-graphql`.

> **Note:** `DataLoader::load` never resolves in this version. See the 0.2.0
> entry above. Use 0.2.0 or later.

[0.2.0]: https://github.com/cleitonaugusto/async-graphql-dataloader/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/cleitonaugusto/async-graphql-dataloader/releases/tag/v0.1.0
