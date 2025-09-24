# Compile 
Write-Host "Compiling the project..."
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "build" -Wait
Write-Host "Compilation finished."

# Start the server in a background process
Write-Host "Starting the server..."
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run -- server"

# Wait for the server to initialize
Write-Host "Waiting for the server to initialize..."
Start-Sleep -Seconds 3

# Start the client in a new window
Write-Host "Starting the first client..."
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run -- client --client-id 1 --autoconnect"

Write-Host "Starting the second client..."
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run -- client --client-id 2 --autoconnect"

# Optionally, wait for user input to keep the script open
Write-Host "Server and client launched. Press Escape to exit them."
