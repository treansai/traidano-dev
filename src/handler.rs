// /// Get Account information
// #[instrument(skip(state))]
// pub async fn get_account(
//     State(state): State<Arc<AppState>>,
// ) -> Result<Json<account::Account>, StatusCode> {
//     tracing::info!("app_events: get account information");
//     let client = &state.alpaca_client;
//
//     match client.issue::<account::Get>(&()).await {
//         Ok(account) => {
//             tracing::info!(
//                 account_id = ?account.id,
//                 "Retrieved account information"
//             );
//             Ok(Json(account))
//         }
//         Err(e) => {
//             tracing::error!("Failed to get account: {:?}", e);
//             Err(StatusCode::INTERNAL_SERVER_ERROR)
//         }
//     }
// }
//
