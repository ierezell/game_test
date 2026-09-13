# Implementation Prompt: Stealth, Noise, Light, and Enemy Escalation

```text
Work in d:/Projects/game_test with the existing shared sleeper, navigation, building, health, flashlight, and Lightyear protocol code. Read the roadmap and inspect the actual components and systems before editing. Server owns all alert, AI, damage, and spawn decisions; clients only present replicated state and local flashlight visuals.

Goal: create the first tension loop: dormant creatures can be avoided, disturbed, escalated, and eventually fought.

Implement a small data-driven model:
- NoiseEvent: stable source/owner, position, radius or intensity, noise type, propagation mode, tick/expiry.
- Light stimulus: source, cone/range/intensity, tick/expiry; use the existing PlayerFlashlight state as input but do not make the client authoritative.
- Sleeper alert state with explicit transitions such as Dormant, Disturbed, Searching, Alerted, and Combat.
- Per-archetype sensitivity, reaction delay, hearing/light range, and alert behavior.
- Scout/screamer chain alert with a bounded propagation budget and duplicate suppression.
- Search and chase targets selected from server-known players, with navmesh/pathing work bounded per tick.
- A threat director or encounter budget that controls when and how many enemies wake/spawn.

Start with cheap spatial filtering and deterministic scoring. Add physics raycasts for occlusion only where they materially change behavior. Do not run full AI on clients and do not spawn unbounded hordes. Keep AI decisions in fixed/update schedules appropriate to the existing code.

Interaction with existing systems:
- Emit noise from movement, doors, shots, impacts, and tools through an event/message path.
- Use flashlight state for server-side stimulus checks, while keeping the actual SpotLight client-side.
- Reuse Sleeper, PatrolState, SimpleNavigationAgent, Health, and Replicate when their semantics fit.
- Replicate compact state/target information needed for presentation; do not replicate server query internals.

Tests required:
- Noise radius/falloff and expiry.
- Door/wall dampening and occlusion behavior.
- State transition table, including repeated stimuli and damage while dormant.
- Light cone/range thresholds.
- Scout alert reaches eligible sleepers once and respects a bound.
- Search/chase target selection is deterministic and does not panic with zero players.
- Director respects population and budget caps.
- Two-client integration test proves an action by one player wakes the correct server-side enemies and both clients receive the same result.
- Headless performance test with a representative enemy count; record a budget rather than claiming unlimited scalability.

After the first substantive edit run the focused stealth tests before reading or changing adjacent systems. Then run the required format/check/clippy/full test commands. Report any behavior that is intentionally simplified for the vertical slice.
```
