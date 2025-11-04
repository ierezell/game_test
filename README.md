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

### Solo Mode
```bash
cargo run -- solo
```
or 
```bash
cargo run -- solo --client-id 1
```
Runs both client and server in the same process for single-player or local testing. Perfect for development and offline play.



### Levels
With the "generate procedural" the client AND the server generate the level with THE SAME SEED.
Then the server send dynamic elements to the client to replicate.