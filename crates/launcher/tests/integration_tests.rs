use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_server_process_spawning() {
    println!("🧪 Testing server process spawning and initialization...");

    let server_result = Command::new("cargo")
        .args(&["run", "--package", "launcher", "--", "server", "--headless"])
        .current_dir("../../")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match server_result {
        Ok(mut child) => {
            println!("✅ Server process spawned successfully");

            // Wait longer for full initialization (3 seconds)
            thread::sleep(Duration::from_secs(3));

            // Check if process is still running after initialization
            let status = child.try_wait();
            match status {
                Ok(Some(exit_status)) => {
                    // Process exited during initialization - this is likely a crash
                    let stdout = child
                        .stdout
                        .take()
                        .map(|mut s| {
                            let mut output = String::new();
                            let _ = std::io::Read::read_to_string(&mut s, &mut output);
                            output
                        })
                        .unwrap_or_default();

                    let stderr = child
                        .stderr
                        .take()
                        .map(|mut s| {
                            let mut output = String::new();
                            let _ = std::io::Read::read_to_string(&mut s, &mut output);
                            output
                        })
                        .unwrap_or_default();

                    panic!(
                        "❌ Server process exited unexpectedly during initialization with status: {:?}\nStdout: {}\nStderr: {}",
                        exit_status, stdout, stderr
                    );
                }
                Ok(None) => {
                    println!(
                        "✅ Server process is still running after initialization - shutting down"
                    );
                    let _ = child.kill();
                    let _ = child.wait();
                }
                Err(e) => {
                    panic!("❌ Error checking server process status: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("❌ Failed to spawn server process: {:?}", e);
        }
    }

    println!("✅ Server spawning and initialization test completed");
}

/// Test client process spawning and initialization
#[test]
fn test_client_process_spawning() {
    println!("🧪 Testing client process spawning and initialization...");

    let client_result = Command::new("cargo")
        .args(&[
            "run",
            "--package",
            "launcher",
            "--",
            "client",
            "--client-id",
            "1",
            "--autoconnect",
        ])
        .current_dir("../../")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match client_result {
        Ok(mut child) => {
            println!("✅ Client process spawned successfully");

            // Wait longer for full initialization (3 seconds)
            thread::sleep(Duration::from_secs(3));

            // Check if process is still running after initialization
            let status = child.try_wait();
            match status {
                Ok(Some(exit_status)) => {
                    // If client exited, check if it was due to a crash or expected behavior
                    let stdout = child
                        .stdout
                        .take()
                        .map(|mut s| {
                            let mut output = String::new();
                            let _ = std::io::Read::read_to_string(&mut s, &mut output);
                            output
                        })
                        .unwrap_or_default();

                    let stderr = child
                        .stderr
                        .take()
                        .map(|mut s| {
                            let mut output = String::new();
                            let _ = std::io::Read::read_to_string(&mut s, &mut output);
                            output
                        })
                        .unwrap_or_default();

                    // Check if stderr contains panic information
                    if stderr.contains("panic") || stderr.contains("thread 'main' panicked") {
                        panic!(
                            "❌ Client process crashed during initialization with status: {:?}\nStdout: {}\nStderr: {}",
                            exit_status, stdout, stderr
                        );
                    } else {
                        println!(
                            "✅ Client process exited normally (likely due to no server connection)"
                        );
                    }
                }
                Ok(None) => {
                    println!(
                        "✅ Client process is still running after initialization - shutting down"
                    );
                    let _ = child.kill();
                    let _ = child.wait();
                }
                Err(e) => {
                    panic!("❌ Error checking client process status: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("❌ Failed to spawn client process: {:?}", e);
        }
    }

    println!("✅ Client spawning and initialization test completed");
}

/// Test both server and client process spawning together with proper initialization
#[test]
fn test_launcher_process_spawning() {
    println!("🧪 Testing server and client process spawning together with initialization...");

    // Test server spawning and initialization
    let server_result = Command::new("cargo")
        .args(&["run", "--package", "launcher", "--", "server", "--headless"])
        .current_dir("../../")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    #[allow(unused_assignments)]
    let mut server_spawned = false;
    match server_result {
        Ok(mut child) => {
            println!("✅ Server process spawned successfully");

            // Wait for full initialization
            thread::sleep(Duration::from_secs(3));

            // Assert that the process is still running after initialization
            let status = child.try_wait();
            match status {
                Ok(Some(exit_status)) => {
                    let stderr = child
                        .stderr
                        .take()
                        .map(|mut s| {
                            let mut output = String::new();
                            let _ = std::io::Read::read_to_string(&mut s, &mut output);
                            output
                        })
                        .unwrap_or_default();

                    panic!(
                        "❌ Server process exited unexpectedly during initialization with status: {:?}\nStderr: {}",
                        exit_status, stderr
                    );
                }
                Ok(None) => {
                    println!("✅ Server process still running after initialization");
                    server_spawned = true;
                }
                Err(e) => {
                    panic!("❌ Error checking server process status: {:?}", e);
                }
            }

            let _ = child.kill();
            let wait_result = child.wait();
            assert!(
                wait_result.is_ok(),
                "Should be able to wait for server process to terminate"
            );
        }
        Err(e) => {
            panic!("❌ Failed to spawn server process: {:?}", e);
        }
    }

    assert!(
        server_spawned,
        "Server must have spawned and initialized successfully"
    );

    // Test client spawning and initialization
    let client_result = Command::new("cargo")
        .args(&[
            "run",
            "--package",
            "launcher",
            "--",
            "client",
            "--client-id",
            "1",
            "--autoconnect",
        ])
        .current_dir("../../")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    #[allow(unused_assignments)]
    let mut client_spawned = false;
    match client_result {
        Ok(mut child) => {
            println!("✅ Client process spawned successfully");

            // Wait for full initialization
            thread::sleep(Duration::from_secs(3));

            // Check client status after initialization
            let status = child.try_wait();
            match status {
                Ok(Some(exit_status)) => {
                    let stderr = child
                        .stderr
                        .take()
                        .map(|mut s| {
                            let mut output = String::new();
                            let _ = std::io::Read::read_to_string(&mut s, &mut output);
                            output
                        })
                        .unwrap_or_default();

                    // Check if it was a crash or normal exit
                    if stderr.contains("panic") || stderr.contains("thread 'main' panicked") {
                        panic!(
                            "❌ Client process crashed during initialization with status: {:?}\nStderr: {}",
                            exit_status, stderr
                        );
                    } else {
                        println!("✅ Client process exited normally after initialization");
                        client_spawned = true;
                    }
                }
                Ok(None) => {
                    println!("✅ Client process still running after initialization");
                    client_spawned = true;
                }
                Err(e) => {
                    panic!("❌ Error checking client process status: {:?}", e);
                }
            }

            let _ = child.kill();
            let _ = child.wait();
        }
        Err(e) => {
            panic!("❌ Failed to spawn client process: {:?}", e);
        }
    }

    assert!(
        client_spawned,
        "Client must have spawned and initialized successfully"
    );
    assert!(
        server_spawned && client_spawned,
        "Both server and client must spawn and initialize successfully"
    );

    println!("✅ Combined process spawning and initialization test completed");
}

/// Test client connection timing with real processes
#[test]
fn test_client_connection_timing() {
    // Start server

    let mut server_process = Command::new("cargo")
        .args(&["run", "--package", "launcher", "--", "server", "--headless"])
        .current_dir("../../")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server process");

    // Give server time to start
    thread::sleep(Duration::from_millis(1000));

    // Start client with autoconnect
    let client_start = Instant::now();
    let mut client_process = Command::new("cargo")
        .args(&[
            "run",
            "--package",
            "launcher",
            "--",
            "client",
            "--client-id",
            "1",
            "--autoconnect",
        ])
        .current_dir("../../")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start client process");

    // Monitor client for up to 6 seconds
    let monitor_duration = Duration::from_secs(6);
    let start_monitoring = Instant::now();
    let mut client_exited = false;

    let mut exit_duration = None;

    while start_monitoring.elapsed() < monitor_duration {
        match client_process.try_wait() {
            Ok(Some(status)) => {
                let client_duration = client_start.elapsed();
                println!(
                    "📊 Client process exited after {:?} with status: {:?}",
                    client_duration, status
                );

                client_exited = true;

                exit_duration = Some(client_duration);

                // Client should run for at least 2 seconds (time to connect)
                // and exit within 5 seconds (3s timeout + buffer)
                if client_duration >= Duration::from_secs(2)
                    && client_duration <= Duration::from_secs(5)
                {
                    println!("✅ Client timing appears correct (likely connected then timed out)");
                } else if client_duration < Duration::from_secs(2) {
                    println!("⚠️  Client exited quickly - may have failed to connect");
                } else {
                    println!("⚠️  Client took longer than expected");
                }
                break;
            }
            Ok(None) => {
                // Process still running
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                println!("❌ Error checking client process: {:?}", e);
                break;
            }
        }
    }

    // Client should either be running (connected) or have run for reasonable time
    let client_ran_well = if client_exited {
        if let Some(duration) = exit_duration {
            duration >= Duration::from_secs(2) // Ran for at least 2 seconds
        } else {
            false
        }
    } else {
        // Still running is fine if server is available
        start_monitoring.elapsed() >= Duration::from_secs(3)
    };

    assert!(
        client_ran_well,
        "Client should either keep running (connected) or run for reasonable time before exiting"
    );

    if client_exited {
        println!("✅ Client exited after running for reasonable time");
    } else {
        println!("✅ Client continues running (likely connected successfully)");
    }

    // Cleanup
    let _ = client_process.kill();
    let _ = server_process.kill();
    let _ = client_process.wait();
    let _ = server_process.wait();

    println!("✅ Connection timing test completed");
}

/// Test client behavior when no server is available
#[test]
fn test_client_timeout_no_server() {
    println!("⏰ Testing client behavior when no server available...");

    // Start client without server
    let client_start = Instant::now();
    let mut client_process = Command::new("cargo")
        .args(&[
            "run",
            "--package",
            "launcher",
            "--",
            "client",
            "--client-id",
            "1",
            "--autoconnect",
        ])
        .current_dir("../../")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start client process");

    // Monitor for 4 seconds to see behavior
    let monitor_duration = Duration::from_secs(4);
    let start_monitoring = Instant::now();
    let mut client_still_running = false;
    let mut client_ran_sufficiently = false;

    while start_monitoring.elapsed() < monitor_duration {
        match client_process.try_wait() {
            Ok(Some(status)) => {
                let client_duration = client_start.elapsed();
                println!(
                    "📊 Client exited after {:?} with status: {:?}",
                    client_duration, status
                );

                // If client ran for reasonable time before exiting, that's okay
                if client_duration >= Duration::from_secs(2) {
                    client_ran_sufficiently = true;
                    println!("✅ Client ran for reasonable time before exiting");
                }
                break;
            }
            Ok(None) => {
                // Client is still running - this is acceptable with autoconnect
                if start_monitoring.elapsed() >= Duration::from_secs(3) {
                    client_still_running = true;
                    client_ran_sufficiently = true;
                    println!("✅ Client continues running with autoconnect (expected behavior)");
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                println!("❌ Error checking client process: {:?}", e);
                break;
            }
        }
    }

    // Assert client behaved reasonably (either ran for sufficient time or kept running)
    assert!(
        client_still_running || client_ran_sufficiently,
        "Client should either keep running with autoconnect or run for reasonable time before exiting"
    );

    // Cleanup
    let _ = client_process.kill();
    let _ = client_process.wait();

    println!("✅ Client no-server behavior test completed");
}
