# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a full-stack poker bankroll tracker with a Rust backend and Svelte frontend. The application tracks poker sessions with profit/loss calculations, displays analytics, and provides time-based visualizations.

## Development Commands

### Backend (Rust + Actix-web + Diesel)

```bash
# Navigate to backend
cd backend

# Development
cargo run                    # Start dev server (localhost:8080)
cargo test                   # Run tests
cargo fmt                    # Format code
cargo clippy                 # Lint

# Database
diesel migration run         # Apply pending migrations
diesel migration generate <name>  # Create new migration
```

### Frontend (Svelte + Vite + TypeScript)

```bash
# Navigate to frontend
cd frontend

# Development
npm run dev                  # Start dev server (localhost:5173)
npm run build                # Production build
npm run preview              # Preview production build
npm run check                # Type check TypeScript

# Install dependencies
npm install
```

### Docker

```bash
# From project root
docker-compose up -d         # Start all services
docker-compose logs -f       # View logs
docker-compose down          # Stop services
docker-compose up -d --build # Rebuild and start
```

## Architecture

### Backend Architecture

**Layered Structure:**
- `main.rs` - Application entry point, runs migrations automatically on startup, configures Actix-web server
- `handlers/` - Request handlers (auth, poker_session)
- `models/` - Data structures and validation (User, PokerSession, request/response types)
- `middleware/` - AuthMiddleware extracts user_id from JWT and adds to request extensions
- `utils/` - DB connection pooling (r2d2) and JWT creation/validation
- `schema.rs` - Diesel auto-generated schema from migrations

**Key Patterns:**
- All `/api/sessions/*` and protected `/api/auth/*` routes wrapped with AuthMiddleware
- User ID extracted from JWT in middleware, accessed via `req.extensions().get::<Uuid>()`
- All handlers return `impl Responder` with JSON responses
- Validation using `validator` crate on request models before processing

**Database:**
- PostgreSQL with Diesel ORM
- Migrations run automatically on server start via `embed_migrations!` macro
- Connection pooling with r2d2
- Two main tables: `users` and `poker_sessions` (with foreign key to users)

### Frontend Architecture

**State Management:**
- `stores/auth.ts` - Authentication state (user, token, isAuthenticated)
  - Token stored in localStorage
  - Provides login/logout/updateUser methods
- `stores/theme.ts` - Theme state (light/dark mode)
  - Persisted to localStorage
  - Updates `data-theme` attribute on document element

**Routing:**
- Hash-based routing using svelte-spa-router
- Routes defined in `App.svelte`
- Protected routes checked via `conditionsFailed` handler
- Public routes: `/login`, `/register`, `/help`, `/privacy`
- Protected routes: `/` (Dashboard), `/settings`

**API Layer:**
- `lib/api.ts` - Centralized API client
- Auto-includes JWT token from localStorage in Authorization header
- Returns `{ data, error }` wrapper for consistent error handling
- All endpoints typed with TypeScript interfaces

**Components:**
- `pages/` - Full page components (Dashboard, Login, Register, Settings, Help, Privacy)
- `components/` - Reusable components (Navigation, CookieConsent, SessionTable, SessionForm, BankrollChart)
- `Navigation.svelte` - Only renders when authenticated (checks `$authStore.isAuthenticated`)

**Theming:**
- CSS variables defined in `App.svelte` under `:global([data-theme='light'])` and `:global([data-theme='dark'])`
- Theme toggle in Navigation updates store, which updates DOM attribute
- Variables: `--color-bg`, `--color-bg-secondary`, `--color-text`, `--color-primary`, etc.

## Environment Setup

### First Time Setup

1. **PostgreSQL Database:**
   ```bash
   createdb poker_tracker
   ```

2. **Backend Environment:**
   ```bash
   cd backend
   cp .env.example .env
   # Edit .env: Update DATABASE_URL and JWT_SECRET
   ```

3. **Frontend Environment:**
   ```bash
   cd frontend
   cp .env.example .env
   # Edit .env: Set VITE_API_URL (default: http://localhost:8080/api)
   ```

4. **Install Diesel CLI (one-time):**
   ```bash
   cargo install diesel_cli --no-default-features --features postgres
   ```

## API Endpoints

All endpoints prefixed with `/api`

**Public:**
- `POST /auth/register` - Register (email, username, password)
- `POST /auth/login` - Login (email, password) → returns JWT token

**Protected (requires JWT in Authorization header):**
- `GET /auth/me` - Get current user
- `PUT /auth/cookie-consent` - Update cookie consent
- `POST /auth/change-password` - Change password
- `POST /sessions` - Create poker session
- `GET /sessions` - Get all user sessions
- `GET /sessions/{id}` - Get specific session
- `PUT /sessions/{id}` - Update session
- `DELETE /sessions/{id}` - Delete session

## Database Schema

**users:**
- id (UUID PK), email (unique), username (unique), password_hash, cookie_consent, cookie_consent_date, created_at, updated_at

**poker_sessions:**
- id (UUID PK), user_id (FK to users), session_date, duration_minutes, buy_in_amount, rebuy_amount, cash_out_amount, notes, created_at, updated_at

Profit calculation: `cash_out_amount - (buy_in_amount + rebuy_amount)`

## Adding New Features

### Adding Backend Endpoint

1. Create handler function in `handlers/auth.rs` or `handlers/poker_session.rs`
2. Add route in `main.rs` within appropriate scope
3. If protected, ensure route is wrapped with `AuthMiddleware`
4. Access user_id via `req.extensions().get::<Uuid>()`

### Adding Frontend Page

1. Create component in `frontend/src/pages/`
2. Add route to `routes` object in `App.svelte`
3. Update route protection logic in `conditionsFailed` if needed
4. Add navigation link in `Navigation.svelte` if applicable

### Database Migration

```bash
cd backend
diesel migration generate <descriptive_name>
# Edit up.sql and down.sql in new migrations/ folder
diesel migration run
# Update schema.rs is auto-generated
# Update models in src/models/ to match new schema
```

## Important Notes

- Backend automatically runs pending migrations on startup (no manual intervention needed in production)
- JWT tokens expire after 7 days (configured in `utils/jwt.rs`)
- CORS is configured to allow any origin in development (update for production)
- All monetary values use BigDecimal on backend, parsed to/from floats in frontend
- Session profit is calculated on both backend and frontend for display consistency
