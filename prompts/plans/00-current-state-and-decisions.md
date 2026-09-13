# Phase 0: Current State and Product Decisions

## Purpose

Turn the broad inspiration list into a buildable contract. Read `00-game-design.md` first. This phase is documentation and small verification work only. It prevents later systems from disagreeing about authority, persistence, procedural prefab scope, loadouts, tension, or the meaning of a successful expedition.

## What is already good

- The crate split is appropriate for a networked game: shared protocol and simulation data, server authority, client presentation/input, launcher composition.
- Local, crossbeam, and UDP modes make fast deterministic integration tests possible.
- `LevelSeed` replication plus shared generation is the right bandwidth strategy.
- `Position`, `Rotation`, and `LinearVelocity` already use Lightyear prediction/interpolation registration.
- Existing end-to-end tests create multiple apps and manually advance time, which is the right foundation for network regression tests.
- The game already has the beginnings of the intended fantasy: procedural zones, sleepers, terminals, weapons, flashlight, and resource pressure ideas.

## Risks to resolve before feature expansion

- The written prompt files contain stale Bevy/Lightyear versions and should not be treated as API authority.
- Client and server lifecycle state machines can drift; define a shared session phase and explicit transition messages.
- `LevelSeed` is not enough to synchronize dynamic state. Define stable IDs for zones, doors, terminals, objectives, loot, enemies, and extraction points.
- Physics prediction and replicated physics components need a single ownership rule. Avoid attaching authoritative physics behavior to interpolated remote entities.
- Procedural generation must be validated for spawn reachability, collision bounds, navmesh viability, objective solvability, resource budget, and extraction reachability before entering `Playing`.
- RL and LLM crates are experiments, not dependencies of the first playable slice.

## Decisions for the first vertical slice

Use these defaults unless playtesting proves they are wrong:

- PvE only, 1-4 players, dedicated server supported, local mode required for tests.
- First-person action with server-authoritative movement outcomes and Lightyear prediction for the controlled player.
- One facility biome and one expedition objective chain.
- One main objective, one optional objective, one locked branch, one alarm/holdout, and one extraction route.
- Three enemy archetypes: dormant melee, ranged/spitter, and alerting scout.
- There are no player roles or classes. Players choose an individual loadout and items, with team coordination emerging from complementary equipment rather than mandatory role assignments.
- Procedural generation assembles authored room, corridor, door, encounter, and prop prefabs. Procedural terrain and voxel digging are explicitly out of scope for the core game.
- The desired emotional loop is uncertainty about the threat and consequences, combined with guaranteed objective reachability, readable telegraphs, and a viable extraction route.
- No destructible voxel terrain, PvP, persistent campaign map, voice chat, RL-driven AI, or LLM-driven gameplay in the first slice.
- A run is successful when all required objective state is complete and the squad reaches an extraction volume. A run fails when the server declares the expedition unrecoverable or the squad is eliminated.

## Required architecture contract

- Server owns session state, seed, procedural validation, objective state, enemy AI, damage, inventory/resource outcomes, extraction, and persistence.
- Client owns input sampling, local prediction, interpolation, camera, VFX, audio, HUD, and non-authoritative presentation.
- Shared owns serializable components/messages, pure generation, pure validation, gameplay constants, and systems safe to run on both sides.
- Every client request is treated as an intent. The server checks actor identity, phase, distance, line of sight, cooldown, inventory, and target validity before applying it.
- Every replicated entity that matters to gameplay has a stable network identity or server-owned ID. Never use an entity index as a persistent gameplay ID.

## Definition of done

A checked-in design note records:

- The state transition diagram.
- The authoritative owner of every major component.
- The minimum network messages and replication rules.
- The first vertical-slice mission contract.
- The explicit non-goals listed above.

## Prompt

```text
Read prompts/plans/00-current-state-and-decisions.md and inspect the existing repository before editing anything. Compare the design contract to Cargo.toml, crates/client, crates/server, crates/shared, crates/launcher/src/tests, README.md, TODO.md, and Architecture.mermaid.

Produce a short audit that identifies contradictions between the contract and current code. Then update only the minimum documentation or small type-level scaffolding needed to make these decisions explicit. Do not implement a feature system yet.

Create or update a state/authority matrix covering client state, server state, seed ownership, entity ownership, prediction, interpolation, objectives, damage, inventory, extraction, and disconnect handling. Record any decision that cannot be verified from code as an open question rather than inventing an API.

Validate with cargo fmt --all -- --check and cargo check --workspace. Report files changed, commands run, and unresolved risks.
```
