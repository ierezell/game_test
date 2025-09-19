# Start the server in a background process
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run -- server"

# Wait for the server to initialize
Start-Sleep -Seconds 3

# Start the client in a new window
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run -- client --client-id 1 --autoconnect"
Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run -- client --client-id 2 --autoconnect"

# Optionally, wait for user input to keep the script open
Write-Host "Server and client launched. Press Enter to exit script."
Read-Host
