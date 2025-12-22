# Poker Bankroll Tracker

A full-stack web application for tracking poker bankroll and session history
with detailed analytics and visualizations.

## Background

This is at the moment a "toy" project to see if I can build a full app with
LLM (specifically Claude Code, though a little Gemini thrown in). The idea of
the app itself though was based on a need; I did a quick search for webapps that
do poker session tracking, where I can enter some key things about a given poker
session - what I started with, what additional I might have bought in for, what
I left with, that sort of thing. But I didn't see anything. What I did find were
desktop apps for Windows and MacOS. But I'm using Linux.

So I decided to build this app. Once I get the basic stuff going, I might make it
more public and see if others find it useful. Actually hosting it on, say, AWS,
will cost a little bit, so I set up sponsor links since otherwise at least for
now I have no intention of making this a pay site. Depending on where it goes,
if anywhere, I might do ads, or might charge.

If you do think this might be useful to you, please leave a comment via the wiki.
If you have suggestions, submit an issue. If you want to contribute, feel free
to fork and make a PR.

> [!INFO]
> Just remember, right now this is a side project mostly for learning; I'm a
> backend guy so the Rust side is my thing; the frontend is 100% GenAI right now
> as I learn it.

## Features

### User Management

- User registration and authentication
- Secure password management (bcrypt hashing)
- JWT-based authentication
- Password change functionality
- Cookie consent management (GDPR/CCPA compliant)

### Session Tracking

- Record poker sessions with:
  - Date
  - Duration
  - Buy-in amount
  - Rebuy amount
  - Cash-out amount
  - Notes
- Edit and delete sessions
- View all sessions in a sortable table

### Analytics & Visualization

- Total profit/loss calculation
- Session statistics (total sessions, hours played, hourly rate)
- Interactive bankroll chart
- Time-based filtering (week, month, quarter, year, all-time)

### User Experience

- Light and dark mode themes
- Responsive design for mobile and desktop
- Cookie consent banner
- Help and privacy pages

## Tech Stack

### Backend

- **Language**: Rust
- **Framework**: Axum
- **Database**: PostgreSQL (with configurable connection pooling)
- **ORM**: Diesel
- **Authentication**: JWT (jsonwebtoken), bcrypt (configurable cost)
- **Error Handling**: thiserror
- **Migrations**: diesel_migrations
- **Testing**: testcontainers, rstest, axum-test, proptest

### Frontend

- **Framework**: Svelte 5
- **Build Tool**: Vite
- **Language**: TypeScript
- **Router**: svelte-spa-router
- **Charts**: Chart.js

## Prerequisites

- Rust (1.75 or later)
- Node.js (20 or later)
- PostgreSQL (16 or later)
- Docker and Docker Compose (optional, for containerized deployment)

## Local Development Setup

### 1. Database Setup

Create a PostgreSQL database:

```bash
createdb poker_tracker
```

### 2. Backend Setup

```bash
cd backend

# Option 1: Use TOML configuration (recommended)
cp poker-tracker.toml.example poker-tracker.toml
# Edit poker-tracker.toml and update database.url and security.jwt_secret

# Option 2: Use environment variables
cp .env.example .env
# Edit .env and update DATABASE_URL and SECURITY_JWT_SECRET

# Install diesel CLI (if not already installed)
cargo install diesel_cli --no-default-features --features postgres

# Run migrations
diesel migration run

# Start the backend server
cargo run
```

The backend will start on `http://localhost:8080`.

