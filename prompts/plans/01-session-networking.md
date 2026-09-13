# Implementation Prompt: Session and Networking Hardening

```text
You are working in the existing Rust workspace at d:/Projects/game_test. Read prompts/plans/00-current-state-and-decisions.md and prompts/plans/01-roadmap.md first. Inspect the current client, server, shared protocol, launcher host composition, lobby modules, and launcher integration tests. The repository currently uses Bevy 0.18 and the Lightyear version declared by Cargo.toml; do not copy APIs from newer documentation without checking the installed version.

Goal: harden the existing lobby and session lifecycle for a 1-4 player PvE expedition.

Required behavior:
- Server is authoritative for lobby membership, host identity, ready state, start authorization, session phase, and player removal.
- Client requests are idempotent intents. A client cannot start a game unless it is the server-approved host and the lobby is valid.
- Start, loading, spawning, playing, failed, completed, and returning-to-lobby transitions must not fire repeatedly when the same message or replicated state is observed more than once.
- Clean disconnect, timeout, host departure, and disconnect during gameplay must remove or mark the player consistently on the server and every remaining client.
- A late join is either explicitly supported at a safe phase or explicitly rejected with a reason. Do not silently create a half-loaded player.
- Local, crossbeam, and UDP modes must use the same protocol and state semantics.

Implementation constraints:
- Reuse existing LobbyState, game states, Lightyear observers/messages, and integration-test helpers when they are correct.
- Add the smallest new message/resource/component set needed for session identity, ready state, and transition reasons.
- Keep UI fixes focused: one start/play control and a lobby list that reflects server membership and ready state.
- Do not add persistence, matchmaking, voice, or new gameplay systems.

Tests required:
- Unit tests for idempotent transition handling and host authorization.
- Integration tests for 1, 2, 3, and 4 clients joining and reaching Playing.
- Disconnect in Lobby, Loading, and Playing.
- Host disconnect and a rejected non-host start request.
- Duplicate start messages and duplicate disconnect notifications.
- Late join behavior at each lifecycle phase.
- Assert both state and entity counts; do not rely only on log output.

Workflow:
1. State the current control path and one concrete race or inconsistency found.
2. Make a minimal implementation change.
3. Run the narrowest affected tests immediately.
4. Run formatting, workspace check, clippy, and the full test command from the repository instructions.
5. Report changed files, test commands, and any behavior intentionally left unsupported.

Do not rewrite whole modules or invent a second lobby architecture.
```
