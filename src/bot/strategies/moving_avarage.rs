use axum::extract::State;
use ta::indicators::ExponentialMovingAverage;
use ta::Next;
use crate::base::AppState;
use crate::bot::BotConfig;
use crate::handlers::bar::get_bars;

pub async fn moving_average_strategy(state: AppState, config: BotConfig){
    //ema
    let mut short_ema = ExponentialMovingAverage::new(10).unwrap();
    let mut long_ema = ExponentialMovingAverage::new(30).unwrap();

    // get historical data
    let price = get_bars(&state, &config.symbols, &config.timeframes[0], 50).await.unwrap();
    let short_ema_value = short_ema.next(10);
    let long_ema_value = long_ema.next(20);

    if short_ema_value > long_ema_value {
        // buy order here
    } else {
        // sell order here
    }
}