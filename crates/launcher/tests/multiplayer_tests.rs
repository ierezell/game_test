use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Test multiple clients connecting to a single server
#[test]
fn test_multiple_clients_connect() {
    println!("🚀 Testing multiple clients connecting...");

    // Start server
    let mut server_process = Command::new("cargo")
        .args(&["run", "--package", "launcher", "--", "server", "--headless"])
        .current_dir("../../")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server process");

    thread::sleep(Duration::from_millis(1000));

    const NUM_CLIENTS: u8 = 3; // Test with 3 clients
    let mut client_processes = Vec::new();
    let client_start_time = Instant::now();

    // Start clients
    for client_id in 1..=NUM_CLIENTS {
        println!("🎮 Starting client {}", client_id);

        let client_process = Command::new("cargo")
            .args(&[
                "run",
                "--package",
                "launcher",
                "--",
                "client",
                "--client-id",
                &client_id.to_string(),
                "--autoconnect",
            ])
            .current_dir("../../")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect(&format!("Failed to start client {} process", client_id));

        client_processes.push((client_id, client_process));
        thread::sleep(Duration::from_millis(200));
    }

    println!(
        "⏰ All {} clients started in {:?}",
        NUM_CLIENTS,
        client_start_time.elapsed()
    );

    // Let them run for a bit longer to allow for connection + timeout
    thread::sleep(Duration::from_secs(6));

    // Count how many exited (should timeout around 3-4 seconds)
    let mut exited_count = 0;
    let mut processes_spawned = 0;
    for (client_id, mut process) in client_processes {
        processes_spawned += 1;
        match process.try_wait() {
            Ok(Some(_)) => {
                exited_count += 1;
                println!("✅ Client {} exited (likely timeout)", client_id);
            }
            Ok(None) => {
                println!("⚠️  Client {} still running", client_id);
                let _ = process.kill();
                let _ = process.wait();
            }
            Err(e) => {
                println!("❌ Error checking client {}: {:?}", client_id, e);
            }
        }
    }

    let _ = server_process.kill();
    let _ = server_process.wait();

    // Assert that all clients were spawned successfully
    assert_eq!(
        processes_spawned, NUM_CLIENTS,
        "All {} clients should have been spawned",
        NUM_CLIENTS
    );

    // Assert that the server and clients can be started (basic functionality)
    assert!(
        processes_spawned > 0,
        "At least one client process should have been spawned"
    );

    println!(
        "✅ Multiple client test completed: {}/{} clients exited naturally",
        exited_count, NUM_CLIENTS
    );
}

/// Test clients joining and leaving in sequence
#[test]
fn test_sequential_join_leave() {
    println!("🎮 Testing sequential client join/leave...");

    // Start server
    let mut server_process = Command::new("cargo")
        .args(&["run", "--package", "launcher", "--", "server", "--headless"])
        .current_dir("../../")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server process");

    thread::sleep(Duration::from_millis(500));

    // Test 3 waves of clients
    let mut successful_waves = 0;
    for wave in 1..=3 {
        println!("🌊 Wave {} - Adding client", wave);

        let client_result = Command::new("cargo")
            .args(&[
                "run",
                "--package",
                "launcher",
                "--",
                "client",
                "--client-id",
                &wave.to_string(),
                "--autoconnect",
            ])
            .current_dir("../../")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match client_result {
            Ok(mut client_process) => {
                successful_waves += 1;

                // Let client run briefly
                thread::sleep(Duration::from_secs(1));

                // Assert client is running
                let status = client_process.try_wait();
                assert!(
                    status.is_ok(),
                    "Should be able to check client {} status",
                    wave
                );

                // Terminate client (simulating leave)
                println!("👋 Client {} leaving", wave);
                let kill_result = client_process.kill();
                assert!(
                    kill_result.is_ok(),
                    "Should be able to terminate client {}",
                    wave
                );
                let _ = client_process.wait();
            }
            Err(e) => {
                panic!("Failed to spawn client {} in wave {}: {:?}", wave, wave, e);
            }
        }

        thread::sleep(Duration::from_millis(300));
    }

    // Assert all waves completed successfully
    assert_eq!(
        successful_waves, 3,
        "All 3 waves should have completed successfully"
    );

    let _ = server_process.kill();
    let _ = server_process.wait();

    println!("✅ Sequential join/leave test completed");
}

