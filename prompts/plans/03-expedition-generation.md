# Implementation Prompt: Deterministic Expedition Generation

```text
Work in d:/Projects/game_test. Read the phase 0 contract and roadmap. Inspect crates/shared/src/level/generation.rs, building.rs, prefabs.rs, terminal/sleeper/navigation modules, LevelSeed protocol registration, server level setup, client world creation, and generation/gameplay tests. Use the Bevy and Lightyear versions in Cargo.toml, not versions from old prompt files.

Goal: extend the existing seeded LevelGraph into a deterministic, solvable expedition generator that supports tension and replayability without requiring clients to receive a full generated map over the network.

Design the generator as pure data first, then provide Bevy spawning adapters. Keep the current primitive geometry builder as a valid test backend.

The generated mission must contain:
- A stable expedition seed and generator/schema version.
- A start zone and a reachable extraction zone.
- A main objective path with at least one locked dependency and a side branch containing the dependency.
- At least one optional objective or reward branch.
- Door/connection types: open, locked, alarm/security, and extraction gate.
- Stable IDs for zones, connections, doors, terminals, objectives, enemies, loot, resources, and extraction points.
- Deterministic encounter, resource, light, and terminal placement derived from independent seeded streams so adding one decoration does not reshuffle mission-critical content.
- A bounded threat/resource budget and an explicit difficulty profile.

Validation must reject or repair layouts with:
- Unreachable start, objective, dependency, or extraction.
- Overlapping rooms or invalid connection sockets.
- Unsafe player/enemy spawn positions.
- Missing navmesh coverage or impossible agent widths.
- An objective dependency cycle or a lock without a reachable key.
- Resource and enemy budgets outside configured bounds.
- A mission that has no viable retreat/extraction path.

Networking requirements:
- Server chooses and replicates the seed plus schema/config identity.
- Client regenerates only deterministic static content after validation; dynamic state is replicated by stable ID.
- Never use floating-point iteration order, HashMap order, or entity IDs as hidden randomness.
- Provide a debug export or compact summary that can reproduce a failing seed.

Tests required:
- Same seed yields byte-equivalent or structurally equivalent generated data.
- Different seeds produce variation without violating constraints.
- A corpus of at least 100 deterministic seeds validates.
- Every required objective and extraction is reachable.
- Stable IDs do not collide.
- Client/server generated summaries match.
- Failed generation is reported with a reason and does not transition to Playing.
- Geometry and navmesh adapters spawn expected markers and colliders exactly once.

Implement the smallest useful slice. Do not start with glTF socket authoring unless the existing data model cannot support the test backend. State the future asset-pipeline boundary separately.

Run focused generation tests immediately after editing, then formatting, workspace check, clippy, and the full serialized test command. Report timings and the seed of any failing case.
```
