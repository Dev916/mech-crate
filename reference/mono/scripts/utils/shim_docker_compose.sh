#!/bin/bash

# Check that docker-compose exists
if ! command -v docker-compose >/dev/null; then
  echo "❌ 'docker-compose' is not installed. Aborting."
  exit 1
fi

# Check that docker exists
if ! command -v docker >/dev/null; then
  echo "❌ 'docker' is not installed. Aborting."
  exit 1
fi

# Check if the shim already exists
if [ -f /usr/local/bin/docker ]; then
  echo "⚠️  '/usr/local/bin/docker' already exists. Not overwriting."
  exit 1
fi

# Ask for confirmation
echo "This will create a shim so that running 'docker compose' will call 'docker-compose' instead."
read -p "Do you want to proceed? (y/N): " confirm
if [[ "$confirm" != [yY] ]]; then
  echo "❌ Cancelled."
  exit 1
fi

# Create the shim
sudo tee /usr/local/bin/docker >/dev/null <<'EOF'
#!/bin/bash
if [[ "$1" == "compose" ]]; then
  shift
  exec docker-compose "$@"
else
  exec /usr/bin/docker "$@"
fi
EOF

# Make it executable
sudo chmod +x /usr/local/bin/docker

echo "✅ Shim created at /usr/local/bin/docker"
echo "Now you can use 'docker compose' as an alias for 'docker-compose'"