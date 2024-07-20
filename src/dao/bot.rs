use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    bot::{BotConfig, BotInfo},
    error::RequestError,
};
use crate::error::Error;

pub async fn create_bot(db: PgPool, data: BotConfig) -> Result<String, Error> {
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
    .fetch_one(&db)
    .await?;

    Ok(bot_id)
}
