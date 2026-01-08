#!/bin/bash

# Get the directory where the monorepo Makefile is located
MONO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# Create the mono command script
cat > /tmp/mono << EOL
#!/bin/bash

# Change to the monorepo directory
cd "${MONO_DIR}"

# Pass all arguments to make
make "\$@"
EOL

# Make the script executable
chmod +x /tmp/mono

# Move to appropriate bin directory based on OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS - use /usr/local/bin
    sudo mv /tmp/mono /usr/local/bin/mono
else
    # Linux - use /usr/bin
    sudo mv /tmp/mono /usr/bin/mono
fi

echo "Mono CLI installed successfully!"
echo "You can now use 'mono' from any directory."
echo "Example: mono logs s=campus-lms" 