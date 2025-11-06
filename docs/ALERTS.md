# Notifications and Alerts

Simon provides a comprehensive alerting system to notify you when system metrics exceed defined thresholds. This guide explains how to configure notification methods and create alerts.

## Overview

The alerting system consists of two main components:

1. **Notification Methods**: Define how and where alerts are sent (webhooks, messaging platforms)
2. **Alert Rules**: Define what conditions trigger notifications

## Setting Up Notifications

### Step 1: Access Settings

Navigate to settings by clicking the gear icon in the top navigation bar.

### Step 2: Add Notification Method

1. Click "Add Notification Method"
2. Configure the following:
   - **Name**: A descriptive name for this notification method
   - **Webhook URL**: The endpoint that will receive alert messages
   - **Request Method**: Usually POST (or GET for simple webhooks)
   - **Request Body** (optional): Custom payload template
   - **Headers** (optional): Custom HTTP headers

#### Using Templates

Simon supports notification templates for popular platforms. These templates provide pre-configured settings for common services.

**Available Templates:**
- **Telegram** - Send notifications to a Telegram chat
- **ntfy** - Simple HTTP-based pub-sub notifications
- **Gotify** - Self-hosted push notification service
- **Pushover** - Simple push notifications
- **Pushbullet** - Send notifications to all your devices
- **Matrix** - Decentralized communication protocol
- **Custom Webhook** - Configure a custom webhook with full control

Use the `{notif_msg}` placeholder in your URL or request body to insert the alert message.

### Step 3: Configure Alert Rules

1. Click "Add Alert" in the alerts section
2. Configure the alert:
   - **Name**: Descriptive name for the alert
   - **Time Window**: How long the condition must be true before triggering (in minutes)
   - **Resource Category**: Type of resource to monitor (CPU, Memory, Disk, Network, Docker)
   - **Resource Name**: Specific resource identifier
   - **Property**: Metric to monitor (usage percentage, bytes, etc.)
   - **Condition**: Comparison operator (greater than, less than, equal to)
   - **Threshold**: Value that triggers the alert
   - **Notification Method**: Select from configured notification methods
   - **Active**: Toggle to enable/disable the alert

## Notification Method Examples

### Telegram

Send notifications directly to a Telegram chat or channel.

**Required Information:**
- Bot Token: Obtain from [@BotFather](https://t.me/botfather)
- Chat ID: Your chat or channel ID

The template automatically configures the webhook URL for you.

### ntfy

Simple, HTTP-based pub-sub notification service (self-hosted or ntfy.sh).

**Required Information:**
- Server URL: e.g., `https://ntfy.sh` or your self-hosted instance
- Topic: Your notification topic name

### Gotify

Self-hosted push notification server.

**Required Information:**
- Server URL: Your Gotify server address (e.g., `https://gotify.example.com`)
- App Token: Generate from your Gotify dashboard

### Pushover

Simple push notifications to iOS, Android, and Desktop.

**Required Information:**
- User Key: Your Pushover user key
- App Token: Create an application in Pushover to get a token

### Pushbullet

Send notifications to all your devices (iOS, Android, Desktop, Browser).

**Required Information:**
- Access Token: Generate from your Pushbullet account settings

### Matrix

Decentralized, open-source communication protocol.

**Required Information:**
- Homeserver: Your Matrix homeserver URL (e.g., `https://matrix.org`)
- Room ID: The room where notifications will be sent (e.g., `!abc123:matrix.org`)
- Access Token: Your Matrix access token

### Custom Webhook

For custom integrations or services not listed above, you can configure a custom webhook with full control over:
- HTTP method (GET, POST, PUT, etc.)
- Request headers
- Request body
- URL parameters

The `{notif_msg}` placeholder will be replaced with the actual alert message.