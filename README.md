# auth-service

Rust microservice for authentication and account management.

## Features

- Axum HTTP API on port `8083`
- Neo4j persistence via `neo4rs`
- Password hashing with `argon2`
- JWT access token issuing with `jsonwebtoken` (1 hour expiry)
- Google OAuth ID token validation via `https://oauth2.googleapis.com/tokeninfo?id_token={token}`
- Structured logging via `tracing` / `tracing-subscriber`

## Environment variables

Copy `.env.example` to `.env` and adjust:

- `SERVER_PORT` (default: `8083`)
- `NEO4J_URI` (example: `bolt://neo4j:7687`)
- `NEO4J_USER`
- `NEO4J_PASSWORD`
- `JWT_SECRET`
- `GOOGLE_TOKENINFO_ENDPOINT` (example: `https://oauth2.googleapis.com/tokeninfo`)

## Run locally

```bash
cp .env.example .env
cargo run
```

## Run with Docker

```bash
cp .env.example .env
docker-compose up --build
```

## API

Base path: `/api/v1`

### POST /api/v1/createPassword

Header:

- `Authorization: Bearer <jwtAccessToken>`

Body:

```json
{ "newPassword": "SecurePassword123" }
```

### POST /api/v1/login

Body:

```json
{ "username": "parent@email.com", "password": "SecurePassword123" }
```

### POST /api/v1/googleLogin

Body:

```json
{ "googleToken": "GOOGLE_OAUTH_TOKEN" }
```
