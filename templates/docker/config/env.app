# Application service configuration

# Server
APP_PORT=3000
APP_HOST=0.0.0.0

# Connection strings
# Derived from the generated credentials: scripts/generate-secrets.sh resolves
# these references into literals. Two reasons they live here and not in a compose
# `environment:` block: compose never interpolates an env_file value from a
# sibling env file (a reference in compose would render empty and warn
# `variable is not set`), and an `environment:` block outranks every env_file
# layer, so the same keys written there would overwrite these
# (bd:mech-crate-q1w).
DATABASE_URL=postgresql://${DB_USER}:${DB_PASSWORD}@${DB_HOST:-db}:${DB_PORT:-5432}/${DB_NAME}
# No password segment: the redis service ships no `requirepass`, and
# REDIS_PASSWORD is blank by design (see MECH_BLANK_BY_DESIGN_KEYS in
# scripts/.bashrc), so an embedded empty password is a reference nothing can
# resolve rather than a credential.
REDIS_URL=redis://${REDIS_HOST:-redis}:${REDIS_PORT:-6379}

# Node.js
NODE_ENV=development

# Features
ENABLE_DEBUG=true
ENABLE_HOT_RELOAD=true

# Limits
MAX_UPLOAD_SIZE=10mb
