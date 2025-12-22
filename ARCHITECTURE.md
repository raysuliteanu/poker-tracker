# Architecture Documentation

This document provides detailed technical documentation of the Poker Bankroll
Tracker application.

## System Overview

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    Frontend     │────▶│     Backend     │────▶│   PostgreSQL    │
│  (Svelte/Vite)  │     │  (Rust/Actix)   │     │    Database     │
│   Port: 8888    │     │   Port: 8080    │     │   Port: 5432    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Backend Architecture

### Technology Stack

- **Framework:** Axum 0.8
- **ORM:** Diesel 2.x with PostgreSQL
- **Authentication:** JWT (jsonwebtoken crate)
- **Password Hashing:** bcrypt (configurable cost)
- **Configuration:** config crate with TOML support
- **Validation:** validator crate
- **Error Handling:** thiserror
- **Testing:** testcontainers, rstest, axum-test, proptest

### Configuration System

The application uses a centralized TOML-based configuration system via the `config` crate with environment variable overrides and hardcoded defaults.

**Configuration Precedence:** defaults → TOML file → environment variables

**Configuration Structure:**

```rust
AppConfig {
    server: ServerConfig {
        host: String,      // Default: "127.0.0.1"
        port: u16,         // Default: 8080
    },
    database: DatabaseConfig {
        url: String,           // Required, no default
        max_connections: u32,  // Default: 100
        min_idle: u32,         // Default: 10
    },
    security: SecurityConfig {
        jwt_secret: String,    // Required, no default
        bcrypt_cost: u32,      // Default: 12
    },
}
```

**Configuration Sources:**

1. **TOML File** (`poker-tracker.toml`, optional):
   ```toml
   [server]
   host = "127.0.0.1"
   port = 8080

   [database]
   url = "postgres://..."
   max_connections = 100
   min_idle = 10

   [security]
   jwt_secret = "secret"
   bcrypt_cost = 12
   ```

2. **Environment Variables** (override TOML):
   - `DATABASE_URL` → `database.url`
   - `SECURITY_JWT_SECRET` → `security.jwt_secret`
   - `DATABASE_MAX_CONNECTIONS` → `database.max_connections`
   - `DATABASE_MIN_IDLE` → `database.min_idle`
   - `SECURITY_BCRYPT_COST` → `security.bcrypt_cost`
   - `SERVER_HOST` → `server.host`
   - `SERVER_PORT` → `server.port`

3. **Hardcoded Defaults** (used if not in TOML or env)

**Required Fields:**
- `DATABASE_URL`: PostgreSQL connection string
- `SECURITY_JWT_SECRET`: JWT signing secret

If required fields are missing, the application exits with a clear error message.

**Loading Process:**
1. `main.rs` calls `AppConfig::load()` on startup
2. Config is passed to `PokerTrackerApp::new(config)`
3. Config is stored in `AppState` for handler access
4. JWT secret is passed to `AuthLayer` middleware
5. Handlers access config via `state.config`

### Directory Structure

```
backend/
├── src/
│   ├── main.rs              # Binary entry point
│   ├── lib.rs               # Re-exports modules for integration tests
│   ├── app.rs               # Application setup, server config, routes
│   ├── schema.rs            # Diesel-generated database schema
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── auth.rs          # Authentication handlers
│   │   └── poker_session.rs # Session CRUD handlers
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs          # User model and request types
│   │   └── poker_session.rs # PokerSession model
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── auth.rs          # JWT authentication middleware (Axum layer)
│   └── utils/
│       ├── mod.rs           # Module exports
│       ├── config.rs        # Configuration system (AppConfig)
│       ├── db.rs            # Database pool and DbProvider trait
│       └── jwt.rs           # JWT creation/validation
├── tests/
│   ├── common/
│   │   └── mod.rs           # DirectConnectionTestDb and PooledConnectionTestDb fixtures
│   ├── http_common/
│   │   └── mod.rs           # HTTP test fixtures using axum-test
│   ├── auth_tests.rs        # Auth integration tests (19 tests)
│   ├── session_tests.rs     # Session integration tests (40 tests)
│   ├── http_auth_tests.rs   # HTTP auth integration tests (24 tests)
│   └── http_session_tests.rs # HTTP session integration tests (25 tests)
├── migrations/              # Diesel migrations
├── Cargo.toml
└── Dockerfile
```

### Request Flow

1. Request arrives at Axum server
2. TraceLayer logs request
3. CORS middleware processes headers
4. For protected routes, `AuthLayer` validates JWT and extracts user_id
5. Handler processes request, interacts with database via Diesel
6. JSON response returned

