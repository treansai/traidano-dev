use apca::{ApiInfo, Client, api::v2::{account}, RequestError};
use apca::api::v2::account::{Account, GetError};
use tokio::task::unconstrained;
use tracing::instrument::WithSubscriber;
use crate::configuration::BaseConfig;

mod trade;
mod configuration;


#[tokio::main]
async fn main() {
    // init tracing
    tracing_subscriber::fmt::init();

    tracing::info!("App start");
    // connection to
    // configuration of api
    let config = configuration::build_config().expect("cannot load configuration");
    let api_config = config.api_config;
    let api_config = ApiInfo::from_parts(
        api_config.base_url,
        api_config.api_key,
        api_config.secret
    ).unwrap();

    // alpaca client
    let client = Client::new(api_config);
    let account = get_account(client).await.unwrap();

    tracing::event!(
        name: "account_info",
        tracing::Level::INFO,
        r#"
        account_id : {:?}
        "#,
        account.id
    )

}

async fn get_account(client: Client) -> Result<Account, ()> {
    tracing::info!("app_events: get account information");
    let account = client
        .issue::<account::Get>(&())
        .await
        .expect("error to get account");

    Ok(account)
}