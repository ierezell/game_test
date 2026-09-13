# Co-op Horror Extraction Shooter - Game Design Prompt

You are a game design agent helping to transform this multiplayer FPS template into a **cooperative horror extraction shooter** inspired by **GTFO, Deep Rock Galactic, Alien Swarm: Reactive Drop, Aliens: Dark Descent, and ARC Raiders**.

## Current Codebase Overview

**Tech Stack:**
- Bevy 0.18 + Lightyear 0.26 (multiplayer networking)
- Avian3D (physics)
- vleue_navigator (navmesh/navigation)
- Procedural level generation with zone graphs (Hub, Corridor, Utility, Industrial, Objective, Storage)
- Raycast + projectile weapons
- Health/regeneration/respawn systems
- Patrol-based enemy AI with navmesh pathfinding
- Flashlight component
- And many more

**Existing Assets:**
- Zone-based procedural generation (`crates/shared/src/level/generation.rs`)
- Physics building with doors/openings (`crates/shared/src/level/building.rs`)
- Enemy spawning with patrol routes (`crates/shared/src/level/building.rs:126-169`)
- Navigation system with navmesh (`crates/shared/src/navigation.rs`)
- Weapons: hitscan Gun + ProjectileGun (`crates/shared/src/components/weapons.rs`)
- Health with regen + Respawnable (`crates/shared/src/components/health.rs`)

---

## Core Design Pillars (Synthesized from All References)

### 1. **Squad-Based Cooperative Extraction** (GTFO, DRG, ARC Raiders, Dark Descent)
- **4-player squad** (GTFO, DRG, Alien Swarm) - optimal team size
- **Drop in → Complete objectives → Extract** loop (all games)
- **Permadeath per expedition** with penalties (GTFO, Dark Descent)
- **Extraction types**: Loud (elevator/dropship) vs Silent (hatch/key) (ARC Raiders)
- **Safe pockets** for critical items that persist on death (ARC Raiders)
- The goal is to bring new thrills and unexpected games on each expedition like discovering a new level in GTFO.

### 2. **Class-Based Loadouts with Synergy** (DRG, Alien Swarm, Dark Descent)

**Loadout Selection**: Pre-mission screen, complementary tools, meta-progression unlocks (DRG, Dark Descent)

### 3. **Stealth-First with Noise/Light Systems** (GTFO, Dark Descent, ARC Raiders)
- **Sleepers**: Dormant enemies with 4 alert levels (GTFO: Asleep → Disturbed → Pulsing → Awake)
- **Noise Propagation**: Spherical falloff, doors/walls dampen, different sources (footsteps, gunfire, doors, tools)
- **Light Detection**: Flashlight cone wakes sleepers; use bio-tracker/glowsticks/flares instead
- **Silent Takedown**: Melee from behind on dormant = instant kill; partial charge = alert
- **Sound = Aggro**: Every gunshot draws both AI and players (ARC Raiders)

### 4. **Procedural Expeditions with Terminal Objectives** (GTFO, DRG, Dark Descent)
- **Seed-based generation**: Same seed = same layout (client + server deterministic)
- **Terminal Commands**: `LIST`, `QUERY`, `PING`, `UNLOCK`, `DOWNLOAD`, `EXTRACT`, `REACTOR_*`, `UPLINK_*`
- **Objective Types**: 
  - Mining/Collection (DRG: Morkite, Aquarq, Eggs)
  - Hacking/Terminal sequences (Alien Swarm, GTFO)
  - Escort/Defend (DRG: Doretta, Refinery; Dark Descent: colonists)
  - Elimination (DRG: Dreadnoughts; Dark Descent: hive)
  - Scan/Investigate (DRG: Deep Scan; Dark Descent: motion tracker)
- **Secondary Objectives**: Optional bonus rewards (DRG)

### 5. **Enemy Archetypes & AI Behaviors** (All Games)

