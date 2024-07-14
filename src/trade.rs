use traidano::models::account::Account;

pub fn calculate_position_size(
    account: Account,
    current_price: f64,
    risk_per_trade:  f64
) -> f64 {
    let risk_amount = account.equity * risk_per_trade;
    let shares = (risk_amount/ current_price).floor();
    shares.min(account.buying_power/ current_price)
}