### Authentication Middleware

The `AuthLayer` and `AuthMiddleware` (`middleware/auth.rs`):

- Implemented as an Axum Tower layer
- Extracts Bearer token from Authorization header
- Validates JWT and extracts user_id from claims
- Injects user_id as an Axum `Extension` for handler access
- Returns JSON error `{"error": "Invalid or missing token"}` on failure
- Uses `thiserror` for error type definitions

```rust
// Accessing user_id in Axum handlers:
pub async fn handler(Extension(user_id): Extension<Uuid>) -> Response {
    // handler logic
}
```

### Error Handling

- Uses `thiserror` crate for custom error types with derive macros
- All errors return JSON responses with `{"error": "message"}` format
- Custom error enums defined per handler module (e.g., `CreateSessionError`)
- Unique constraint violations (duplicate email/username) return user-friendly messages
- Auth failures return 401 with JSON body
- Error types implement `Display` and `Error` traits via `#[derive(Error)]`

### Database

**Connection Pooling:** r2d2 with Diesel PostgreSQL backend

**Configuration:**
- `max_connections` (default: 100) - Maximum pool size
- `min_idle` (default: 10) - Keeps connections warm for reduced latency
- Both configurable via `DatabaseConfig` in `AppConfig`

**DbProvider Trait:** Single trait abstraction for database connections:
- Production: `DbPool` implements `DbProvider` with pooled connections
- Unit tests: `DirectConnectionTestDb` implements `DbProvider` with ephemeral single-connection pools
- HTTP tests: `PooledConnectionTestDb` implements `DbProvider` with proper connection pools (production fidelity)

This enables true integration testing of business logic without mocking while maintaining clear separation between test types.

**Migrations:** Run automatically on startup via `embed_migrations!` macro

**Tables:**

```sql
-- users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    cookie_consent BOOLEAN NOT NULL DEFAULT FALSE,
    cookie_consent_date TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- poker_sessions
CREATE TABLE poker_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    session_date DATE NOT NULL,
    duration_minutes INTEGER NOT NULL,
    buy_in_amount DECIMAL(10,2) NOT NULL,
    rebuy_amount DECIMAL(10,2) NOT NULL DEFAULT 0,
    cash_out_amount DECIMAL(10,2) NOT NULL,
    notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

### Integration Testing

**Framework:** testcontainers + rstest + axum-test + proptest

**Location:** `backend/tests/` directory

**Test Types:**

1. **Unit Tests** (`auth_tests.rs`, `session_tests.rs`): Test business logic directly
   - 59 tests covering auth and session operations
   - Use `DirectConnectionTestDb` for fast, isolated testing
   - Call `do_*` functions directly without HTTP layer

2. **HTTP Tests** (`http_auth_tests.rs`, `http_session_tests.rs`): Test full HTTP stack
   - 49 tests covering HTTP endpoints and middleware
   - Use `PooledConnectionTestDb` for production fidelity
   - Use `axum-test` for HTTP request/response testing

3. **Property-based Tests** (in `middleware/auth.rs`): Validate invariants
   - Use `proptest` for auth header parsing edge cases
   - Ensures robust error handling

**Key Components:**

1. **DirectConnectionTestDb** (`tests/common/mod.rs`):
   - Manages a temporary PostgreSQL container using testcontainers
   - Creates ephemeral single-connection pools (cost 4 for tests)
   - Implements `DbProvider` trait for handler compatibility
   - Used by unit tests for maximum clarity and isolation

2. **PooledConnectionTestDb** (`tests/common/mod.rs`):
   - Also manages a temporary PostgreSQL container
   - Maintains a proper connection pool like production
   - Implements `DbProvider` trait
   - Used by HTTP tests to match production behavior

3. **HttpTestContext** (`tests/http_common/mod.rs`):
   - Wraps `axum-test::TestServer` with `PooledConnectionTestDb`
   - Provides helper functions for common HTTP operations
   - Manages JWT tokens for authenticated requests

4. **rstest fixtures**:
   - `#[fixture]` async fn test_db() provides `DirectConnectionTestDb`
   - `#[fixture]` async fn http_ctx() provides `HttpTestContext`
   - Use with `#[rstest]` attribute on test functions

5. **Handler testing pattern**:
   - Extract business logic into `do_*` functions accepting `&dyn DbProvider`
   - Call `do_*` functions directly in unit tests
   - Call HTTP handlers through `TestServer` in HTTP tests
   - All handlers use non-generic trait objects for simplicity