| Archetype | Source | Behavior | Weakness |
|-----------|--------|----------|----------|
| **Striker/Drone** | GTFO, DRG, Alien Swarm | Melee rush, climb walls/vents | Headshots, C-Foam, flamethrower |
| **Shooter/Spitter** | GTFO, DRG, Alien Swarm | Ranged projectile, acid/spit | Cover, flank, shields |
| **Scout/Screamer** | GTFO, Dark Descent | Screams → chain wake all nearby | Silent takedown before scream |
| **Tank/Praetorian/Charger** | GTFO, DRG, Alien Swarm | High HP, charge, armor | Focus fire, weak spots, environment |
| **Shadow/Stalker** | GTFO, Dark Descent | Invisible/stealth, flank | Bio-tracker, sound, flares |
| **Brood Mother/Matriarch** | DRG, ARC Raiders | Boss, spawns minions, area denial | Coordinated focus, heavy weapons |
| **Synth/Human Hostiles** | Dark Descent, ARC Raiders | Cover, tactics, grenades | Flanking, suppression |

**AI Behaviors** (from Dark Descent, ARC Raiders):
- **Flank & hunt**: AI flanks, flushes from cover, punishes predictable movement
- **Hive aggression meter**: Detected → Hunted → escalating spawns (Dark Descent)
- **Persistent world**: Welded doors stay shut, hacked terminals stay hacked (Dark Descent)
- **Extraction camping**: Players/ARC converge on extraction alarms (ARC Raiders)

### 6. **Resource Scarcity & Management** (GTFO, DRG, Dark Descent)
- **Ammo**: Limited, scavenge from lockers/boxes, Nitra calls resupply (DRG)
- **Health**: No mid-combat regen; medkits rare; Red Sugar/healing beacon (DRG, Alien Swarm)
- **Tools**: Consume batteries/charges (bio-tracker, sentry, mines, C-Foam)
- **Infection/Stress/Trauma** (GTFO, Dark Descent):
  - **Infection**: % from fog/enemies → caps max HP → cured by medkits/extraction
  - **Stress**: Combat → Stress Effects (shaking, accuracy loss) → Trauma (permanent traits)
  - **Trauma**: Psychiatric care at base, rotate marines

### 7. **Tools & Gadgets (Deployables)** (GTFO, DRG, Alien Swarm, Dark Descent, ARC Raiders)

| Tool | Function | Resource | Class Affinity |
|------|----------|----------|----------------|
| **Bio-Tracker/Motion Scanner** | Ping enemies through walls | Battery | Scout/Tech |
| **C-Foam Launcher** | Seal doors, slow enemies, barriers | Canisters | Engineer |
| **Sentry Gun/Turret** | Auto-turret (180° arc, limited ammo) | Power cells | Engineer/Tech |
| **Mine Deployer** | Proximity/trigger/tripwire mines | Mines | Tech/Driller |
| **Motion Sensor** | Audio/visual alerts on approach | Battery | Tech |
| **Fog Repeller/Turbine** | Clears infectious fog in radius | Charges | All |
| **Flare Gun** | Illuminates massive areas | Flares | Scout |
| **Platform Gun** | Creates climbable platforms | Ammo | Engineer |
| **Drill/C4** | Terrain destruction, shortcuts | Fuel/C4 | Driller |
| **Zipline/Shield** | Traversal, emergency defense | Charges | Gunner |
| **Heal Beacon/Medigun** | Stationary/mobile healing | Medical supplies | Medic |
| **Gravity Trap/Smoke** | Crowd control, LOS blocking | Charges | All (ARC Raiders) |

### 8. **Meta-Progression & Persistence** (DRG, Dark Descent, ARC Raiders)
- **Between missions**: Hub (Space Rig / Otago / Speranza)
- **Upgrade workbenches** → stronger weapons, armor, gadgets
- **Class promotion/perks** → specialize roles (Dark Descent: 5 roles)
- **Research tree** → new tech from samples (Dark Descent)
- **Seasonal content** → new weapons, cosmetics, mutators (DRG)
- **Free loadout** + **Scrappy/drip feed** for bad runs (ARC Raiders)

### 9. **Atmosphere & Horror** (GTFO, Dark Descent, ARC Raiders)
- **Fog of War**: Hidden areas create dread (Dark Descent)
- **Dynamic Music**: Eerie stealth → intense combat cues (GTFO)
- **Enemy Audio Cues**: Distinct vocalizations per type (GTFO)
- **Lighting**: Emergency lights, total darkness, flare illumination
- **Environmental Storytelling**: Logs, terminals, visual clues

---

