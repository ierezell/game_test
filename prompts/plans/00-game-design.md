# Desired Game Design

## One-sentence identity

A 1-4 player first-person cooperative extraction-horror game where a squad enters a procedurally assembled sci-fi facility, investigates an uncertain threat, completes a reachable objective, and decides how much danger it can endure before extracting.

The game should feel like GTFO under pressure, Species: Unknown during investigation, and Deep Rock Galactic in replayability and mission variety, while using Alien Swarm: Reactive Drop and Aliens: Dark Descent as references for encounter pressure, squad tactics, and escalating consequences.

## Non-negotiable constraints

- PvE only for the initial product direction.
- 1-4 players with a dedicated server, local mode, and deterministic test harness.
- First-person perspective.
- No player classes and no fixed roles.
- Every player chooses a loadout and items within explicit capacity, unlock, and resource rules.
- Teamwork comes from voluntary equipment combinations and communication, not role requirements.
- No procedural terrain or voxel digging in the core plan.
- Facilities are assembled procedurally from authored room, corridor, door, encounter, and prop prefabs.
- The server authoritatively owns mission state, objectives, AI, damage, inventory outcomes, extraction, and rewards.
- Clients may predict local movement and presentation, but cannot decide gameplay outcomes.
- The first playable slice must be fun with a small content set before adding progression, many enemy types, or experimental AI.

## Player fantasy

Players are a disposable specialist crew entering a hostile facility with incomplete information. They are not superheroes and do not have a permanent class identity. Before deployment they decide what to carry, what information they can gather, how much defensive power they sacrifice for utility, and which risks they are willing to accept.

Every expedition should create moments such as:

- “We know what we came here to do, but not what is hunting us.”
- “We can probably reach the objective, but the safe route may cost too much ammunition.”
- “Something reacted to that sound. Do we wait, hide, retreat, or commit?”
- “The extraction route is technically open, but returning through the facility is now more dangerous.”
- “We found the answer, but using it may wake something worse.”

## Core loop

1. **Prepare:** inspect the contract, choose a loadout, and decide which information and tools to bring.
2. **Drop:** enter a facility with a known broad goal but incomplete local information.
3. **Investigate:** explore, read terminals, identify environmental clues, locate dependencies, and learn the threat’s behavior.
4. **Infiltrate:** move quietly through authored spaces assembled in a new arrangement. Manage light, noise, stamina, ammunition, health, and tool charges.
5. **Commit:** unlock a route, activate a terminal, retrieve an object, scan a target, or begin another objective that changes the danger state.
6. **Survive escalation:** respond to sleepers, searches, alarms, pursuit, and bounded waves. Fighting is powerful but expensive and loud.
7. **Decide:** continue for optional rewards and information, or retreat with partial success.
8. **Extract:** reach and activate an extraction route, then hold the area or complete a timed escape condition.
9. **Resolve:** calculate success, failure, recovered items, evidence, credits, and persistent discoveries on the server.

## Tension model: uncertainty with fairness

The game should create uncertainty about the next problem without hiding the rules or invalidating player decisions.

### The player should know

- The primary contract and a clear success condition.
- The current objective stage and the prerequisites discovered so far.
- The cost and duration of loud actions when possible.
- The available health, ammunition, tool charges, and retreat options.
- The observable warning signs: sounds, lights, movement, terminal messages, environmental traces, and enemy states.
- Why a request failed: out of range, locked, missing item, wrong phase, blocked route, or invalid target.

### The player should not know immediately

- Which threat variant is active in the expedition.
- Where every enemy, resource, dependency, or shortcut is located.
- Whether a quiet room is truly safe or only temporarily undisturbed.
- What consequence a new objective activation will cause until clues or terminal information reveal it.
- Whether the shortest path is the safest path.

### Fairness rules

- Every critical objective has at least one reachable solution and a recoverable route unless the squad has already made a clearly signposted irreversible choice.
- Procedural generation validates connectivity, dependency ordering, spawn safety, extraction reachability, navmesh coverage, and resource budgets before gameplay begins.
- Surprise events are selected from authored encounter patterns with telegraphs, response windows, and bounded intensity.
- The director may increase pressure, but it cannot silently delete the only viable objective or extraction path.
- A failure should be attributable to a decision, execution error, resource mistake, or escalating consequence, not an invisible generator defect.

## Procedural facility design

### What is procedural

- Macro zone graph and branch structure.
- Room and corridor prefab selection.
- Door and lock placement.
- Objective and dependency placement.
- Enemy archetype, sleeper group, and encounter placement.
- Resource, terminal, clue, and optional reward placement.
- Threat variant and mission modifiers.
- Lighting, power, alarm, and environmental state configuration.

