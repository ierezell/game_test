# Implementation Prompt: Player, Camera, Physics, and Interaction Foundation

```text
Work in d:/Projects/game_test using the existing Bevy 0.18, Avian3D, Lightyear, and bevy_enhanced_input versions from Cargo.toml. Read prompts/plans/00-current-state-and-decisions.md and 01-roadmap.md. Inspect shared input/movement modules, player physics bundles, client camera/input/entities, server player spawning, protocol registration, and the existing gameplay tests before editing.

Goal: produce one correct player foundation that behaves in local mode and under network reconciliation.

First investigate:
- Whether movement is currently integrated by Avian3D, a custom system, or both.
- Which components are predicted, interpolated, replicated, or server-only.
- Whether remote interpolated entities accidentally receive local physics or input systems.
- Whether camera transforms are derived from the predicted controlled player without writing back authoritative gameplay transforms.
- Whether Tnua is necessary. Only add it after documenting a concrete current limitation and verifying version compatibility.

Implement only what is needed for:
- Fixed-step movement and look input.
- Collision and grounded state using the current physics stack.
- Sprint and stamina with server validation and client prediction only where safe.
- Camera follow/look for the controlled player, with clean setup and teardown.
- A generic interaction intent containing a stable target ID or validated ray information. The server must validate phase, actor, distance, line of sight, target state, and cooldown.
- A clear distinction between local predicted presentation, authoritative server state, and interpolated remote presentation.

Do not trust client-provided damage, item counts, target ownership, or arbitrary world positions. Do not add a new camera framework if the existing one can be corrected.

Tests required:
- Pure movement math for WASD, diagonal normalization, yaw, sprint drain, stamina regeneration, and exhausted movement.
- Collision/grounding regression tests using the existing physics test setup.
- Camera follows the controlled predicted player and ignores interpolated remote players.
- Interaction succeeds in range and fails out of range, through a wall, during the wrong session phase, or after cooldown.
- Local and two-client integration tests prove that the controlled player is predicted and the remote player is interpolated.
- Simulated latency/reconciliation test proves bounded correction and no double integration.

After the first edit run the narrowest player tests. Then run cargo fmt --all -- --check, cargo check --workspace, cargo clippy --workspace --all-targets -- -D warnings, and cargo test --workspace -- --no-capture --test-threads=1.

Report the authority table and any remaining physics limitations. Do not replace working systems wholesale.
```
