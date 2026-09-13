# Implementation Prompt: Loadouts, Atmosphere, and Progression

```text
Work in d:/Projects/game_test only after phases 1-5 pass. Read the roadmap and inspect current client VFX, flashlight, HUD, lobby, asset loading, health, weapons, terminals, and server-authoritative inventory/protocol code.

Goal: make the proven expedition loop readable, atmospheric, and cooperative without allowing presentation or progression to destabilize simulation.

Implement in this order:
1. Loadout and item data with server validation at expedition start. Players have no fixed classes or roles.
2. Equipment combinations with bounded, testable tradeoffs, such as reconnaissance, defense, healing, traversal, or suppression items. These are selectable items, not role abilities or role requirements.
3. A pre-mission loadout screen and lobby summary that show capacity, conflicts, item charges, and team coverage without requiring a class composition.
4. A minimal between-run progression record containing cosmetics or sidegrades first. Do not add permanent power that makes resource scarcity meaningless.
5. Stress or infection only as one isolated mechanic with clear sources, caps, UI feedback, and counterplay. Do not implement both until one is fun.
6. Atmosphere pass: darkness, flashlight contrast, fog where supported by the current renderer, sound-state hooks, enemy audio cues, alarm state, and a minimal HUD for health/stamina/ammo/tools/objective/teammates.
7. Environmental storytelling through stable logs, terminal records, and authored prop metadata.

Networking requirements:
- Server validates loadout, item compatibility, inventory, unlocks, and progression rewards. It never accepts a client-declared role because roles do not exist.
- Clients receive compact state and can predict only cosmetic presentation.
- Do not make audio, fog, UI, or post-processing dependencies required by the headless server.
- Keep the existing flashlight as a client visual and ensure remote visuals never create server gameplay effects by themselves.

Tests required:
- Loadout validation, capacity, duplicate restrictions where appropriate, item compatibility, and fallback behavior.
- Server rejects forged items, charges, quantities, unlocks, or progression.
- Item cooldown, charge, resource, and team-combination tests.
- HUD reflects replicated state and does not crash with missing/late entities.
- Headless app starts without renderer-only resources.
- Flashlight/fog/audio presentation setup and teardown tests where practical.
- Save/load or serialization compatibility tests if persistence is introduced.
- A full two-client mission test still passes with independent loadouts and no role-selection dependency.

Do not introduce a large UI framework or a full progression economy before the vertical slice is playable. Use the existing Bevy UI/egui patterns and preserve headless testability. Run focused tests after each slice and then the required workspace validation.
```
