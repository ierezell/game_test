# Phased Roadmap

This is the order of work. Each phase is complete only when its exit gate and tests pass.

## Current progress

- Phase 0 design contract: documented in `00-game-design.md` and `00-current-state-and-decisions.md`.
- Phase 1 session hardening: in progress. Host start requests are now server-authorized, focused server authorization tests pass, and the two-client ECS app-flow suite passes all 11 tests.
- Input-foundation regression resolved: CCC movement/camera tests now use Enhanced Input's `ActionMock` contract, and all three focused tests pass.
- Interaction foundation started: shared terminal validation now rejects non-playable sessions, invalid positions, out-of-range use, and cooldown reuse; five focused tests pass.
- Terminal interaction networking added: clients can send typed terminal requests, and the server resolves sender ownership, stable terminal identity, range, and authoritative command mutation; focused server tests pass.
- Client terminal input added: pressing `E` selects the nearest in-range terminal and sends the configured command through Lightyear; nearest-target tests and launcher lifecycle validation pass.
- Physics ownership corrected: predicted local players retain client physics, while interpolated remote players and NPCs are presentation-only; the ownership regression, CCC movement tests, and two-client lifecycle tests pass.
- Procedural generation coverage expanded: a 100-seed corpus now verifies objective reachability and terminal objective indexing, alongside the existing 18 generation tests.
- Stealth propagation coverage expanded: cyclic zone graphs now have an executable regression proving noise propagation terminates, attenuates, and stays bounded; all 11 noise and 21 sleeper tests pass.
- Terminal authority hardened: `UNLOCK` now requires an indexed keycard item, preventing forged client commands from mutating unlocked-keycard state; terminal and server authority tests pass.
- Next implementation slice: strengthen player physics/reconciliation and interaction validation, then add the remaining Phase 2 edge-case tests.

## Phase 1: Session and networking hardening

**Goal:** A reliable 1-4 player session exists before adding more gameplay.

Work:

- Make lobby membership, host identity, ready state, start authorization, and disconnect handling explicit.
- Replace duplicate or race-prone start/play transitions with idempotent server-driven lifecycle messages.
- Handle connect, timeout, clean disconnect, reconnect, late join, host departure, and player removal.
- Ensure local/crossbeam/UDP modes exercise the same gameplay protocol.
- Add session IDs and stable player/session IDs where absent.
- Fix the repeated Play button and lobby representation issues in TODO.md.

Exit gate: four simulated clients can join, one host can start, all clients reach `Playing`, one client disconnects during lobby and gameplay, and the remaining clients remain consistent.

Prompt: `01-session-networking.md`

## Phase 2: Movement, camera, physics, and interaction foundation

**Goal:** Movement feels correct and remains correct under reconciliation.

Work:

- Verify the existing movement action path and decide whether Tnua is needed. Do not add it automatically.
- Define collision ownership for predicted local, authoritative server, and interpolated remote entities.
- Add look/camera behavior, grounded/jump/sprint/stamina rules, and interaction range checks.
- Ensure movement uses fixed-step simulation and presentation smoothing without double-integrating transforms.
- Add interaction intent messages and server validation without attaching gameplay outcomes to client raycasts.

Exit gate: a player can walk, sprint, jump if enabled, collide with generated geometry, interact with a terminal, and remain stable with artificial latency and packet loss.

Prompt: `02-player-foundation.md`

## Phase 3: Deterministic expedition generation

**Goal:** A seed produces an interesting, solvable, performance-bounded mission.

Work:

- Preserve the existing `LevelGraph` and extend it with explicit mission metadata.
- Generate macro topology first: start, objective path, optional branch, key/lock dependencies, alarm/holdout, and extraction.
- Assemble geometry from validated room/connection modules. Keep the current primitive builder as a test fixture while the asset pipeline matures.
- Generate stable IDs and deterministic placements for doors, terminals, sleepers, loot, resources, and extraction.
- Validate reachability, collision bounds, navmesh coverage, spawn safety, sightline/cover budget, resource budget, and extraction reachability.
- Add a seed/debug export so failed seeds can be replayed.

Exit gate: a seed can be generated on server and client, validated identically, and loaded without soft-locks across a seed corpus.

Prompt: `03-expedition-generation.md`

## Phase 4: Stealth, noise, light, and enemy escalation

**Goal:** The game creates tension before combat and a readable escalation after detection.

Work:

- Implement server-side noise events with source type, position, radius, propagation, and expiry.
- Implement sleeper states and deterministic alert transitions driven by noise, light, proximity, damage, and scout alerts.
- Add occlusion/dampening through doors and geometry; start with a cheap bounded query and add physics raycasts only where needed.
- Add silent takedown validation, scout chain alerts, search behavior, and combat escalation.
- Add a director budget that schedules encounters without spawning arbitrary unbounded hordes.

Exit gate: a squad can sneak through, cause a partial alert, fully wake a room, trigger a scout chain, and survive a bounded wave. Clients see smooth presentation while the server owns all decisions.

Prompt: `04-stealth-and-ai.md`

## Phase 5: Objective, combat, resources, tools, and extraction

**Goal:** One complete expedition is playable from drop to extraction or failure.

Work:

- Formalize terminal commands and objective state transitions.
- Harden hitscan/projectile validation, cooldowns, ammo, friendly fire policy, damage types, and death/downed behavior.
- Add a small resource model: ammo, health, tool charges, and one scarce mission resource.
- Add two tools that create cooperation, such as a scanner and deployable defense.
- Implement alarm holdouts, extraction activation, extraction progress, cancellation/failure, and final result replication.
- Add a deterministic run summary for debugging and future progression.

Exit gate: two players can complete the entire mission loop, with server validation and tests covering cheating attempts and contradictory requests.

Prompt: `05-core-expedition-loop.md`

## Phase 6: Loadouts, progression, and atmosphere

**Goal:** The vertical slice has distinct team decisions and a coherent presentation layer.

Work:

- Add flexible loadout and item data structures with meaningful tradeoffs; do not add player classes or fixed roles.
- Define server-authoritative loadout validation, inventory, item ownership, charges, and equipment compatibility.
- Ensure cooperation emerges from player-selected item combinations rather than role requirements.
- Add limited between-run progression without power creep that invalidates mission tension.
- Add stress/infection only after the base resource and extraction loop is fun; treat them as separate meters with clear counterplay.
- Add darkness, flashlight readability, fog, audio cues, encounter music states, and a minimal HUD.
- Add environmental storytelling through stable logs, terminal records, and authored prop tags.

Exit gate: a squad must make meaningful loadout choices, understand threat/resource state from the HUD/audio, and complete multiple seeds without presentation blocking play.

Prompt: `06-roles-atmosphere-progression.md`

## Phase 7: Performance, observability, and release hardening

**Goal:** The game is diagnosable and reliable under representative load.

Work:

- Add structured logs and counters for ticks, replication, corrections, RTT, packet loss, AI population, noise events, and generation timings.
- Add deterministic seed replay and a headless expedition runner.
- Add soak tests for four clients, enemy waves, repeated level generation, disconnects, and late joins.
- Profile server fixed-step cost, navmesh updates, physics, replication volume, and client rendering.
- Add save/version migration only when a persistent feature actually exists.
- Keep RL and LLM work in isolated experiments with no effect on authoritative gameplay until benchmarked against a deterministic baseline.

Exit gate: a clean build, tests, clippy, reproducible headless run, documented performance budget, and no known release-blocking desync.

Prompt: `07-testing-and-release-gates.md`
