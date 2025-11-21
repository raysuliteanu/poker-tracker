CREATE TABLE poker_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_date DATE NOT NULL,
    duration_minutes INTEGER NOT NULL,
    buy_in_amount DECIMAL(10, 2) NOT NULL,
    rebuy_amount DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    cash_out_amount DECIMAL(10, 2) NOT NULL,
    notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_poker_sessions_user_id ON poker_sessions(user_id);
CREATE INDEX idx_poker_sessions_session_date ON poker_sessions(session_date);
CREATE INDEX idx_poker_sessions_user_date ON poker_sessions(user_id, session_date DESC);