**Configuration Note:** The backend supports TOML files, environment variables, and hardcoded defaults. See the [Configuration](#configuration) section for details.

### 3. Frontend Setup

```bash
cd frontend

# Install dependencies
npm install

# Copy environment file
cp .env.example .env

# Start the development server
npm run dev
```

The frontend will start on `http://localhost:5173`.

## Docker Deployment

The easiest way to run the entire application:

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

Services will be available at:

- Frontend: <http://localhost:8888>
- Backend API: <http://localhost:8080>
- PostgreSQL: localhost:5433 (non-default port to avoid conflicts with local PostgreSQL)

## API Endpoints

### Authentication

- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login user
- `GET /api/auth/me` - Get current user (requires auth)
- `PUT /api/auth/cookie-consent` - Update cookie consent (requires auth)
- `POST /api/auth/change-password` - Change password (requires auth)

### Poker Sessions

- `POST /api/sessions` - Create new session (requires auth)
- `GET /api/sessions` - Get all user sessions (requires auth)
- `GET /api/sessions/{id}` - Get specific session (requires auth)
- `PUT /api/sessions/{id}` - Update session (requires auth)
- `DELETE /api/sessions/{id}` - Delete session (requires auth)

## Configuration

The backend supports multiple configuration methods with the following precedence:
**defaults → TOML file → environment variables**

### TOML Configuration (Recommended)

Create `backend/poker-tracker.toml` (see `poker-tracker.toml.example`):

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "postgres://postgres:password@localhost/poker_tracker"
max_connections = 100
min_idle = 10

[security]
jwt_secret = "your-secret-key-change-this-in-production"
bcrypt_cost = 12  # 4-6 for tests, 12+ for production
```

### Environment Variables

Environment variables override TOML values:

```sh
# Required (no defaults)
DATABASE_URL=postgres://postgres:password@localhost/poker_tracker
SECURITY_JWT_SECRET=your-secret-key-change-this-in-production

# Optional (have defaults)
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
DATABASE_MAX_CONNECTIONS=100
DATABASE_MIN_IDLE=10
SECURITY_BCRYPT_COST=12

# Logging
RUST_LOG=info
```

**Production Recommendation:** Use TOML for non-sensitive configuration, environment variables for secrets (DATABASE_URL, SECURITY_JWT_SECRET).

### Frontend (.env)

```sh
VITE_API_URL=http://localhost:8080/api
```

## Database Schema

### Users Table

- `id` (UUID, primary key)
- `email` (VARCHAR, unique)
- `username` (VARCHAR, unique)
- `password_hash` (VARCHAR)
- `cookie_consent` (BOOLEAN)
- `cookie_consent_date` (TIMESTAMP, nullable)
- `created_at` (TIMESTAMP)
- `updated_at` (TIMESTAMP)

### Poker Sessions Table

- `id` (UUID, primary key)
- `user_id` (UUID, foreign key to users)
- `session_date` (DATE)
- `duration_minutes` (INTEGER)
- `buy_in_amount` (DECIMAL)
- `rebuy_amount` (DECIMAL)
- `cash_out_amount` (DECIMAL)
- `notes` (TEXT, nullable)
- `created_at` (TIMESTAMP)
- `updated_at` (TIMESTAMP)

## Security Features

- Password hashing with bcrypt (configurable cost via `security.bcrypt_cost`, default: 12)
- JWT token authentication (7-day expiration, secret via `security.jwt_secret`)
- Centralized configuration with TOML + environment variable support
- CORS configuration
- SQL injection prevention via Diesel ORM
- Input validation with validator crate
- Secure session management
- Database connection pooling with configurable limits

## Production Deployment

### Considerations

1. **Configuration**: Use TOML for non-secret config, environment variables for secrets
   - Never commit `SECURITY_JWT_SECRET` or `DATABASE_URL` with real credentials to version control
   - Use environment variables for `DATABASE_URL` and `SECURITY_JWT_SECRET` in production
   - Consider using `poker-tracker.toml` for server host/port and performance tuning
2. **HTTPS**: Use a reverse proxy (nginx/traefik) with SSL certificates
3. **Database**: Use managed PostgreSQL or secure your database server
4. **CORS**: Configure allowed origins in production
5. **Logging**: Set appropriate log levels
6. **Backups**: Implement regular database backups

### Docker Production Deployment

```bash
# Update environment variables in docker-compose.yml
# Build with production settings
docker-compose -f docker-compose.yml up -d --build
```

## Development Commands

### Backend

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Create new migration
diesel migration generate <migration_name>
```

### Frontend

```bash
# Run development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Type check
npm run check

# Run unit tests
npm test
```

## Testing

### Backend Tests

The backend includes comprehensive testing at multiple levels:

- **Unit tests**: Located within individual source files (93 tests)
- **Integration tests**: Located in `backend/tests/` directory (108 tests)
  - `auth_tests.rs` - 19 auth business logic tests
  - `session_tests.rs` - 40 session business logic tests
  - `http_auth_tests.rs` - 24 HTTP endpoint tests
  - `http_session_tests.rs` - 25 HTTP endpoint tests
- **Property-based tests**: Using proptest for auth middleware edge cases

Integration tests use:
- **testcontainers**: Automatically spins up PostgreSQL containers for isolated testing
- **rstest**: Provides fixtures for test setup (`test_db`, `http_ctx` fixtures)
- **axum-test**: Provides HTTP testing framework for end-to-end endpoint tests
- **proptest**: Validates invariants with generated test cases

The `DbProvider` trait allows handlers to work with both pooled connections
(production and HTTP tests) and direct connections (unit tests), enabling true
integration testing of business logic with production fidelity.

```bash
cd backend
cargo test                        # Run all tests (201 total)
cargo test --lib                  # Run only unit tests (93 tests)
cargo test --test auth_tests      # Run auth integration tests
cargo test --test http_auth_tests # Run HTTP auth tests
```

### Frontend Tests

The frontend uses Vitest with Testing Library for unit tests. Tests cover:

- **Stores**: Auth store, theme store state management
- **Components**: SessionTable, SessionForm, BankrollChart, Navigation
- **Pages**: Dashboard functionality and user interactions

```bash
cd frontend
npm test              # Run tests in watch mode
npm test -- --run     # Run tests once
```

Test files are located in `frontend/src/tests/` and follow the pattern `*.test.ts`.

### Performance Tests

The project includes k6 load tests for the backend API. Performance tests use a
separate database and optimized bcrypt configuration to avoid interfering with
development data.

**Prerequisites:**

- [k6](https://k6.io/docs/get-started/installation/) installed locally
- Docker and Docker Compose

**Running Performance Tests:**

```bash
# Quick way: Use the helper script
./run-perf-tests.sh          # Default 100 virtual users
./run-perf-tests.sh 500      # Run with 500 virtual users

# Manual way: Run docker-compose commands directly
docker compose -f docker-compose.yml -f docker-compose.perf.yml up -d --build
# Wait for services to be healthy
docker compose -f docker-compose.yml -f docker-compose.perf.yml ps
# Run k6 performance tests
k6 run -e K6_VUS=100 performance-test.js
# Tear down when finished
docker compose -f docker-compose.yml -f docker-compose.perf.yml down
```

The helper script (`run-perf-tests.sh`) automates the entire workflow: starting
the performance environment, waiting for services to be healthy, running k6 tests,
and tearing down afterward.

**Configuration:**

- Each virtual user (VU) gets a unique test account to eliminate database contention
- Backend uses `BCRYPT_COST=4` for faster authentication (configured in docker-compose.perf.yml)
- Connection pool sized for concurrent load
- Pagination limits ensure consistent performance

The performance test (`performance-test.js`) simulates user workflows including:

- User registration and login
- Creating poker sessions
- Retrieving session data

Default thresholds:

- HTTP error rate < 1%
- 95th percentile response time < 500ms
- Login p95 latency < 500ms (achievable with BCRYPT_COST=4)
- Session operations p95 latency < 500ms

### E2E Tests

The project includes Playwright end-to-end tests for the full stack application.

**Prerequisites:**

- Docker and Docker Compose

**Running E2E Tests:**

```bash
# Quick way: Use the helper script
./run-e2e-tests.sh

# Manual way: Run docker-compose commands directly
docker compose -f docker-compose.e2e.yml up --build --abort-on-container-exit --exit-code-from playwright
docker compose -f docker-compose.e2e.yml down
```

The helper script (`run-e2e-tests.sh`) automates the entire workflow and reports where test artifacts are saved.

Test reports and artifacts are saved to:
- `frontend/playwright-report/` - HTML test reports
- `frontend/test-results/` - Test artifacts, screenshots, and traces

## CI/CD

GitHub Actions workflows automatically run on push and pull requests to the `main`
branch:

### Backend CI (`.github/workflows/rust-ci.yml`)

- Code formatting check (`cargo fmt`)
- Security audit (`cargo audit`)
- Unit tests (`cargo test`)

### Frontend CI (`.github/workflows/frontend-ci.yml`)

- TypeScript type checking (`npm run check`)
- Unit tests (`npm test`)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## License

Apache License - see LICENSE file for details

## Support

For issues and questions, please open an issue on the GitHub repository.
