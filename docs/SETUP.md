# Setup Guide

This guide covers various installation and deployment options for Simon.

## Table of Contents

- [Using Prebuilt Binaries](#using-prebuilt-binaries)
- [Using Docker](#using-docker)
- [Using Docker Compose](#using-docker-compose)
- [Running Behind a Reverse Proxy](#running-behind-a-reverse-proxy)
  - [Nginx Configuration](#nginx-configuration)
  - [Traefik Configuration](#traefik-configuration)
  - [Base Path Configuration](#base-path-configuration)
- [Authentication](#authentication)

## Using Prebuilt Binaries

The simplest way to install Simon is using prebuilt binaries:

Download the latest release for your platform from the [Releases](https://github.com/alibahmanyar/simon/releases) page.

```bash
chmod +x simon
./simon
```

Just run the binary and Simon will start monitoring! The web UI will be available at http://localhost:30000.

## Using Docker

```bash
docker run -d \
  --name simon \
  -p 30000:30000 \
  -v /sys:/sys:ro \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  -v /:/fs:ro \
  -v ./simon-data:/app/simon-data \
  alibahmanyar/simon
```

## Using Docker Compose

Create a `docker-compose.yml` file:

```yaml
services:
  simon:
    image: alibahmanyar/simon
    hostname: simon # Set container hostname (replace with your own)
    ports:
      - "30000:30000"
    environment:
      # Authentication configuration
      # Bcrypt hash for 'secret', replace with your own hash or remove to disable authentication
      # Note: Dollar signs need to be escaped with additional dollar signs in Docker Compose files
      SIMON_PASSWORD_HASH: "$$2a$$12$$nmCGsgJ3ovx76sc/J8Bcs.Vn235KLQK7Cze83Kzm36a1v59QKVOO."
    volumes:
      - /sys:/sys:ro
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - /:/fs:ro
      - ./simon-data:/app/simon-data
```

Then run:
```bash
docker-compose up -d
```

**Important Notes:**

1. **Docker Access**: For accessing Docker stats, the user account running Simon needs access to the Docker socket (`/var/run/docker.sock`). This can be achieved by:
   - Using a user that belongs to the `docker` group
   - Running as root (not recommended for production)

2. **System Information**: For accurate system information, mount relevant filesystem paths:
   - Mount `/etc/lsb-release` or similar OS identification files for correct OS detection
   - Mount `/sys` for hardware, network and process information
   - Mount filesystems you want to monitor (e.g., `/` as `/fs`) for disk usage statistics

3. **Password Hash**: The password hash should be provided correctly; pay attention to escaping special characters (like dollar signs) in the hash

## Running Behind a Reverse Proxy

Simon can be deployed behind a reverse proxy like Nginx or Traefik. WebSocket support is required for real-time monitoring.

### Nginx Configuration

```nginx
location / {
    proxy_pass http://localhost:30000;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    
    # WebSocket support (required)
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_read_timeout 86400;
}
```

### Traefik Configuration

The compose file below provides a reverse proxy with automatic TLS support using Let's Encrypt. Set the `HOST` and `ACME_MAIL` environment variables:

```yaml
services:
  reverse-proxy:
    image: traefik:v3.2
    ports:
      - "443:443"
      - "80:80"
    volumes:
      - "./letsencrypt:/letsencrypt"
      - "/var/run/docker.sock:/var/run/docker.sock:ro"
    command:
      - --providers.docker.exposedByDefault=false
      - --entrypoints.web.address=:80
      - --entrypoints.web.http.redirections.entrypoint.to=websecure
      - --entryPoints.web.http.redirections.entrypoint.scheme=https
      - --entrypoints.websecure.address=:443
      - --entrypoints.websecure.asDefault=true 
      - --entrypoints.websecure.http.tls.certresolver=myresolver
      - --certificatesresolvers.myresolver.acme.email=${ACME_MAIL}
      - --certificatesresolvers.myresolver.acme.tlschallenge=true
      - --certificatesresolvers.myresolver.acme.storage=/letsencrypt/acme.json
  
  simon:
    image: alibahmanyar/simon
    hostname: simon
    environment:
      SIMON_PASSWORD_HASH: "$$2a$$12$$nmCGsgJ3ovx76sc/J8Bcs.Vn235KLQK7Cze83Kzm36a1v59QKVOO."
    volumes:
      - /sys:/sys:ro
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - /:/fs:ro
      - ./simon-data:/app/simon-data
    labels:
      - traefik.enable=true
      - traefik.http.routers.simon.rule=Host(`${HOST}`)
      - traefik.http.routers.simon.entrypoints=websecure
      - traefik.http.routers.simon.tls.certresolver=myresolver
      - traefik.http.services.simon.loadbalancer.server.port=30000
```

### Base Path Configuration

Simon supports deployment on a custom base path (e.g., `/monitor` instead of root `/`). The reverse proxy should strip the base path prefix before forwarding requests to Simon.

**Traefik Example:**

```yaml
services:
  simon:
    image: alibahmanyar/simon
    hostname: simon
    environment:
      SIMON_PASSWORD_HASH: "$$2a$$12$$nmCGsgJ3ovx76sc/J8Bcs.Vn235KLQK7Cze83Kzm36a1v59QKVOO."
    volumes:
      - /sys:/sys:ro
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - /:/fs:ro
      - ./simon-data:/app/simon-data
    labels:
      - traefik.enable=true
      - traefik.http.routers.simon.rule=PathPrefix(`/monitor`)
      - traefik.http.routers.simon.entrypoints=web
      - traefik.http.services.simon.loadbalancer.server.port=30000
      - traefik.http.middlewares.stripprefix.stripprefix.prefixes=/monitor
      - traefik.http.routers.simon.middlewares=stripprefix
```

**Nginx Example:**

```nginx
location /monitor/ {
    proxy_pass http://localhost:30000/;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    
    # WebSocket support
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_read_timeout 86400;
}
```

Note the trailing slashes in both the `location` and `proxy_pass` directives, which ensure proper path stripping.

## Authentication

Simon can be secured with password authentication using bcrypt hashed passwords.

### Setting Up Authentication

1. Generate a bcrypt hash of your password. You can use online tools or command-line utilities:
   ```bash
   # Using Python
   python -c "import bcrypt; print(bcrypt.hashpw(b'your_password', bcrypt.gensalt()).decode())"
   
   # Using htpasswd (Apache tools)
   htpasswd -bnBC 12 "" your_password | tr -d ':\n'
   ```

2. Set the hash using the `SIMON_PASSWORD_HASH` environment variable or `--password-hash` flag

3. **Important**: Pay attention to dollar signs (`$`) in the hash:
   - In Docker Compose files, escape each `$` as `$$`
   - In shell commands, use single quotes to prevent shell expansion

**Example - Docker Compose:**
```yaml
environment:
  SIMON_PASSWORD_HASH: "$$2a$$12$$nmCGsgJ3ovx76sc/J8Bcs.Vn235KLQK7Cze83Kzm36a1v59QKVOO."
```

**Example - Command Line:**
```bash
# Using environment variable
SIMON_PASSWORD_HASH='$2a$12$YOUR_BCRYPT_HASH' ./simon

# Using CLI flag
./simon --password-hash '$2a$12$YOUR_BCRYPT_HASH'
```

The default password for the example hash shown throughout this documentation is `secret`. **Always change this in production environments.**
