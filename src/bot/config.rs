#[derive(Deserialize, Serialize)]
pub struct TradingConfig {
    pub trading_strategy: TradingConfig,
    pub symbols: Vec<String>,
    pub lookback: usize,
    pub threshold: f64,
    pub risk_per_trade: f64,
    pub max_position: usize,
}