## Implementation Roadmap

### Phase 1: Core Stealth & Sleeper Systems
- [ ] **Noise System**: `NoiseEvent { source, position, radius, type, propagation }` → raycast occlusion
- [ ] **Sleeper State Machine**: `Dormant → Investigating (pulsing) → Alerted → Combat`
- [ ] **Wake Thresholds**: Per-archetype sensitivity to sound/light/proximity
- [ ] **Light Detection**: Flashlight cone intersection → wake probability curve
- [ ] **Silent Takedown**: Melee charge mechanic, backstab multiplier, debris physics
- [ ] **Scout Scream**: Chain reaction event wakes all sleepers in radius

### Phase 2: Terminal & Expedition Framework
- [ ] **Terminal UI**: egui text-based with autocomplete, command history, ping sounds
- [ ] **Expedition Generator**: Seed → zone graph + objectives + enemy placement + loot tables
- [ ] **Objective System**: Multi-step with terminal integration, reactor/uplink sequences
- [ ] **Extraction Mechanic**: 
  - Loud: Call dropship → alarm → defend zone (90s cargo, 60s airshaft)
  - Silent: Raider Hatch Key → 15s window, no alarm
  - Passive: 2-min idle in zone → auto-extract (fail)
- [ ] **Security Doors**: Bioscan (all players), hacking minigame, physical locks

### Phase 3: Classes, Loadouts & Tools
- [ ] **Class System**: 4 classes with unique traversal, weapon, tool, throwable
- [ ] **Loadout Screen**: Pre-expedition, complementary picks, meta unlocks
- [ ] **Tool Components**: All 11+ tools as components with cooldowns, ammo, batteries
- [ ] **Resource Pickups**: Lockers with ammo/medkits/tool refills (DRG Nitra equivalent)

### Phase 4: Infection, Stress & Progression
- [ ] **Infection Component**: %, max HP cap, tick rate, visual distortion, cure items
- [ ] **Stress/Trauma System** (Dark Descent): Stress meter → effects → trauma traits → psychiatric care
- [ ] **Downed/DBNO State**: Crawl, revive, carry, extract while downed (ARC Raiders)
- [ ] **Checkpoint/Respawn**: Terminals = limited respawns, permadeath on expedition fail
- [ ] **Hub/Progression**: Weapon upgrades, class perks, research, cosmetics

### Phase 5: Polish, Atmosphere & PvPvE (Optional)
- [ ] **Fog System**: Volumetric, infectious, cleared by repeller/turbine
- [ ] **Audio**: 3D positional, occlusion, sleeper vocalizations, music states
- [ ] **Lighting**: Dynamic, emergency, flashlight variants per weapon
- [ ] **UI/UX**: Minimal HUD, diegetic terminal, teammate status (health, stress, ammo)
- [ ] **PvPvE Layer** (ARC Raiders): Aggression-based matchmaking, extraction camping, proximity voice

---

## Key Code Locations to Extend

| Feature | File | Integration Notes |
|---------|------|-------------------|
| Sleeper AI | `crates/shared/src/navigation.rs` | Extend `SimpleNavigationAgent` with state machine, alert levels |
| Level Gen | `crates/shared/src/level/generation.rs` | Add `ExpeditionSeed`, `ObjectiveZone`, `EnemyPlacementConfig` |
| Enemy Spawn | `crates/shared/src/level/building.rs:126` | Replace patrols with sleeper zones, archetype weights by zone type |
| Weapons | `crates/shared/src/components/weapons.rs` | Add `MeleeWeapon` (charge), `ToolWeapon` (deployables), silent variants |
| Health | `crates/shared/src/components/health.rs` | Add `Infection`, `Stress`, `Downed`, `Trauma` components |
| Protocol | `crates/shared/src/protocol.rs` | Terminal commands, noise events, extraction signals, class loadouts |
| Player | `crates/shared/src/components/mod.rs` | Class, loadout, safe pocket, progression data |

---

## Reference Mechanics Deep-Dive

