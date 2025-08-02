# GTFO-Like Game

A cooperative survival horror game inspired by GTFO, built with the Bevy game engine and Avian physics.

## Features

### 🎮 Core Gameplay
- **First-person perspective** with realistic movement mechanics
- **Cooperative multiplayer** foundation for up to 4 players  
- **Tactical combat** with multiple weapon types
- **Stealth mechanics** with crouching and noise management
- **Stamina system** affecting movement and actions
- **Atmospheric horror** environment with dynamic lighting

### 🔫 Combat System
- **Multiple weapon types**: Assault rifles, shotguns, pistols, sniper rifles
- **Realistic ballistics** with projectile physics
- **Ammo management** and reload mechanics
- **Damage system** with health regeneration
- **Weapon accuracy** affected by movement and stance

### 🤖 Enemy AI
- **Multiple enemy types**:
  - **Striker**: Fast, low health, melee attacks
  - **Shooter**: Ranged attacks, medium health
  - **Tank**: High health, slow movement, heavy damage
- **Advanced AI behaviors**: Patrol, chase, attack, search states
- **Dynamic difficulty** scaling based on team performance

### 🌍 Environment & Procedural Generation
- **Procedural level generation** with noise-based corridor networks
- **Dynamic room placement** with special room types (Server, Lab, Security)
- **Procedural decoration system** with pipes, lights, and atmospheric elements
- **Intelligent enemy spawning** based on tension levels and player position
- **Atmospheric lighting** with flickering emergency lights
- **Interactive objects** and environmental props
- **Realistic physics** with Avian3D integration
- **Dynamic events** that affect tension and gameplay
- **Infinite level streaming** for expansive exploration

### 🎵 Audio & Atmosphere
- **3D spatial audio** for immersive sound design
- **Dynamic footsteps** based on movement speed and surface
- **Weapon sound effects** and environmental audio
- **Tension system** that affects ambient sounds

### 🌐 Networking (Foundation)
- **Multiplayer architecture** ready for cooperative gameplay
- **Player synchronization** for position and actions
- **Team-based mechanics** for coordinated gameplay
- **Scalable networking** for 4-player teams

## Controls

| Action | Key |
|--------|-----|
| Move | WASD |
| Look | Mouse |
| Jump | Space |
| Crouch | Left Ctrl |
| Sprint | Left Shift |
| Interact | E |
| Flashlight | F |
| Fire | Left Mouse |
| Aim | Right Mouse |
| Reload | R |
| Switch Weapon | Q |

## Getting Started

### Prerequisites
- Rust (latest stable version)
- Cargo

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd gtfo-like-game
```

2. Build the project:
```bash
cargo build --release
```

3. Run the game:
```bash
cargo run --release
```

### Development

For development with faster compile times, this project includes several optimizations:

#### Fast Compilation Setup
The project is configured with optimizations from the [Bevy Setup Guide](https://bevy.org/learn/quick-start/getting-started/setup/):

- **Dynamic linking**: Bevy uses dynamic linking in debug builds for faster iteration
- **Optimized dependencies**: Dependencies are compiled with `opt-level = 3` while your code uses `opt-level = 1`
- **LLD linker**: Uses rust-lld for faster linking on Windows
- **Incremental compilation**: Enabled for faster rebuilds

#### Development Commands
```bash
# Run in development mode (faster compilation)
cargo run

# Run specific binary
cargo run --bin full   # Full game with all systems
cargo run --bin simple # Simple test scene

# Build with optimizations for testing
cargo run --release

# Quick compilation check without linking
cargo check

