# Prompt For The Next Model

You are continuing work in the existing Rust workspace at `d:/Projects/game_test`.

Read these files first:

1. `prompts/plans/HANDOFF.md`
2. `prompts/plans/00-game-design.md`
3. `prompts/plans/00-current-state-and-decisions.md`
4. `prompts/plans/01-roadmap.md`
5. `prompts/plans/07-testing-and-release-gates.md`

The worktree is intentionally dirty and contains user or automation changes. Inspect current file contents before editing. Do not use `git reset`, `git checkout`, broad rewrites, or mass formatting. Preserve unrelated changes.

## Current project direction

This is a 1-4 player first-person cooperative extraction-horror game inspired by GTFO, Species: Unknown, Deep Rock Galactic, Alien Swarm: Reactive Drop, and Aliens: Dark Descent.

Hard constraints:

- No player classes or fixed roles.
- Players choose flexible loadouts and items.
- No procedural terrain or voxel digging.
- Generate facilities from authored room, corridor, door, encounter, and prop prefabs.
- Keep objectives and extraction routes reachable and fair while preserving uncertainty and tension.
- Server owns mission state, objectives, AI, damage, inventory outcomes, extraction, and rewards.
- Clients own input, local prediction, interpolation, camera, UI, audio, VFX, and presentation.
- Use the Bevy and Lightyear versions already declared in `Cargo.toml`; inspect current APIs instead of copying newer examples.

## Completed baseline

The repository already has:

- Client, server, shared, launcher, local, crossbeam, and UDP modes.
- Lightyear replication, prediction, interpolation, and typed messages.
- Lobby and game lifecycle tests with two clients.
- Seeded procedural zone generation and prefab-style geometry.
- Terminal interaction protocol and client `E` input.
- Shared terminal validation and server-side sender/target/range checks.
- Noise and sleeper systems.
- Client physics ownership fixed so interpolated remote players/NPCs are presentation-only.

Relevant files:

- `crates/shared/src/protocol.rs`
- `crates/shared/src/terminal.rs`
- `crates/server/src/lobby.rs`
- `crates/client/src/terminal.rs`
- `crates/client/src/entities.rs`
- `crates/launcher/src/tests/ccc.rs`
- `crates/launcher/src/tests/app_flow.rs`

## Task: next implementation slice

Implement authoritative terminal interaction security:

1. Add a real server-side cooldown for `TerminalInteractionRequest`.
2. Add Avian line-of-sight validation between the controlled player and the target terminal.
3. Reject stale or duplicate requests without mutating terminal state twice.
4. Preserve existing stable terminal ID, sender ownership, session-phase, finite-position, and range validation.
5. Keep the client request path unchanged unless a small change is required to display a server rejection or success result.

## Required authority behavior

- The client sends intent only: terminal ID and command.
- The server derives actor identity from the Lightyear `ClientOf` connection.
- The server resolves the controlled player entity and its authoritative `Position`.
- The server resolves the terminal by stable `TerminalConsole.terminal_id`.
- The server validates:
  - Session is `Playing`.
  - Sender owns the controlled player.
  - Target terminal exists and is active.
  - Positions are finite.
  - Player is within `TERMINAL_INTERACTION_RANGE`.
  - A spatial line-of-sight query is clear, using the current Avian3D API.
  - The actor’s terminal interaction cooldown is ready.
  - The request is not a duplicate/stale request.
- Only after all checks pass may `execute_terminal_command` mutate `TerminalState`.
- Never trust client-provided player positions, authority, inventory, cooldowns, or command outcomes.

## Implementation guidance

- Inspect existing Avian spatial-query usage before choosing an API.
- Prefer a small replicated/server-only component or resource for cooldown and request sequencing.
- Do not add a second terminal architecture.
- Do not make terminal command parsing responsible for spatial/network validation.
- Keep pure validation separate from ECS/entity lookup where practical.
- Do not implement voice chat, classes, progression, procedural terrain, PvP, RL, or LLM behavior in this task.

## Tests required

Add or update focused tests for:

- Valid in-range, visible terminal request succeeds.
- Request through a wall fails.
- Out-of-range request fails.
- Wrong terminal ID fails.
- Non-playing phase fails.
- Duplicate request does not execute twice.
- Cooldown blocks immediate reuse and allows reuse after expiry.
- Forged/unknown keycard still fails.
- Server sender ownership is required.
- Two-client integration test proves one client can interact only with a valid authoritative terminal and the replicated `TerminalState` is consistent.

Use the existing manual-time app/test helpers. Assert state changes, not only logs.

## Workflow rules

Before editing:

1. Read the current relevant files.
2. State one falsifiable local hypothesis about the controlling code path.
3. Identify the cheapest test that could disprove it.

After the first substantive edit:

1. Run the narrowest focused test immediately.
2. Repair that same slice if it fails.
3. Then run the broader relevant tests.

Run at minimum:

```text
cargo test -p shared terminal::tests -- --no-capture --test-threads=1
cargo test -p server lobby::tests -- --no-capture --test-threads=1
cargo test -p client terminal::tests -- --no-capture --test-threads=1
cargo test -p launcher --lib tests::app_flow -- --no-capture --test-threads=1
cargo check -p launcher
```

At the end, report:

- Files changed.
- Authority and validation behavior implemented.
- Focused test results.
- Any API or physics limitation.
- Any unrelated baseline warnings/failures.
- The next smallest recommended slice.
