# Yolo Game Roadmap and Prompt Pack

This directory is the execution plan for the Bevy + Lightyear cooperative survival-horror game.

## Current baseline

The repository already contains:

- Separate `client`, `server`, `shared`, `launcher`, `llm`, and reinforcement-learning crates.
- UDP, crossbeam, and local networking modes.
- Lightyear replication, prediction, interpolation, controlled entities, and replicated messages.
- A lobby and game lifecycle: `LocalMenu`, `Connecting`, `Lobby`, `Loading`, `Spawning`, `Playing` on the client; `Lobby`, `Loading`, `Playing` on the server.
- Seeded procedural zone generation and client/server level creation from the same `LevelSeed`.
- Avian3D physics bundles, player movement, health, weapons, projectiles, terminals, sleepers, patrol/navigation components, and flashlight VFX.
- Integration tests under `crates/launcher/src/tests/`, plus focused unit tests in shared and client code.

This means the next goal is a playable, testable vertical slice, not a new networking bootstrap.

## Read first

1. `00-game-design.md`
2. `00-current-state-and-decisions.md`
3. `01-roadmap.md`
4. The phase prompt that matches the next unchecked gate
5. `07-testing-and-release-gates.md` before changing shared or networked behavior

## Execution rules

- Work in the phase order unless a prompt explicitly marks work as optional.
- Keep the server authoritative for gameplay outcomes. Clients may predict presentation and local movement, but never damage, loot, objective, extraction, or progression outcomes.
- Keep deterministic generation as pure data as long as possible. Spawn Bevy entities only after validation succeeds.
- Generate facilities from authored room, corridor, door, encounter, and prop prefabs. Do not introduce procedural terrain or voxel digging into the core plan.
- Preserve uncertainty through threat selection, clues, pacing, and consequences while guaranteeing reachable objectives and extraction paths.
- Prefer existing modules and protocol patterns over new parallel abstractions.
- Each implementation prompt is a request for an investigation, a minimal edit, focused tests, and validation. Do not ask an AI to generate an entire replacement crate.
- After every slice run `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and the narrowest relevant tests. Use the exact commands in `07-testing-and-release-gates.md`.

## Phase map

| Phase | Outcome | Gate |
| --- | --- | --- |
| 0 | Decisions, architecture contract, and measurable vertical-slice definition | The team can explain authority, state ownership, seed ownership, and failure behavior |
| 1 | Reliable lobby, lifecycle, disconnect, and four-player session | Two to four clients can join, start, play, disconnect, and recover without duplicate state |
| 2 | Movement, camera, physics, stamina, and interaction foundation | A player can move, look, collide, sprint, interact, and be reconciled under simulated latency |
| 3 | Deterministic expedition generator with playable authored modules | Same seed produces the same validated graph, geometry plan, gates, encounters, and resources |
| 4 | Stealth and sleeper threat loop | Noise, light, alert propagation, silent takedowns, combat escalation, and horde pacing work server-side |
| 5 | Objective, terminal, combat, tools, and extraction loop | A squad can infiltrate, solve an objective, survive an escalation, and extract or fail |
| 6 | Loadouts, persistence, atmosphere, and content pipeline | A coherent replayable vertical slice supports complementary equipment choices and readable feedback |
| 7 | Performance, observability, release hardening, and optional AI experiments | Headless soak tests, network metrics, deterministic replay, and a reproducible build pass |

## Inspirations used as design inputs

- **GTFO:** four-player cooperation, stealth before violence, light/noise/vibration waking sleepers, scarce resources, terminal objectives, and pressure that turns preparation into panic.
- **Aliens: Dark Descent:** escalating aggression, persistent world changes, safe zones, squad specialization, stress, and permanent consequences. Borrow the pressure model, not the single-player squad-control structure.
- **Deep Rock Galactic:** 1-4 player readability, equipment synergy, procedural replayability, traversal options, distinct mission types, and an extraction beat. Do not promise destructible voxel terrain in the first slice.
- **Species: Unknown:** 1-4 player contracts, procedurally selected intelligent threats, investigation before commitment, proximity/radio communication, compact mission tools, and gear upgrades. Use the contract and threat-identification structure without copying its content or presentation.
- **Alien Swarm: Reactive Drop:** squad-level tactics, equipment/loadout complementarity, friendly-fire consequences, many-player pressure, challenge modifiers, bots, and repeatable campaign/mutation structure. Adapt the coordination lessons to a first-person game without fixed player classes, and keep the first slice at four players.

## Reference links

- [Species: Unknown](https://store.steampowered.com/app/2747330/Species_Unknown/)
- [Alien Swarm: Reactive Drop](https://store.steampowered.com/app/563560/Alien_Swarm_Reactive_Drop/)
- [Aliens: Dark Descent](https://store.steampowered.com/app/1150440/Aliens_Dark_Descent/)
- [GTFO](https://store.steampowered.com/app/493520/GTFO/)
- [Deep Rock Galactic](https://store.steampowered.com/app/548430/Deep_Rock_Galactic/)

## Prompt conventions

Every phase prompt asks the implementer to return:

1. A short investigation of the relevant existing code.
2. A proposed data model and authority boundary.
3. The smallest implementation slice.
4. Focused unit, integration, and edge-case tests.
5. Commands run and remaining risks.

The prompts are intentionally version-aware: inspect `Cargo.toml` and current Lightyear/Bevy APIs before writing code. Do not copy APIs from newer examples without checking compatibility.
