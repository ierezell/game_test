<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# GTFO-Like Game Development Instructions

This is a GTFO-like cooperative survival horror game built with Bevy game engine and Avian physics engine.

## Project Structure
- `player.rs` - Player movement, actions, and flashlight mechanics
- `combat.rs` - Weapon systems, projectiles, and damage handling
- `environment.rs` - Atmospheric environments, lighting, and props
- `enemies.rs` - AI-driven enemies with different behaviors
- `ui.rs` - HUD elements including health, ammo, and crosshair
- `audio.rs` - Sound effects and atmospheric audio
- `networking.rs` - Multiplayer foundation for cooperative gameplay
- `utils.rs` - Utility functions for tension, events, and team mechanics

## Key Features
- **Player Movement**: WASD movement with crouching, sprinting, and stamina
- **Combat System**: Realistic weapon mechanics with different weapon types
- **Enemy AI**: Multiple enemy types with patrol, chase, and attack behaviors
- **Atmospheric Environment**: Dark corridors with flickering emergency lights
- **Cooperative Elements**: Foundation for 4-player cooperative gameplay
- **Tension System**: Dynamic tension that affects gameplay and atmosphere
- **Procedural** : Adding randomness in levels, map, enemies quests and the like. 

## Coding Guidelines
- Use Bevy's ECS (Entity Component System) architecture
- Implement systems that are modular and can run in parallel
- Use Avian3D for physics interactions
- Focus on performance for real-time multiplayer gameplay
- Maintain atmospheric horror elements in all systems
- Design with cooperative gameplay in mind

## Game Mechanics
- Health and stamina management
- Ammunition scarcity
- Stealth and noise mechanics
- Environmental interaction
- Team-based objectives
- Dynamic lighting and shadows
- Dynamic Expedition System
* Procedurally generated facility layouts with hand-crafted room templates
* Mission objectives that evolve based on team performance and discoveries
* Resource scarcity that forces difficult decisions about when to push forward vs. retreat
- Enhanced Communication Mechanics
* Proximity-based voice chat with whisper/normal/shout volume levels affecting enemy detection
* Environmental audio cues that require team coordination to interpret
* Equipment that can jam communications in certain areas, forcing visual signals
- Asymmetric Player Roles
* Technician: Hacks terminals faster, can jury-rig equipment, sees additional interface elements
* Medic: Faster healing, can craft medical supplies, detects teammate health status
* Engineer: Repairs/maintains equipment, sets up defensive positions, manages power systems
* Scout: Moves quieter, better at lockpicking, enhanced motion detection
- Unique Features
- Sanity/Stress System
* Prolonged exposure to horrors affects accuracy, reaction time, and decision-making
* Team cohesion mechanics where isolated players suffer penalties
* Environmental storytelling that reveals disturbing lore, affecting mental state
- Equipment Degradation
* Weapons jam and break down under stress
* Flashlights dim and flicker at crucial moments
* Oxygen/power management for sealed facility sections
- Emergent Horror Elements
* AI director that adapts enemy spawns based on team stress levels and performance
* Environmental hazards that create cascading failure scenarios
* False security moments followed by overwhelming encounters
- Mission Variety
- Extraction Scenarios
* Rescue operations where you protect AI teammates with different movement speeds
* Data recovery missions requiring extended time in dangerous areas
* Supply runs where you must return with specific items intact
- Facility Exploration
* Multi-level complexes with branching paths and optional areas
* Time-sensitive objectives where delayed action changes the facility state
* Mysteries that unfold through environmental storytelling and audio logs
* The key is balancing overwhelming odds with just enough tools and teamwork to make victory feel earned rather than impossible.

When adding new features, consider:
1. How it affects cooperative gameplay
2. Impact on game tension and atmosphere  
3. Balance between difficulty and fun
4. Performance implications for multiplayer
5. Integration with existing systems
