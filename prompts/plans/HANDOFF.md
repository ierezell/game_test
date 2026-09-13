# Implementation Handoff

Date: 2026-09-13
Workspace: `d:/Projects/game_test`

## Mission

Build a 1-4 player first-person cooperative extraction-horror game inspired by GTFO, Species: Unknown, Deep Rock Galactic, Alien Swarm: Reactive Drop, and Aliens: Dark Descent.

Hard design constraints:

- No player classes or fixed roles.
- Players choose flexible loadouts and items.
- No procedural terrain or voxel digging.
- Procedural facilities are assembled from authored room, corridor, door, encounter, and prop prefabs.
- Uncertainty and tension are important, but objectives and extraction routes must remain reachable and fair.
- Server owns mission state, objectives, AI, damage, inventory outcomes, extraction, and rewards.
- Clients own input, prediction, interpolation, camera, UI, audio, VFX, and presentation.

Read first:

1. `prompts/plans/00-game-design.md`
2. `prompts/plans/00-current-state-and-decisions.md`
3. `prompts/plans/01-roadmap.md`
4. `prompts/plans/07-testing-and-release-gates.md`

## Completed and verified in this session

### Planning

- Created the `prompts/plans/` roadmap and implementation prompt pack.
- Added the full design contract in `prompts/plans/00-game-design.md`.
- Corrected the reference links for Species: Unknown and Alien Swarm: Reactive Drop.
- Removed player role/class assumptions from the plan.
- Documented prefab-only procedural generation and fair uncertainty.

### Session and multiplayer

- Server lobby start requests are authorized against `LobbyState.host_id` in `crates/server/src/lobby.rs`.
- Host, non-host, and `requested: false` start-request tests pass.
- Existing two-client app-flow suite passes all 11 tests, covering connection, lobby, replication, start, and disconnect behavior.

### Player input and physics ownership

- CCC tests use Enhanced Input `ActionMock` for deterministic injected actions in `crates/launcher/src/tests/ccc.rs`.
- Local movement/look/camera tests pass.
- Interpolated remote players and NPCs no longer receive dynamic client physics in `crates/client/src/entities.rs`.
- Added a regression test proving interpolated players receive visuals but no `RigidBody`.

### Interaction and terminals

- Added `TerminalInteractionRequest` to the shared Lightyear protocol in `crates/shared/src/protocol.rs`.
- Added client terminal plugin in `crates/client/src/terminal.rs`.
- Pressing `E` selects the nearest terminal within range and sends a typed request.
- Added shared validation for session phase, finite positions, range, and cooldown in `crates/shared/src/terminal.rs`.
- Server resolves the Lightyear sender to its controlled player, finds the stable terminal ID, validates range, and executes commands authoritatively in `crates/server/src/lobby.rs`.
- `UNLOCK` now requires an indexed `IndexedItemKind::Keycard`, preventing forged keycard IDs.
- Terminal tests: 29 passed. Server terminal/lobby authority tests: 5 passed. Client nearest-terminal tests: 2 passed.

### Procedural generation

- Existing generation has same-seed determinism and GTFO-style expedition design tests.
- Added a 100-seed corpus regression in `crates/shared/src/level/generation.rs` checking objective existence, spawn-to-objective reachability, and terminal objective indexing.
- Full generation test module: 18 passed.

### Stealth foundation

- Existing noise and sleeper systems cover noise types, attenuation, state transitions, scream propagation, and target selection.
- Added a cyclic zone-graph noise propagation regression in `crates/shared/src/noise.rs`.
- It verifies propagation terminates, attenuates, reaches the cycle, and stays finite/bounded.
- Noise tests: 11 passed. Sleeper tests: 21 passed.

## Validation results

Passed recently:

```text
cargo test -p client --lib -- --no-capture --test-threads=1
cargo test -p launcher --lib tests::ccc -- --no-capture --test-threads=1
cargo test -p launcher --lib tests::app_flow -- --no-capture --test-threads=1
cargo test -p shared level::generation::tests -- --no-capture --test-threads=1
cargo test -p shared noise::tests -- --no-capture --test-threads=1
cargo test -p shared sleeper::tests -- --no-capture --test-threads=1
cargo test -p shared terminal::tests -- --no-capture --test-threads=1
cargo test -p server lobby::tests -- --no-capture --test-threads=1
cargo check -p launcher
```

Known workspace-level issues:

- `cargo check --workspace` passes, with an existing unused `With` import warning in `crates/client/src/inputs/mod.rs` and dependency future-incompatibility warnings.
- `cargo clippy --workspace --all-targets -- -D warnings` currently fails on broad pre-existing lint debt in `shared` and test targets. Do not mass-refactor unrelated code during feature work.
- The full workspace test run previously passed 60 tests but failed three existing CCC movement tests before the `ActionMock` fix. Those three now pass in the focused suite; rerun the full workspace suite later to refresh the global baseline.
- Lightyear/Bevy tests emit repeated global logger/metrics recorder warnings when multiple apps are created. They do not currently fail the focused suites.

## Important worktree warning

`git status` is already dirty across many source files and prompt files, including user or automation changes. Do not use `git reset`, `git checkout`, or broad formatting rewrites. Inspect current file contents before editing. Preserve unrelated changes.

## Recommended next tasks

1. **Finish Phase 1 lifecycle semantics:** add explicit session phase/reason data, make loading/playing transitions idempotent, decide and test late-join behavior in every phase, and add four-client coverage.
2. **Finish Phase 2 interaction security:** add line-of-sight validation using the current Avian spatial query, make terminal cooldown authoritative instead of passing `true`, and add a client response/HUD path for rejected or successful commands.
3. **Finish Phase 2 movement:** add a real reconciliation/latency regression with the existing Lightyear test stepper. Keep predicted local physics and interpolated remote presentation separate.
4. **Advance the expedition loop:** objective prerequisites, extraction prerequisites, resource/inventory authority, and alarm/holdout state need a complete server-owned state machine.
5. **Keep procedural scope bounded:** extend authored prefab selection and validation, not terrain generation.
6. **Add full-run validation:** rerun `cargo test --workspace -- --no-capture --test-threads=1`, then record the current complete baseline and investigate only failures relevant to the next slice.

## Suggested first prompt for the next model

```text
Continue from prompts/plans/HANDOFF.md. Inspect current files before editing because the worktree is dirty. Implement the next smallest Phase 2 interaction-security slice: authoritative terminal cooldown and Avian line-of-sight validation for TerminalInteractionRequest. Reuse the existing shared validation, server handler, Lightyear protocol, and tests. Add focused pure/ECS tests for valid use, through-wall rejection, cooldown rejection, stale target, and duplicate request. Run the focused tests immediately, then the two-client app-flow suite and cargo check -p launcher. Do not refactor unrelated lint debt or reset existing changes.
```
