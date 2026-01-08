#!/bin/bash
#
# MechCrate Add Service Command
# Add a new service to an existing project
#

# Add service to existing project
add_service() {
    local service_name="$1"
    
    if [[ -z "$service_name" ]]; then
        error "Service name is required. Usage: mx add <service-name>"
    fi
    
    if ! is_mech_crate_project; then
        error "Not in a MechCrate project. Run 'mx new <name>' first."
    fi
    
    add_service_internal "$service_name"
}

add_service_internal() {
    local service_name="$1"
    local service_upper=$(echo "$service_name" | tr '[:lower:]' '[:upper:]' | tr '-' '_')
    
    info "Adding service: ${BOLD}$service_name${NC}"
    
    # Create app source directory
    mkdir -p "apps/$service_name/src"
    
    # Create system directories (configs, logs, etc.)
    mkdir -p "docker/system/$service_name/etc/$service_name"
    mkdir -p "docker/system/$service_name/var/log/$service_name"
    mkdir -p "docker/dockerfiles/$service_name"
    
    # Create base compose file
    cat > "docker/compose/$service_name.yml" << EOF
services:
  $service_name:
    build:
      context: ../..
      dockerfile: docker/dockerfiles/$service_name/app
    container_name: $service_name
    env_file:
      - ../.config/.env.shared
      - ../.config/.env.secrets
      - ../.config/.env.$service_name
    ports:
      - "\${${service_upper}_PORT:-3000}:3000"
    volumes:
      - ../system/$service_name/:/
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 3s
      retries: 3

networks:
  default:
    name: mech-network
    external: true
EOF

    # Create dev compose override
    cat > "docker/compose/$service_name.dev.yml" << EOF
services:
  $service_name:
    build:
      target: development
    environment:
      - NODE_ENV=development
      - LOG_LEVEL=debug
    ports:
      - "\${${service_upper}_PORT:-3000}:3000"
      - "9229:9229"  # Debugger
    volumes:
      # Mount app source for hot-reload
      - ../../apps/$service_name/src:/app/src:cached
      - ../../apps/$service_name/package.json:/app/package.json:ro
      # Volume mount over node_modules for performance
      - ${service_name}_node_modules:/app/node_modules
    restart: "no"
    healthcheck:
      disable: true

volumes:
  ${service_name}_node_modules:
EOF

    # Create service env file
    cat > "docker/.config/.env.$service_name" << EOF
# $service_name configuration
${service_upper}_PORT=3000
${service_upper}_LOG_LEVEL=info
EOF

    # Create sample Dockerfile
    cat > "docker/dockerfiles/$service_name/app" << EOF
# ----- Build Stage -----
FROM node:20-alpine AS builder
WORKDIR /app

# Install dependencies (from apps/$service_name)
COPY apps/$service_name/package*.json ./
RUN npm ci

# Copy source and build
COPY apps/$service_name/ .
RUN npm run build

# ----- Development Stage -----
FROM node:20-alpine AS development
WORKDIR /app

COPY apps/$service_name/package*.json ./
RUN npm install

COPY apps/$service_name/ .

EXPOSE 3000 9229
CMD ["npm", "run", "dev"]

# ----- Production Stage -----
FROM node:20-alpine AS production
WORKDIR /app

RUN addgroup -g 1001 -S nodejs && \\
    adduser -S nodejs -u 1001

COPY --from=builder --chown=nodejs:nodejs /app/dist ./dist
COPY --from=builder --chown=nodejs:nodejs /app/node_modules ./node_modules
COPY --from=builder --chown=nodejs:nodejs /app/package.json ./

# Copy system files (configs, etc.)
COPY docker/system/$service_name/ /

USER nodejs
EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \\
    CMD node -e "require('http').get('http://localhost:3000/health', (r) => process.exit(r.statusCode === 200 ? 0 : 1))"

CMD ["node", "dist/index.js"]
EOF

    # Create sample package.json for the app
    cat > "apps/$service_name/package.json" << EOF
{
  "name": "$service_name",
  "version": "0.0.1",
  "type": "module",
  "scripts": {
    "dev": "node --watch src/index.js",
    "build": "echo 'Add your build step here'",
    "start": "node dist/index.js"
  },
  "engines": {
    "node": ">=20"
  }
}
EOF

    # Create sample index.js
    cat > "apps/$service_name/src/index.js" << EOF
// $service_name service entry point
import http from 'http';

const PORT = process.env.PORT || 3000;

const server = http.createServer((req, res) => {
  if (req.url === '/health') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ status: 'ok', service: '$service_name' }));
    return;
  }
  
  res.writeHead(200, { 'Content-Type': 'text/plain' });
  res.end('Hello from $service_name!');
});

server.listen(PORT, () => {
  console.log(\`$service_name listening on port \${PORT}\`);
});
EOF

    success "Service '$service_name' added!"
    echo ""
    info "Created files:"
    echo "    apps/$service_name/                  # App source code"
    echo "    apps/$service_name/package.json"
    echo "    apps/$service_name/src/index.js"
    echo "    docker/compose/$service_name.yml"
    echo "    docker/compose/$service_name.dev.yml"
    echo "    docker/.config/.env.$service_name"
    echo "    docker/dockerfiles/$service_name/app"
    echo "    docker/system/$service_name/         # System files"
}