### GTFO (Primary Inspiration)
- **4 alert levels**: Visual pulsing chest, audio cues
- **Terminal commands**: LIST/QUERY/PING + expedition-specific (REACTOR, UPLINK)
- **Infection**: Fog (environmental) + Spitters (enemy) → max HP cap → medkit cure
- **C-Foam**: 2s harden, 30s duration, shootable, seals doors
- **Sentry**: 180°, 150 rounds, friendly fire ON
- **Extraction**: Security scan sequence → alarm waves → defend → extract

### Deep Rock Galactic
- **Mission types**: 10 types (Mining, Egg Hunt, Point Extraction, Elimination, Escort, Sabotage, Deep Scan, Heavy Extraction, Salvage, On-site Refining)
- **Nitra**: Red mineral → calls resupply pod (80 Nitra = 4 racks of 50% ammo/health)
- **Secondary objectives**: Fossils, Apoca Bloom, Dystrum, Hollomite, extermination
- **Hazard levels**: 1-5 + mutators (double XP/credits)
- **Season system**: Free content, no playerbase split

### Alien Swarm: Reactive Drop
- **4 classes, 8 characters**: Officer, Special Weapons, Medic, Tech
- **Tech required** for hacking objectives (welding, terminals)
- **Formation tactics**: Tank front → DPS middle → Medic rear → Tech rear guard
- **Friendly fire**: Always on, explosives = gibs
- **8-player support**: Doubles spawn rate
- **Persistent unlocks**: Level → weapons/items (situational, not power creep)

### Aliens: Dark Descent
- **Squad control**: Single unit, leader auto-assigns tasks
- **Command Points**: Limited pool (3), slow regen → abilities (suppression, grenades)
- **Alien Aggression**: Detected → Hunted → escalating horde spawns
- **Stress → Trauma**: 100% stress = effect → level up → trauma trait (permanent)
- **Persistent maps**: Welded doors, hacked terminals, looted containers persist
- **Ammo scarcity**: No limit on pickup, but can't bring extra into mission
- **Base management**: Heal, promote, research, outfit (light XCOM layer)

### ARC Raiders
- **PvPvE**: Machines + players, sound = aggro for both
- **Extraction types**: Cargo elevator (90s, loud), Airshaft (60s, quieter), Metro (Buried City), Raider Hatch (15s, silent, key)
- **Safe Pockets**: Items persist on death (keys, quest items, high-value)
- **Aggression matchmaking**: Weighted PvP/PvE preference, not binary
- **TTK**: Longer, back-and-forth fights, time to react
- **Free loadout + Scrappy**: Safety nets for bad runs

---

## Design Decisions Needed

1. **Scope**: Pure PvE (GTFO/DRG/Dark Descent) or PvPvE (ARC Raiders)?
2. **Perspective**: FPS (GTFO/DRG) or top-down (Alien Swarm/Dark Descent)? *Current: FPS*
3. **Class System**: Rigid 4-class (DRG) or flexible loadout (GTFO/ARC Raiders)?
4. **Meta-Progression**: Deep (DRG/Dark Descent) or session-only (GTFO)?
5. **Terminal**: Full text parser (GTFO) or radial+hotkeys (accessible)?
6. **Infection vs Stress**: Both? One? Hybrid?
7. **Map Persistence**: Full (Dark Descent) or per-expedition (GTFO/DRG)?

---

## Suggested First Implementation Steps

1. **`Sleeper` Component** + State Machine (Dormant/Investigating/Alerted/Combat)
2. **`NoiseEvent` Message** + Propagation System (sphere + raycast occlusion)
3. **`Expedition` Resource** + Terminal Command Protocol
4. **Class/Loadout Components** + Pre-Mission Selection UI
5. **`Infection` + `Stress` Components** + Visual/Audio Feedback
6. **Tool Deployables** (Sentry, Mine, C-Foam, Scanner) as `ToolWeapon` variants

---

## Output Format for Implementation Requests

When implementing, provide:
1. **Components/Resources**: `#[derive(Component, Serialize, Deserialize, Clone, Debug)]`
2. **Systems**: `fn system_name(mut commands: Commands, query: Query<...>, ...)` with proper filters
3. **Messages**: Lightyear `#[derive(Message, Serialize, Deserialize)]` for networking
4. **Integration**: Exact file:line references to existing code
5. **Tests**: `#[cfg(test)]` modules demonstrating behavior

---

*Ready to start? Specify: Phase 1 system, or a specific component to implement first.*