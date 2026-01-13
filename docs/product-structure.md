# MechCrate Project Structure

```
project-name/
├── Makefile              # Root makefile
├── apps/                 # Application source code
│   └── <service>/        # Each service's source
│       ├── src/          # Source code
│       ├── package.json  # Dependencies
│       └── ...
├── make/                 # Make modules
│   ├── common.mk         # Shared helpers
│   ├── dev.mk            # Development commands
│   ├── up.mk             # Service management
│   └── ...
├── scripts/              # Shell scripts
│   ├── .bashrc           # Helper functions
│   ├── dev.sh            # Development script
│   └── ...
├── docker/
│   ├── .config/          # Environment files
│   │   ├── .env.shared   # Shared config
│   │   ├── .env.secrets  # Secrets (gitignored)
│   │   └── .env.<svc>    # Per-service config
│   ├── compose/          # Compose files
│   │   ├── <svc>.yml     # Service baseline
│   │   └── <svc>.dev.yml # Dev overrides
│   ├── system/           # System-level files
│   │   └── <service>/    # Maps to container /
│   │       ├── etc/      # Config files (supervisor, nginx)
│   │       └── usr/      # Scripts (entrypoint)
│   └── dockerfiles/      # Dockerfiles
│       └── <service>/
│           └── app       # Dockerfile
└── infra/                # Infrastructure (optional)
    └── cloudflare/       # Cloudflare workers
```

## Key Conventions

1. **Build Context**: Dockerfiles are built from project root
2. **System Files**: `docker/system/<svc>/` mirrors container filesystem
3. **Networking**: Services join `devmesh-traefik` network for routing
4. **Labels**: Traefik labels define hostname routing rules
