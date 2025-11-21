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

- **Framework:** Actix-web 4.x
- **ORM:** Diesel 2.x with PostgreSQL
- **Authentication:** JWT (jsonwebtoken crate)
- **Password Hashing:** bcrypt
- **Validation:** validator crate

### Directory Structure

```
backend/
├── src/
│   ├── main.rs              # Entry point, server config, routes
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
│   │   └── auth.rs          # JWT authentication middleware
│   └── utils/
│       ├── mod.rs
│       ├── db.rs            # Database connection pool
│       └── jwt.rs           # JWT creation/validation
├── migrations/              # Diesel migrations
├── Cargo.toml
└── Dockerfile
```

### Request Flow

1. Request arrives at Actix-web server
2. CORS middleware processes headers
3. For protected routes, `AuthMiddleware` validates JWT
4. Handler processes request, interacts with database via Diesel
5. JSON response returned

### Authentication Middleware

The `AuthMiddleware` (`middleware/auth.rs`):

- Extracts Bearer token from Authorization header
- Validates JWT and extracts user_id from claims
- Stores user_id in request extensions for handler access
- Returns JSON error `{"error": "Invalid or missing token"}` on failure

```rust
// Accessing user_id in handlers:
let user_id = req.extensions().get::<Uuid>().unwrap();
```

### Error Handling

- All errors return JSON responses with `{"error": "message"}` format
- Unique constraint violations (duplicate email/username) return user-friendly messages
- Auth failures return 401 with JSON body

### Database

**Connection Pooling:** r2d2 with Diesel PostgreSQL backend

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

## Security Considerations

- Passwords hashed with bcrypt
- JWT tokens expire after 7 days
- CORS configured (currently allows any origin - restrict for production)
- Auth middleware validates all protected routes
- SQL injection prevented by Diesel's parameterized queries
- XSS mitigated by Svelte's automatic escaping

## Production Deployment Notes

1. Update CORS configuration in `main.rs` to restrict origins
2. Set strong `JWT_SECRET` environment variable
3. Use proper PostgreSQL credentials
4. Consider adding rate limiting
5. Enable HTTPS via reverse proxy or load balancer
