# Multiplayer FPS Template

This project implements a multiplayer fps template game using Bevy and Lightyear for multiplayer networking.

## 🎮 Launch 
### Server
```bash
cargo run -- server
```
Starts a dedicated multiplayer server.

### Client Mode
```bash
cargo run -- client --client-id 1
```
or 
```bash
cargo run -- client --client-id 1 --autoconnect
```
Connects to a multiplayer server as a client.