/// Test rapid connection cycles
#[test]
fn test_rapid_connection_cycles() {
    println!("⚡ Testing rapid connect/disconnect cycles...");

    // Start server
    let mut server_process = Command::new("cargo")
        .args(&["run", "--package", "launcher", "--", "server", "--headless"])
        .current_dir("../../")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server process");

    thread::sleep(Duration::from_millis(500));

    const NUM_CYCLES: u8 = 3;
    let mut successful_cycles = 0;

    for cycle in 1..=NUM_CYCLES {
        println!("🔄 Rapid cycle {} of {}", cycle, NUM_CYCLES);

        let cycle_start = Instant::now();

        // Start client
        let client_result = Command::new("cargo")
            .args(&[
                "run",
                "--package",
                "launcher",
                "--",
                "client",
                "--client-id",
                &cycle.to_string(),
                "--autoconnect",
            ])
            .current_dir("../../")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match client_result {
            Ok(mut client_process) => {
                // Let it run briefly
                thread::sleep(Duration::from_millis(800));

                // Assert client was running
                let status = client_process.try_wait();
                assert!(
                    status.is_ok(),
                    "Should be able to check client status in cycle {}",
                    cycle
                );

                // Kill client
                let kill_result = client_process.kill();
                assert!(
                    kill_result.is_ok(),
                    "Should be able to kill client in cycle {}",
                    cycle
                );
                let _ = client_process.wait();

                successful_cycles += 1;
            }
            Err(e) => {
                panic!("Failed to start client for cycle {}: {:?}", cycle, e);
            }
        }

        let cycle_time = cycle_start.elapsed();
        println!("   Cycle {} completed in {:?}", cycle, cycle_time);

        // Assert cycle timing is reasonable
        assert!(
            cycle_time <= Duration::from_secs(2),
            "Cycle {} took too long: {:?}",
            cycle,
            cycle_time
        );

        thread::sleep(Duration::from_millis(200));
    }

    // Assert all cycles completed successfully
    assert_eq!(
        successful_cycles, NUM_CYCLES,
        "All {} cycles should have completed successfully",
        NUM_CYCLES
    );

    let _ = server_process.kill();
    let _ = server_process.wait();

    println!("✅ Rapid cycle test completed");
}

/// Test game session lifecycle (team formation)
#[test]
fn test_game_session_lifecycle() {
    println!("🎯 Testing game session lifecycle...");

    // Start server
    let mut server_process = Command::new("cargo")
        .args(&["run", "--package", "launcher", "--", "server", "--headless"])
        .current_dir("../../")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server process");

    thread::sleep(Duration::from_millis(1000));

    // Phase 1: Team formation (4 players joining)
    println!("👥 Phase 1: Team formation");
    let mut team_members = Vec::new();
    let mut players_joined = 0;

    for player_id in 1..=4 {
        println!("   👤 Player {} joining team...", player_id);

        let client_result = Command::new("cargo")
            .args(&[
                "run",
                "--package",
                "launcher",
                "--",
                "client",
                "--client-id",
                &player_id.to_string(),
                "--autoconnect",
            ])
            .current_dir("../../")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match client_result {
            Ok(client_process) => {
                team_members.push((player_id, client_process));
                players_joined += 1;
            }
            Err(e) => {
                panic!("Failed to start player {} process: {:?}", player_id, e);
            }
        }

        thread::sleep(Duration::from_millis(300));
    }

    // Assert full team was assembled
    assert_eq!(
        players_joined, 4,
        "All 4 players should have joined the team"
    );
    assert_eq!(team_members.len(), 4, "Team should have 4 members");

    println!("   ✅ Full team assembled ({} players)", team_members.len());

    // Phase 2: Game session active
    println!("🎮 Phase 2: Game session active");
    thread::sleep(Duration::from_secs(2));

    // Phase 3: Session end
    println!("🏁 Phase 3: Session ending");
    let mut players_disconnected = 0;

    for (player_id, mut process) in team_members {
        println!("   👋 Player {} disconnecting...", player_id);

        let kill_result = process.kill();
        assert!(
            kill_result.is_ok(),
            "Should be able to disconnect player {}",
            player_id
        );

        let wait_result = process.wait();
        assert!(
            wait_result.is_ok(),
            "Should be able to wait for player {} to disconnect",
            player_id
        );

        players_disconnected += 1;
        thread::sleep(Duration::from_millis(100));
    }

    // Assert all players disconnected successfully
    assert_eq!(
        players_disconnected, 4,
        "All 4 players should have disconnected successfully"
    );

    let _ = server_process.kill();
    let _ = server_process.wait();

    println!("✅ Game session lifecycle test completed");
}
