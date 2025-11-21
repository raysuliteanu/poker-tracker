# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a full-stack poker bankroll tracker with a Rust backend and Svelte frontend. The application tracks poker sessions with profit/loss calculations, displays analytics, and provides time-based visualizations.

## Development Commands

### Backend (Rust + Actix-web + Diesel)

```bash
cd backend
cargo run                    # Start dev server (localhost:8080)
cargo test                   # Run tests
cargo fmt                    # Format code
cargo clippy                 # Lint
```

### Frontend (Svelte + Vite + TypeScript)

```bash
cd frontend
npm run dev                  # Start dev server (localhost:5173)
npm run build                # Production build
npm run check                # Type check TypeScript
```

### Docker (Recommended)

```bash
docker-compose up -d         # Start all services
docker-compose logs -f       # View logs
docker-compose down          # Stop services
docker-compose up -d --build # Rebuild and start
```

Services run on:
- Frontend: http://localhost:8888
- Backend API: http://localhost:8080
- PostgreSQL: localhost:5432

## Quick Architecture Reference

See `ARCHITECTURE.md` for detailed implementation documentation.

**Backend:** Rust + Actix-web + Diesel ORM + PostgreSQL
**Frontend:** Svelte + TypeScript + Vite

### Key Files

- `backend/src/main.rs` - Server entry point, route configuration
- `backend/src/handlers/` - API request handlers
- `backend/src/middleware/auth.rs` - JWT authentication middleware
- `frontend/src/App.svelte` - Main app component, routing, global styles
- `frontend/src/lib/api.ts` - API client
- `frontend/src/stores/` - Svelte stores (auth, theme)

## API Endpoints

All endpoints prefixed with `/api`

**Public:**
- `POST /auth/register` - Register new user
- `POST /auth/login` - Login, returns JWT token

**Protected (requires JWT):**
- `GET /auth/me` - Get current user
- `PUT /auth/cookie-consent` - Update cookie consent
- `POST /auth/change-password` - Change password
- `POST /sessions` - Create poker session
- `GET /sessions` - Get all user sessions
- `PUT /sessions/{id}` - Update session
- `DELETE /sessions/{id}` - Delete session

## Database Schema

**users:** id mod(UUID), email (unique), username (unique), password_hash, cookie_consent, cookie_consent_date, created_at, updated_at

**poker_sessions:** id (UUID), user_id (FK), session_date, duration_minutes, buy_in_amount, rebuy_amount, cash_out_amount, notes, created_at, updated_at

## Adding Features

### New Backend Endpoint
1. Add handler in `handlers/auth.rs` or `handlers/poker_session.rs`
2. Add route in `main.rs`
3. Wrap with `AuthMiddleware` if protected

### New Frontend Page
1. Create component in `frontend/src/pages/`
2. Add route in `App.svelte` (both `publicRoutes` and `protectedRoutes` if needed)
3. Add navigation link if applicable

### Database Migration
```bash
cd backend
diesel migration generate <name>
# Edit up.sql and down.sql
diesel migration run
```

## Important Notes

- Migrations run automatically on backend startup
- JWT tokens expire after 7 days
- Auth middleware returns JSON errors (not plain text)
- Frontend uses page reload after login/logout/register to ensure proper route re-evaluation
- Navbar and footer use `position: fixed` for consistent viewport-width layout