**Example (Unit Test):**

```rust
#[rstest]
#[tokio::test]
async fn test_create_session(#[future] test_db: DirectConnectionTestDb) {
    let db = test_db.await;
    let user = create_test_user_raw(&db, "test@test.com", "test");

    let result = do_create_session(
        &db,
        user.id,
        default_session_request(),
    ).await;

    assert!(result.is_ok());
}
```

**Example (HTTP Test):**

```rust
#[rstest]
#[tokio::test]
async fn test_register_with_valid_data(#[future] http_ctx: HttpTestContext) {
    let ctx = http_ctx.await;
    let response = ctx.server
        .post("/api/auth/register")
        .json(&json!({"email": "test@test.com", "username": "test", "password": "pass123"}))
        .await;

    response.assert_status_created();
}
```

**Benefits:**
- Real PostgreSQL database (not mocks or SQLite)
- Isolated test environment per test
- Tests actual SQL queries and migrations
- Fast setup/teardown with containers
- Comprehensive coverage: 108 integration tests
- Production fidelity with HTTP tests using pooled connections

## Frontend Architecture

### Technology Stack

- **Framework:** Svelte 5 (Runes mode)
- **Build Tool:** Vite 7
- **Language:** TypeScript
- **Routing:** svelte-spa-router (hash-based)
- **Charts:** Chart.js

### Directory Structure

```
frontend/
├── src/
│   ├── App.svelte           # Main app, routing, global styles
│   ├── main.ts              # Entry point
│   ├── lib/
│   │   └── api.ts           # API client
│   ├── stores/
│   │   ├── auth.ts          # Authentication state
│   │   └── theme.ts         # Theme state (light/dark)
│   ├── components/
│   │   ├── Navigation.svelte    # Fixed navbar
│   │   ├── Footer.svelte        # Fixed footer
│   │   ├── SessionTable.svelte  # Sessions data table
│   │   ├── SessionForm.svelte   # Add/edit session modal
│   │   ├── BankrollChart.svelte # Profit chart
│   │   └── CookieConsent.svelte # Cookie consent banner
│   └── pages/
│       ├── Landing.svelte   # Public landing page
│       ├── Login.svelte     # Login form
│       ├── Register.svelte  # Registration form
│       ├── Dashboard.svelte # Main authenticated view
│       ├── Settings.svelte  # User settings
│       ├── Help.svelte      # Help documentation
│       └── Privacy.svelte   # Privacy policy
├── index.html
├── package.json
├── vite.config.ts
├── nginx.conf               # Production nginx config
└── Dockerfile
```

### Layout Structure

The app uses a fixed header/footer layout:

```
┌─────────────────────────────────────┐
│  Navigation (position: fixed)       │  ← Always viewport width
├─────────────────────────────────────┤
│                                     │
│  Main Content Area                  │  ← Scrollable, padded
│  (padding-top: 80px)                │
│  (padding-bottom: 60px)             │
│                                     │
├─────────────────────────────────────┤
│  Footer (position: fixed)           │  ← Always viewport width
└─────────────────────────────────────┘
```

### Routing

Two route sets based on authentication state:

```typescript
// Public routes (unauthenticated)
const publicRoutes = {
  "/": Landing,
  "/login": Login,
  "/register": Register,
  "/help": Help,
  "/privacy": Privacy,
};

// Protected routes (authenticated)
const protectedRoutes = {
  "/": Dashboard,
  "/settings": Settings,
  "/help": Help,
  "/privacy": Privacy,
};
```

Routes switch reactively based on `$authStore.isAuthenticated`.

### State Management

**Auth Store (`stores/auth.ts`):**

- Persists token to localStorage
- Provides `login()`, `logout()`, `updateUser()` methods
- Exposes `isAuthenticated`, `user`, `token` state

**Theme Store (`stores/theme.ts`):**

- Persists theme preference to localStorage
- Updates `data-theme` attribute on document element
- Supports 'light' and 'dark' themes

### API Client

The API client (`lib/api.ts`):

- Centralized fetch wrapper
- Automatically includes JWT in Authorization header
- Returns `{ data, error }` pattern for consistent error handling
- Catches network errors and returns user-friendly messages

```typescript
// Usage example
const response = await api.sessions.getAll();
if (response.error) {
  // Handle error
} else {
  // Use response.data
}
```

### Authentication Flow

**Registration:**

