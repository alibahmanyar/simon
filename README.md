# Simon

[![License: MIT](https://img.shields.io/badge/License-apache2.0-yellow.svg)](https://opensource.org/license/apache-2-0)
[![Docker](https://img.shields.io/docker/pulls/alibahmanyar/simon.svg)](https://hub.docker.com/r/alibahmanyar/simon)

> A lightweight, **web-based system monitor** with alerts and Docker insights—all bundled into a **single binary**.

---


#### Dashboard Overview

![Dashboard Screenshot](docs/media/shot0.png)  
<details>
<summary>Screenshots</summary>

</details>

#### Alert Configuration
![Alerts Screenshot](path/to/alerts.png)  
*Set thresholds and view alerts to keep your system in check.*

---

## ✨ Features

- **💻 System Monitoring**: Track CPU, memory, disk usage, disk I/O and network activity in real-time
- **🌐 Web-Based UI**: A responsive interface accessible from any browser
- **🐳 Docker Integration**: List containers, monitor resource usage, and view container logs
- **🚨 Alerting System**: Configure thresholds and get notified when metrics cross set limits
- **📦 Zero Dependencies**: Single binary deployment with no external requirements
- **⚡ Low Overhead**: Minimal resource footprint

---

## 🔧 Installation

### Using Prebuilt Binaries

The simplest way to install Simon is using prebuilt binaries:

Download the latest release for your platform from the [Releases](https://github.com/alibahmanyar/simon/releases) page.

```bash
chmod +x simon
./simon
```
Just run the binary and Simon will start monitoring!

### Using Docker

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

### Using Docker Compose

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
      # Note: Dollar signs are escaped with additional dollar signs for Docker Compose
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
**Notes:**

1. For accesing docker stats, the user running Simon needs access to the Docker socket (`/var/run/docker.sock`). This can be achieved by:
   - Using a user that belongs to the `docker` group
   - Running as root

2. For accurate system information, mount relevant filesystem paths:
   - Mount `/etc/lsb-release` or similar OS identification files for correct OS detection
   - Mount `/sys` for hardware, network and process information
   - Mount filesystems you want to monitor (e.g., `/` as `/rootfs`) for disk usage statistics

3. The password hash should be provided correctly; pay attention to escaping special characters (like dollar sign) in the hash



### Running Behind a Reverse Proxy

Simon can be deployed behind a reverse proxy like Nginx or Traefik:

#### Nginx Configuration Example
```nginx
...
    
    location / {
        proxy_pass http://localhost:30000;
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
...
```

#### Traefik Configuration Example

For Docker Compose with Traefik the below compose file can be used; it will provide reverse proxy ith TLS support, you only need to provide the `HOST` and `ACME_MAIL` environment variables:

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
    hostname: simon # Set container hostname (replace with your own)
    environment:
      # Authentication configuration
      # Bcrypt hash for 'secret', replace with your own hash or remove to disable authentication
      # Note: Dollar signs are escaped with additional dollar signs for Docker Compose
      SIMON_PASSWORD_HASH: "$$2a$$12$$nmCGsgJ3ovx76sc/J8Bcs.Vn235KLQK7Cze83Kzm36a1v59QKVOO."
    volumes:
      - /sys:/sys:ro
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - /:/fs:ro
      - ./simon-data:/app/simon-data
    labels:
      - traefik.enable=true
      - traefik.http.routers.dz.rule=Host(`${HOST}`)
      - traefik.http.routers.dz.entrypoints=websecure
      - traefik.http.routers.dz.tls.certresolver=myresolver
      - traefik.http.services.dz.loadbalancer.server.port=30000
```

### Authentication

Simon can be secured with password authentication:

1. Generate a bcrypt hash of your password (many online tools available)
2. Set the hash using the `SIMON_PASSWORD_HASH` environment variable or `--password-hash` flag
3. Pay attnetion to the dollar signs and escape them if needed

```bash
# Using env var
SIMON_PASSWORD_HASH='$2a$12$YOUR_BCRYPT_HASH' simon

# Or using CLI flag
simon --password-hash '$2a$12$YOUR_BCRYPT_HASH'
```

## ⚙️ Configuration

Simon is configured through environment variables or command-line arguments:

| Option | Environment Variable | CLI Flag | Default | Description |
|--------|---------------------|----------|---------|-------------|
| Address | `SIMON_ADDRESS` | `-a`, `--address` | `0.0.0.0` | Address to bind the server to |
| Port | `SIMON_PORT` | `-p`, `--port` | `30000` | Port to bind the server to |
| Update interval | `SIMON_UPDATE_INTERVAL` | `-T`, `--update-interval` | `2` | Metrics update interval in seconds (1-30) |
| Password hash | `SIMON_PASSWORD_HASH` | `-H`, `--password-hash` | None | Bcrypt password hash for authentication |
| Database path | `SIMON_DB_PATH` | `--db-path` | `./simon-data/simon.db` | Path to SQLite database |



## 🚨 Notifications and Alerts

Simon provides an alert system to notify you about critical system events.

### Setting Up Notifications
1. **Navigate to settings by clicking on the gear icon**
2. **Add Notification Method**
   - Name your notification method
   - Enter webhook URL for receiving alerts
   - Use `{notif_msg}` placeholder in url or request body to insert alert message

2. **Configure Alert Conditions**
   - Set time window (in minutes)
   - Select resource category, name, and property to monitor
   - Define condition and threshold value
   - Toggle "Active" to enable/disable

Alerts are triggered when conditions are met for the specified duration, sending formatted messages to your webhook endpoint. The system works with any service that accepts webhooks, including Discord, Telegram, Slack, etc.


## Building from Source

Simon consists of a Rust backend and a web frontend. Here's how to build it from source:

```bash
# Rust toolchain and bun should be installed

# Clone the repository
git clone https://github.com/alibahmanyar/simon.git
cd simon

# Build the web frontend first and then compile the Rust application
make web-setup
make web
make release
```


## License

This project is licensed under the Apache-2.0 License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This project utilizes several amazing open-source libraries and tools:

### Rust Dependencies
- [axum](https://github.com/tokio-rs/axum)
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo)
- [bollard](https://github.com/fussybeaver/bollard)
- [tokio](https://tokio.rs/)
- [clap](https://clap.rs/)
- [serde](https://serde.rs/)

### Web Frontend
- [Svelte](https://svelte.dev/)
- [Chart.js](https://www.chartjs.org/)
- [Bun](https://bun.sh/)

Thank you to all the contributors and maintainers of these projects!

---

\
Happy monitoring!