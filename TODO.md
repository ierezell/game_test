AT EACH STEP ENSURE THAT cargo build / check pass and that we have tests to cover for everything, edge cases, etc.... 
- [ ] Merge render and camera for client
- [ ] Clean a bit more the code and abstraction, regroup stuff, have a clean interface for walls floor and player that implement trait to give their meshes or physics so it can be shared when isntanciating.  
- [ ] Add player controler with https://github.com/idanarye/bevy-tnua
- [ ] Add ennemy
- [ ] Add https://github.com/jtothethree/bevy_northstar for ennemy navigation ?  
- [ ] Add health logic
- [ ] Add guns and damages
- [ ] Add a bot that use the same actionState as the player
- [ ] Do reinforcement learning to control this bot (with burn https://github.com/tracel-ai/burn)
- [ ] Add stamina logic
- [ ] Do we want that ? https://github.com/cBournhonesque/lightyear/tree/main/examples/client_replication  (bullets, player spawn etc..)

---
mode: 'agent'
description: 'Review and refactor code in your project according to defined instructions'
---

## Role

You're a senior expert software engineer with extensive experience in maintaining projects over a long time and ensuring clean code and best practices. 

## Task

1. Take a deep breath, and review all coding guidelines instructions in `.github/instructions/*.md` and `.github/copilot-instructions.md`, then review all the code carefully and make code refactorings if needed.
2. The final code should be clean and maintainable while following the specified coding standards and instructions.
3. Do not split up the code, keep the existing files intact.
4. If the project includes tests, ensure they are still passing after your changes.




Ok, now that you implemented a lot of features (+5000 lines), please, review all the libraries, and all the codebase and ensure that: 
- We didn't rewrite something a library is already providing
- We keep it KISS and DRY
- There is no unused, repeated, bloated code
- Clean the codebase, merge similar things, and keep it as simple and maintainable as possible. 
- Finally run `cargo check` and `cargo test` to ensure everything is clean