1. User submits form on Register page
2. API call to `/auth/register`
3. On success, store token and user in authStore
4. Show success message, then reload page to Dashboard

**Login:**

1. User submits credentials on Login page
2. API call to `/auth/login`
3. On success, store token and user in authStore
4. Reload page to trigger route re-evaluation

**Logout:**

1. User clicks Logout in Navigation
2. Clear authStore (removes token from localStorage)
3. Reload page to show Landing page

**Token Validation:**

- On app mount, if token exists, call `/auth/me` to validate
- Invalid token silently logs out user (no error shown)

### Theming

CSS custom properties defined in App.svelte:

```css
:global([data-theme="light"]) {
  --color-bg: #f5f5f5;
  --color-bg-secondary: #ffffff;
  --color-text: #1a1a1a;
  --color-primary: #3b82f6;
  /* ... */
}

:global([data-theme="dark"]) {
  --color-bg: #1a1a1a;
  --color-bg-secondary: #2d2d2d;
  --color-text: #ffffff;
  --color-primary: #3b82f6;
  /* ... */
}
```

Theme toggle in Navigation updates the store, which updates the DOM attribute.

### Components

**Navigation:**

- Fixed position at top
- Only renders when authenticated
- Contains: brand, Dashboard link, Settings link, theme toggle, Logout button

**Footer:**

- Fixed position at bottom
- Always renders
- Contains: Help link, Privacy link, copyright text

**SessionTable:**

- Displays sessions in table format
- Edit (pencil icon) and Delete (trash icon) actions
- Color-coded profit/loss values

**SessionForm:**

- Modal dialog for add/edit session
- Form validation
- Date picker, duration, monetary inputs, notes

**BankrollChart:**

- Chart.js line chart
- Shows cumulative profit over time
- Responsive sizing

## Docker Configuration

### Services

```yaml
services:
  postgres: # PostgreSQL 16 Alpine
  backend: # Rust application
  frontend: # Nginx serving built Svelte app
```

### Networking

- All services on same Docker network
- Backend connects to postgres via hostname `postgres`
- Frontend nginx proxies to backend not needed (direct browser-to-backend)

### Volumes

- `postgres_data`: Persists database data

## Performance Configuration

The backend includes several configurable parameters for optimization, all configurable via `poker-tracker.toml` or environment variables:

### Database Connection Pool

- `database.max_connections` (default: 100) - Maximum number of pooled connections
- `database.min_idle` (default: 10) - Keeps connections warm to reduce latency
- Connection timeout: 5 seconds
- Configure via `DATABASE_MAX_CONNECTIONS` and `DATABASE_MIN_IDLE` env vars or TOML

### Bcrypt Hashing Cost

- `security.bcrypt_cost` (default: 12) - Password hashing cost factor
- Production: 12 (secure, ~250ms per hash)
- Load testing: 4 (fast, ~15ms per hash, configured in docker-compose.perf.yml)
- Trade-off: Lower cost = faster authentication but less security
- Configure via `SECURITY_BCRYPT_COST` env var or TOML

### Tokio Runtime

- Uses default worker threads (number of CPU cores)
- Multi-threaded runtime for concurrent request handling

### API Pagination

- GET /sessions limited to 100 most recent sessions
- Prevents unbounded result sets as data grows
- Ensures consistent performance regardless of session count

### Performance Testing

The k6 load tests can be customized:
- `./run-perf-tests.sh` - Default 100 virtual users
- `./run-perf-tests.sh 500` - Custom VU count
- Each VU uses unique user to eliminate database contention
- Tests use BCRYPT_COST=4 automatically for realistic load simulation

## Security Considerations

- Passwords hashed with bcrypt (configurable cost via `security.bcrypt_cost`)
- JWT tokens expire after 7 days (secret configured via `security.jwt_secret`)
- CORS configured (currently allows any origin - restrict for production)
- Auth middleware validates all protected routes
- SQL injection prevented by Diesel's parameterized queries
- XSS mitigated by Svelte's automatic escaping
- Configuration system supports environment-only secrets (don't commit JWT_SECRET to TOML in production)

## Production Deployment Notes

1. Update CORS configuration in `main.rs` to restrict origins
2. Set strong `SECURITY_JWT_SECRET` environment variable (never commit to version control)
3. Use proper PostgreSQL credentials via `DATABASE_URL` environment variable
4. Consider using TOML for non-secret config, environment variables for secrets
5. Consider adding rate limiting
5. Enable HTTPS via reverse proxy or load balancer
