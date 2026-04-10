# Async SMTP Server (Rust)

A fully asynchronous SMTP server built in Rust implementing core RFC-compliant email protocol commands with TLS, Authentication, and Database-backed mail storage.

This project focuses on building a production-style SMTP server from scratch using the Rust async ecosystem.

## Features

### Core SMTP Commands
- `EHLO` / `HELO`
- `MAIL FROM`
- `RCPT TO`
- `DATA`
- `RSET`
- `NOOP`
- `VRFY`
- `HELP`
- `QUIT`

### Security Features
- STARTTLS support
- AUTH LOGIN authentication
- TLS handshake using `rustls`
- Authenticated mail submission

### Async Architecture
- Tokio async TCP server
- Concurrent multi-client handling
- Non-blocking database writes using `spawn_blocking`

### Database (Diesel + SQLite)
- Email storage
- Multi-recipient support
- User authentication
- Mailbox listing

## Tech Stack
- Rust
- Tokio
- Diesel ORM
- SQLite
- rustls
- r2d2 connection pooling
- Axum (API layer — WIP)

## Architecture
```text
Client
  │
  ▼
TCP Server (Tokio)
  │
  ▼
Session State Machine
  │
  ├── TLS Layer
  ├── Auth Layer
  └── SMTP Commands
  │
  ▼
Storage Layer (Diesel)
  │
  ▼
SQLite Database
```

## Project Structure
```text
src/
 ├── main.rs
 ├── session.rs
 ├── parser.rs
 ├── storage.rs
 ├── models.rs
 ├── schema.rs
 ├── tls.rs
 ├── config.rs
 ├── response.rs
 └── api/ (WIP)
```

## Getting Started

### 1. Clone
```bash
git clone https://github.com/yourrepo/smtp-server
cd smtp-server
```

### 2. Setup Environment
Create `.env` file:
```env
DATABASE_URL=emails.db
```

### 3. Generate TLS Certificates
```bash
mkdir certs

openssl req -x509 -newkey rsa:4096 \
-keyout certs/key.pem \
-out certs/cert.pem \
-days 365 -nodes
```

### 4. Run Server
```bash
cargo run
```

Server runs on:
- **SMTP Server:** `127.0.0.1:2525`
- **API Server (WIP):** `127.0.0.1:3000`

## Example SMTP Flow
```text
EHLO localhost
STARTTLS
AUTH LOGIN
MAIL FROM:<user@test.com>
RCPT TO:<receiver@test.com>
DATA
Hello world
.
QUIT
```

## Authentication
Users stored in database:
- `users` table

Authentication flow:
- `AUTH LOGIN`
- Username
- Password

## Database Schema
**Tables:**
- `users`
- `emails`
- `recipients`

**Supports:**
- Multi-recipient emails
- Mailbox listing
- Authentication

## Current Status

### Completed:
- [x] SMTP Protocol
- [x] Async TCP Server
- [x] TLS Support
- [x] Authentication
- [x] Database storage
- [x] Multi-recipient emails

### In Progress:
- [ ] REST API
- [ ] UI Dashboard
- [ ] Attachments support

### Planned:
- Message Queue
- DKIM/SPF verification
- SMTP relay support
- Production hardening

## Goals
This project aims to:
- Learn the SMTP protocol deeply
- Build a production-grade Rust networking system
- Understand async Rust architecture
- Build mail infrastructure from scratch

## Why This Project
Most developers use email services.
This project builds one from scratch.

**Focus Areas:**
- Async Rust
- Protocol design
- Networking
- Database design
- Security (TLS + Auth)

