# Configuration Reference

This document describes all configuration options available for Simon.

## Configuration Methods

Simon can be configured through:
- Environment variables (recommended for containers)
- Command-line arguments (recommended for direct execution)

Command-line arguments take precedence over environment variables.

## Available Options

### Server Configuration

| Option | Environment Variable | CLI Flag | Default | Description |
|--------|---------------------|----------|---------|-------------|
| Address | `SIMON_ADDRESS` | `-a`, `--address` | `0.0.0.0` | Network address to bind the server to |
| Port | `SIMON_PORT` | `-p`, `--port` | `30000` | Port to bind the server to |
| Update Interval | `SIMON_UPDATE_INTERVAL` | `-T`, `--update-interval` | `2` | Metrics update interval in seconds (1-30) |

### Authentication

| Option | Environment Variable | CLI Flag | Default | Description |
|--------|---------------------|----------|---------|-------------|
| Password Hash | `SIMON_PASSWORD_HASH` | `-H`, `--password-hash` | None | Bcrypt password hash for authentication. If not set, authentication is disabled |

### Storage & Data

| Option | Environment Variable | CLI Flag | Default | Description |
|--------|---------------------|----------|---------|-------------|
| Database Path | `SIMON_DB_PATH` | `--db-path` | `./simon-data/simon.db` | Path to SQLite database file |
| Serve Directories | `SIMON_SERVE_DIRS` | `--serve-dir` | None | Comma-separated list of directories to serve via file browser |

## Examples

### Using Environment Variables

```bash
export SIMON_PORT=8080
export SIMON_UPDATE_INTERVAL=5
export SIMON_PASSWORD_HASH='$2a$12$...'
export SIMON_SERVE_DIRS='/var/log,/home/user/documents'
./simon
```

### Using Command-Line Arguments

```bash
./simon \
  --port 8080 \
  --update-interval 5 \
  --password-hash '$2a$12$...' \
  --serve-dir /var/log \
  --serve-dir /home/user/documents
```

### Using Docker

```bash
docker run -d \
  -e SIMON_PORT=8080 \
  -e SIMON_UPDATE_INTERVAL=5 \
  -e SIMON_PASSWORD_HASH='$$2a$$12$$...' \
  -e SIMON_SERVE_DIRS='/var/log,/home/user/documents' \
  -p 8080:8080 \
  alibahmanyar/simon
```

Note: In Docker Compose files, dollar signs must be escaped as `$$`.

## Security Best Practices

1. **Always enable authentication** when exposing Simon to untrusted networks
2. **Use strong passwords** and generate secure bcrypt hashes
3. **Restrict file browser access** by carefully selecting served directories
4. **Use HTTPS** when deploying with a reverse proxy
5. **Limit network access** using firewall rules or Docker network policies
