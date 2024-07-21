use crate::bot::{BotStrategy, MarketType};
use crate::error::Error;
use crate::{
    bot::{BotConfig, BotInfo},
    error::RequestError,
};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

pub async fn create_bot(db: &PgPool, data: BotConfig) -> Result<String, Error> {
    let bot_id = sqlx::query_scalar!(
        r#"
            INSERT INTO bots (
                id,
                name,
                market,
                trading_strategy,
                symbols,
                lookback,
                threshold,
                risk_per_trade,
                max_positions,
                timeframes,
                volatility_window,
                volatility_threshold,
                is_running
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id
        "#,
        data.id,
        data.name,
        data.market.to_string(),
        data.trading_strategy.to_string(),
        &data.symbols.join(","),
        data.lookback as i32,
        data.threshold,
        data.risk_per_trade,
        data.max_positions as i32,
        &data.timeframes.join(","),
        data.volatility_window as i32,
        data.volatility_threshold,
        true
    )
    .fetch_one(db)
    .await?;

    Ok(bot_id)
}

pub async fn get_all_running_bot(db: &PgPool) -> Result<Vec<BotInfo>, Error> {
    let bots_record = sqlx::query!(
        r#"
        SELECT
            id,
            name, market,
            trading_strategy,
            symbols,
            lookback,
            threshold,
            risk_per_trade,
            max_positions,
            timeframes,
            volatility_window,
            volatility_threshold,
            is_running
        FROM bots
        "#
    )
    .fetch_all(db)
    .await
    .map_err(Error::Database)?;

    let bots = bots_record
        .into_iter()
        .map(|r| {
            let config = BotConfig {
                id: r.id,
                name: r.name,
                market: MarketType::from_str(&r.market).unwrap(),
                trading_strategy: BotStrategy::from_str(&r.trading_strategy).unwrap(),
                symbols: r.symbols.split(',').map(String::from).collect(),
                lookback: r.lookback as usize,
                threshold: r.threshold,
                risk_per_trade: r.risk_per_trade,
                max_positions: r.max_positions as usize,
                timeframes: r.timeframes.split(',').map(String::from).collect(),
                volatility_window: r.volatility_window as usize,
                volatility_threshold: r.volatility_threshold,
            };

            BotInfo {
                config,
                is_running: r.is_running.unwrap(),
            }
        })
        .collect();

    Ok(bots)
}
