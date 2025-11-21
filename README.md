# Poker Bankroll Tracker

A full-stack web application for tracking poker bankroll and session history
with detailed analytics and visualizations.

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
- **Framework**: Actix-web
- **Database**: PostgreSQL
- **ORM**: Diesel
- **Authentication**: JWT (jsonwebtoken), bcrypt
- **Migrations**: diesel_migrations

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

# Copy environment file
cp .env.example .env

# Edit .env and update DATABASE_URL and JWT_SECRET

# Install diesel CLI (if not already installed)
cargo install diesel_cli --no-default-features --features postgres

# Run migrations
diesel migration run

# Start the backend server
cargo run
```

The backend will start on `http://localhost:8080`.

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
- PostgreSQL: localhost:5432

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

## Environment Variables

### Backend (.env)

```
DATABASE_URL=postgres://postgres:password@localhost/poker_tracker
JWT_SECRET=your-secret-key-change-this-in-production
RUST_LOG=info
HOST=127.0.0.1
PORT=8080
```

### Frontend (.env)

```
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

- Password hashing with bcrypt
- JWT token authentication
- CORS configuration
- SQL injection prevention via Diesel ORM
- Input validation
- Secure session management

## Production Deployment

### Considerations

1. **Environment Variables**: Update all secret keys and credentials
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
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Support

For issues and questions, please open an issue on the GitHub repository.
