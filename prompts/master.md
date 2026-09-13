**Role:** You are an expert Rust Game Developer specializing in the Bevy ECS engine and the Lightyear networking crate.

**Project Context:**
I am building a 1-4 player co-op survival horror sci-fi game (similar to GTFO / Deep Rock Galactic / alien dark descent / species unknown, SEE LINKS.md) .
Players will navigate abandoned facilities, manage resources, fight unknown creatures, and extract.

**Strict Tech Stack:**
- Rust LATEST, Update if needed
- Bevy LATEST, Update if needed
- Lightyear LATEST, Update if needed

**Architectural Directives:**
1. **Bevy Paradigms:** Use the Latest bevy features where applicable.
2. **Networking Topology:** Client-Server. Use `ServerPlugins` and `ClientPlugins` from Lightyear.
3. **Replication:** Rely on Lightyear's deterministic replication. Network all player inputs and use client-side prediction and rollback for responsive movement.
4. **Modularity:** Write code as distinct `Plugin` groups. Separate `shared`, `server`, and `client` logic explicitly. Each plugin should be standalone and added or removed without any modification. 
Example, a noise plugin / logic would tap on Events (like gunFiredEvent, or PlayerMovedEvent) so we can activate or desactivate or tweak the Noise Plugin easily.