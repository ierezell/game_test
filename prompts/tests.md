As bevy is an ECS, we need tests to ensure that when running: 

cargo run --bin launcher -- client --auto-host --auto-start --gym
or 
cargo run --bin launcher -- client --auto-host --auto-start

Then all the pieces are tested and validated so we know nothing will break once multiple players join and play. 

We need to test : 
- Level creation
- Physics
- Movement
- Camera
- Networking
- Etc...

So when we launch the game all the ECS components are tested and we know game is running properly. 