# Watch for changes and auto-rebuild (install with: cargo install cargo-watch)
cargo watch -x run
```

#### Development Scripts
For convenience, use the provided development scripts:

**Windows Batch:**
```cmd
dev-run.bat
```

**PowerShell:**
```powershell
.\dev-run.ps1
```

These scripts automatically set optimization flags and build the project.

#### Additional Performance Tips
- **SSD recommended**: Place the project on an SSD for faster I/O
- **RAM**: 16GB+ recommended for comfortable development  
- **Parallel compilation**: The project uses all available CPU cores
- **Dynamic linking**: Enabled by default for faster development builds

#### Project Structure
```
src/
├── main.rs              # Main game setup and initialization
├── game_state.rs        # Game state management
├── player.rs            # Player movement and actions
├── combat.rs            # Weapon systems and damage
├── environment.rs       # Environment and atmosphere
├── enemies.rs           # Enemy AI and behaviors
├── ui.rs                # User interface and HUD
├── audio.rs             # Sound effects and music
├── networking.rs        # Multiplayer networking
├── procedural.rs        # Procedural generation systems
├── level_generation.rs  # Level streaming and construction
└── utils.rs             # Utility functions and systems
``` enemies.rs        # Enemy AI and behaviors
├── ui.rs             # User interface and HUD
### Engine & Libraries
- **Bevy**: Modern game engine with ECS architecture
- **Avian3D**: Advanced physics simulation
- **Leafwing Input Manager**: Flexible input handling
- **Noise**: Procedural generation using Perlin noise
- **Bracket Random**: Advanced random number generation
- **Serde**: Serialization for networking and save data
## Technical Details

### Engine & Libraries
- **Bevy**: Modern game engine with ECS architecture
- **Avian3D**: Advanced physics simulation
- **Leafwing Input Manager**: Flexible input handling
- **Serde**: Serialization for networking and save data

### Performance Optimizations
- Dynamic linking in debug builds for faster compilation
- Optimized dependencies in development
- Parallel system execution with Bevy's scheduler
- Efficient physics simulation with Avian

### Architecture
- **Entity Component System (ECS)**: Modular and performant
- **System-based design**: Reusable and maintainable code
- **Resource management**: Efficient memory usage
- **Event-driven**: Responsive gameplay systems

## Gameplay Mechanics

### Tension System
The game features a dynamic tension system that:
- Increases during combat encounters
- Decreases over time when safe
### Procedural Events
- **Environmental hazards**: Power outages, alarms
- **Dynamic encounters**: Randomized enemy reinforcements
- **Atmospheric events**: Audio cues and visual effects
- **Adaptive difficulty**: Content scales with player performance

### Procedural Generation Features
- **Level Layout**: Noise-based corridor generation with branching paths
- **Room Generation**: Procedural placement of specialized rooms
- **Decoration System**: Dynamic placement of environmental objects
- **Enemy Spawning**: Intelligent enemy placement based on game state
- **Streaming Technology**: Infinite level generation as players explore
- **Team positioning**: Effectiveness based on spacing
- **Resource sharing**: Ammo and equipment distribution  
- **Synchronized actions**: Coordinated breaching and switches
- **Communication**: Built-in team coordination systems

### Procedural Events
- **Environmental hazards**: Power outages, alarms
- **Dynamic encounters**: Randomized enemy reinforcements
- **Atmospheric events**: Audio cues and visual effects

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
✅ Basic player movement and controls  
✅ Combat system with multiple weapons  
✅ Enemy AI with different behaviors  
✅ Atmospheric environment with lighting  
✅ UI system with health/ammo display  
✅ Audio foundation  
✅ Networking architecture  
✅ Procedural level generation  
✅ Dynamic decoration placement  
✅ Intelligent enemy spawning  
✅ Tension and event systems  

🚧 Planned features:
- Complete multiplayer implementation
- Advanced procedural algorithms
- More enemy types and behaviors
- Enhanced cooperative mechanics
- Save/load system with procedural seeds
- Level editor and custom generation parameters
- Performance optimizations for infinite worldsect demonstrating core mechanics. Current features include:

✅ Basic player movement and controls  
✅ Combat system with multiple weapons  
✅ Enemy AI with different behaviors  
✅ Atmospheric environment with lighting  
✅ UI system with health/ammo display  
✅ Audio foundation  
✅ Networking architecture  

🚧 Planned features:
- Complete multiplayer implementation
- Level editor and custom maps
- More enemy types and behaviors
- Advanced cooperative mechanics
- Save/load system
- Settings and configuration
- Performance optimizations

## System Requirements

### Minimum
- OS: Windows 10, macOS 10.15, or Linux (Ubuntu 18.04+)
- Memory: 4 GB RAM
- Graphics: DirectX 11 or OpenGL 3.3 compatible
- Storage: 2 GB available space

### Recommended  
- OS: Windows 11, macOS 12+, or Linux (Ubuntu 20.04+)
- Memory: 8 GB RAM
- Graphics: Dedicated GPU with Vulkan support
- Storage: 4 GB available space
- Network: Broadband connection for multiplayer
