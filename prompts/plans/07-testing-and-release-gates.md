# Testing, Performance, and Release Gates

## Required commands

Run from the workspace root:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --no-capture --test-threads=1
```

For a focused change, run the narrowest package/test first, then the full commands above. Keep the serialized test setting because multi-app network tests can otherwise become order-sensitive.

## Test layers

### Pure data and math

Use ordinary Rust tests for:

- Input normalization, stamina, aim, cooldown, damage, resource arithmetic.
- Deterministic RNG streams and stable IDs.
- Graph reachability, lock/key constraints, budget validation, extraction paths.
- Noise falloff, stimulus thresholds, alert transitions, objective transitions.
- Serialization round trips and protocol compatibility.

These tests should not require a Bevy renderer or network transport.

### Bevy system tests

Use `App` plus `MinimalPlugins` or the smallest plugin set for:

- State transitions and idempotence.
- Entity spawn/despawn and cleanup.
- Physics setup, colliders, grounding, and movement.
- Camera targeting and presentation-only systems.
- Terminal, inventory, health, extraction, and AI systems.
- Headless server startup and client startup.

### Local integration tests

Use the existing manual-time app builders for:

- One server and one to four local/crossbeam clients.
- Lobby -> Loading -> Playing.
- Replication and prediction/interpolation.
- Level seed agreement and dynamic entity replication.
- Complete success and failure runs.
- Disconnect, reconnect policy, late join policy, and duplicate requests.

### UDP smoke tests

Use real UDP for:

- Connection, authentication/configuration, join, replication, and clean shutdown.
- A short movement/objective/extraction smoke path.

Do not make UDP tests depend on timing sleeps when manual clocks or bounded polling are possible.

### Determinism and replay

Add a headless command or test helper that records:

- Generator schema/version and seed.
- Player inputs or accepted intents.
- Server tick and key state transitions.
- Objective, alert, damage, extraction, and final result events.

A seed and recorded input sequence should reproduce a failing pure-simulation case. Presentation and network packet ordering should not be required to reproduce generator failures.

## Required edge cases

- Zero players, one player, four players, and a fifth rejected player.
- Client disconnect in every session phase.
- Duplicate, delayed, stale, and out-of-order intent messages.
- Empty or invalid generation config.
- Failed asset load and headless startup.
- No valid navmesh path, blocked extraction, missing objective dependency.
- Enemy count at cap and no available target.
- Client prediction correction and remote interpolation with missing snapshots.
- Repeated state transitions, despawned targets, stale stable IDs, and cleanup followed by a new run.

## Performance budgets to measure

Do not promise numbers before measuring. Record baseline and target values for:

- Server fixed-step duration at 4 clients and representative enemy count.
- Level generation and validation duration.
- Navmesh build/update duration.
- Physics step duration.
- Replication bytes per second and correction frequency.
- Client frame time and draw calls in the representative facility.
- Peak entities and active noise/AI events.

Use bounded AI work, spatial partitioning where needed, and event expiry. Avoid `par_iter` as a substitute for a measured algorithm; parallelism is useful only after correctness and contention are understood.

## Release gate

A phase is releasable only when:

- The focused behavior tests pass.
- Workspace format, check, clippy, and serialized tests pass.
- A headless server and client can start without renderer-only resources.
- The happy path and at least one failure path are automated.
- Logs identify the seed, session, player, objective, and failure reason.
- Known limitations are written down and do not contradict the authority contract.

## Prompt

```text
Act as a test and release engineer for d:/Projects/game_test. Read the phase documents and inspect the existing launcher test harness before changing code.

For the feature just implemented, identify the narrowest behavior-scoped tests that can falsify its main hypothesis. Add focused pure tests, Bevy app tests, and multi-app integration tests where appropriate. Reuse manual time and existing local/crossbeam/UDP helpers. Cover duplicate requests, disconnects, stale IDs, invalid authority, cleanup, missing resources, and boundary values.

Then run:
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --no-capture --test-threads=1

If a command fails, fix only failures caused by the feature under test; report unrelated baseline failures separately. Add a deterministic seed/replay or debug summary when the feature is procedural or networked. Report test coverage, runtime/performance observations, and residual risk.
```