### What is authored

- Room geometry and connection sockets.
- Corridor widths, door dimensions, cover arrangements, traversal affordances, and navigation metadata.
- Encounter templates, alarm patterns, enemy behavior parameters, and telegraph rules.
- Terminal command vocabulary and objective rules.
- Environmental storytelling fragments, audio cues, VFX, and prop sets.

### Generation pipeline

1. Generate a deterministic mission contract from the expedition seed.
2. Generate a zone graph with start, main objective, optional branch, dependency branch, danger gates, and extraction.
3. Assign authored room prefabs to zones and align compatible sockets.
4. Validate spatial overlap, connection integrity, walkability, spawn safety, and navmesh coverage.
5. Populate stable IDs for zones, doors, terminals, objectives, encounters, items, and extraction points.
6. Place resources and threats using independent deterministic random streams and difficulty budgets.
7. Run a mission solvability validator and export a compact seed/debug summary.
8. Spawn static geometry and deterministic presentation on server/client; replicate dynamic state by stable ID.

Do not use entity order, hash-map iteration order, or floating-point accidents as hidden random inputs.

## Mission structure

The first vertical slice should support one contract family with variations:

- **Primary objective:** recover or download a critical facility data package.
- **Dependency:** locate a key, power cell, or terminal authorization in a side branch.
- **Pressure event:** activating the primary terminal starts an alarm or escalating search.
- **Optional objective:** recover evidence, samples, or a secondary data cache.
- **Extraction:** return to a designated extraction volume and complete a timed hold or traversal condition.

Later contracts can add destruction, capture, investigation, elimination, escort, and multi-stage uplink objectives. Each contract must state what is guaranteed, what is uncertain, and what can be abandoned for partial success.

## Loadouts and items

There are no classes. A player selects from unlocked equipment subject to weight/slot limits, charges, ammunition, and mission economy.

Example item families:

- Weapons with different noise, range, damage, reload, and ammunition tradeoffs.
- Motion or bio scanners that reveal nearby movement or threat evidence.
- Medical items that restore health or stabilize a downed teammate.
- Deployable defenses such as sentries, mines, barriers, or decoys.
- Light and navigation tools such as flares, beacons, or temporary map support.
- Access and utility tools for terminals, doors, power systems, or extraction.
- Emergency items that are strong but loud, rare, or limited to one use.

Items should create cooperation without forcing it. Four players may bring similar equipment, specialize informally, or accept a weak team combination for a high-risk challenge. The server validates every loadout and all item effects.

## Enemy and threat design

Start with a small roster:

- Dormant melee creature: dangerous in close quarters and sensitive to noise/light.
- Ranged creature: pressures cover and makes open routes costly.
- Alerting creature: creates chain escalation if not handled quietly.

Each expedition selects or combines threat behaviors so players investigate clues and adapt. Distinct threats should differ in perception, movement, attack pattern, vulnerability, retreat pressure, and response to tools. Avoid enemies that are only health sponges.

The threat director should create pacing rather than constant combat:

- Quiet reconnaissance.
- Uneasy discovery.
- A preparation window.
- A commitment or alarm.
- A bounded escalation.
- A recovery or retreat decision.

## Technical architecture

- `shared`: serializable components/messages, pure mission generation, validation, stable IDs, deterministic rules, and shared constants.
- `server`: authoritative session, objective, AI, physics outcomes, damage, inventory, extraction, and reward systems.
- `client`: input, prediction, camera, interpolation, HUD, audio, lighting, VFX, and non-authoritative clues/presentation.
- `launcher`: composition for server, client, solo/local, crossbeam, UDP, and headless test applications.

Every new gameplay feature must specify:

- Which side owns the decision.
- Which intent message the client sends.
- Which validation the server performs.
- Which state is replicated.
- How prediction/interpolation presents it.
- Which pure, ECS, and two-client tests prove it works.

## Vertical-slice success criteria

The first complete slice is successful when two players can:

- Connect and appear consistently in the lobby.
- Choose independent loadouts with no role/class selection.
- Enter a seeded prefab facility.
- Investigate clues and locate a dependency.
- Move quietly past or accidentally wake sleepers.
- Complete the primary terminal objective.
- Survive a bounded escalation.
- Return through a changed facility state.
- Extract successfully or fail for a readable reason.
- Repeat the mission with a new seed and receive a valid, reachable layout.

The slice must pass deterministic generation tests, headless ECS tests, two-client local/crossbeam integration tests, and the required workspace validation commands before content breadth expands.
