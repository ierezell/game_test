# Implementation Prompt: Objective, Combat, Resources, Tools, and Extraction

```text
Work in d:/Projects/game_test. Read the phase 0 contract and roadmap. Inspect existing weapons, projectile/hitscan code, Health, Respawnable, Gun, Projectile, TerminalConsole, TerminalState, replicated protocol messages, lobby/game states, and gameplay integration tests.

Goal: implement one complete server-authoritative expedition loop:
DROP -> explore/stealth -> terminal objective -> alarm/holdout -> collect/return -> extraction -> success or failure.

Scope the first slice:
- Terminal commands or structured terminal intents: LIST, QUERY, PING, UNLOCK, OBJECTIVE, and EXTRACT. A text parser is acceptable only if it remains deterministic and testable.
- One main objective with multiple server-owned stages and one optional objective.
- One locked door/key dependency and one alarm holdout.
- Ammo, health, and two tool-charge resources with explicit caps and pickup/refill rules.
- Existing hitscan/projectile weapons hardened with server-side origin validation, line of sight, target validation, cooldown, ammo, and damage application.
- Explicit friendly-fire policy; document it and test it.
- Downed/dead state or the existing respawn behavior, but do not mix two incompatible death models.
- Two tools that promote teamwork, such as a bio-scanner ping and a bounded deployable sentry/mine.
- Extraction activation, progress, cancellation on invalid conditions, success result, and failure result.

Security and authority:
- Clients send intent messages only.
- The server obtains actor identity from the Lightyear connection, not from a client-supplied player ID.
- Server checks game phase, stable target ID, distance, line of sight, cooldown, inventory, and objective prerequisites.
- Replicated state changes are idempotent and have stable IDs.
- Client prediction may show muzzle flash/tracer intent immediately, but authoritative hit/effect feedback arrives from the server.

Tests required:
- Valid and invalid terminal command transitions.
- Objective cannot skip stages or be completed by the wrong actor.
- Weapon cooldown, ammo, range, line of sight, friendly fire, and duplicate request tests.
- Tool charge consumption and failed deployment rollback/no-consume behavior.
- Alarm starts exactly once, wave count stays bounded, and completion/cancellation are consistent across clients.
- Extraction cannot complete before the objective, can be cancelled, and produces one final result.
- Two-client end-to-end success path and failure path.
- Cheating attempts: forged actor, far target, wall target, stale target, repeated request, impossible quantity, and client-side damage claim.

Use the existing test app builders and manual time. Run focused tests immediately after each substantive edit, then the full validation commands. Report the exact mission state machine and all client-to-server messages added.
```
