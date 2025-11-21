---
name: Rust commands
description: Useful Rust commands to use in this project
---

# Project rust commands

## Instructions
Use this commands to interact with the project.

## Running the game, with auto hosting, auto start the game and kill it after 15 seconds
`cargo run --bin launcher -- client --auto-host --auto-start --stop-after 15`

## Running the game, with auto hosting, auto start the game
`cargo run --bin launcher -- client --auto-host --auto-start --stop-after 15`

## Run the server
`cargo run --bin launcher -- server --headless`

## Run tests (with only one thread as bevy ask for it.)
`cargo test -- --test-threads=1`

## Check tests
`cargo check --tests`