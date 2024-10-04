CREATE TABLE bots (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255),
    market VARCHAR(255) NOT NULL,
    trading_strategy VARCHAR(255) NOT NULL,
    symbols TEXT NOT NULL,
    lookback INT NOT NULL,
    threshold DOUBLE PRECISION NOT NULL,
    risk_per_trade DOUBLE PRECISION NOT NULL,
    max_positions INT NOT NULL,
    timeframes TEXT NOT NULL,
    volatility_window INT NOT NULL,
    volatility_threshold DOUBLE PRECISION NOT NULL,
    is_running BOOLEAN DEFAULT FALSE
);
-- zflub$0$1$9$8$!