# Trading Bot Strategy Manager

A robust Rust-based trading bot framework that manages and executes multiple trading strategies across different cryptocurrency exchanges.

## Features

- 🤖 Automated trading strategy execution
- 📊 Currently supports Alpaca API (more exchanges coming soon)
- ⚡ Built-in rate limiting and request throttling
- 📈 Strategy management system
- 🔍 OpenTelemetry integration for observability
- 🚀 High-performance Rust implementation
- 🛡️ Error handling and recovery mechanisms

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo
- Alpaca API credentials

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/trading-bot
cd trading-bot
```

2. Install dependencies:
```bash
cargo build
```

3. Configure your environment variables:
```bash
cp .env.example .env
# Edit .env with your Alpaca API credentials
```

### Usage

1. Start the trading bot:
```bash
cargo run
```

2. Configure your trading strategies in `config/strategies.yaml`:
```yaml
strategies:
  - name: moving_average_crossover
    params:
      short_period: 10
      long_period: 20
  # Add more strategies as needed
```

## Configuration

### Rate Limiting

The bot includes a configurable rate limiter to ensure compliance with exchange API limits:

```rust
REQUESTS_PER_MINUTE=300
BURST_SIZE=10
```

### OpenTelemetry Integration

Metrics and traces are exported using OpenTelemetry. Configure your collector in `config/otel.yaml`:

```yaml
exporters:
  otlp:
    endpoint: "http://localhost:4317"
```

## Architecture

```
src/
├── api/
│   ├── alpaca.rs    # Alpaca API integration
│   └── mod.rs       # API trait definitions
├── strategy/
│   ├── manager.rs   # Strategy management
│   └── mod.rs       # Strategy trait definitions
├── ratelimit/       # Rate limiting implementation
├── telemetry/       # OpenTelemetry integration
└── main.rs
```

## Adding New Exchanges

To add support for a new exchange:

1. Implement the `ExchangeAPI` trait for your exchange
2. Add rate limiting configuration
3. Update the strategy manager to support the new exchange
4. Add appropriate tests

Example:
```rust
impl ExchangeAPI for NewExchange {
    fn place_order(&self, order: Order) -> Result<OrderResponse, Error> {
        // Implementation
    }
}
```

## Roadmap

- [ ] Add support for Binance API
- [ ] Implement WebSocket connections for real-time data
- [ ] Add backtesting capabilities
- [ ] Implement more sophisticated trading strategies
- [ ] Add portfolio management features
- [ ] Implement risk management system

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2024 [Your Name]

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

[...]
```

## Acknowledgments

- Alpaca API for providing the trading infrastructure
- The Rust community for excellent documentation and support
- OpenTelemetry for observability tools

## Support

If you have any questions or run into issues, please open an issue on the GitHub repository.