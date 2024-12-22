use actix_web::{web, App, HttpServer};
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;
use std::sync::Arc;
use hyperliquid::{HyperliquidClient, HyperliquidConfig};

mod schema;
mod handlers;

pub struct AppState {
    conn: DatabaseConnection,
    hl_client: Arc<HyperliquidClient>,
}

async fn establish_connection() -> Result<DatabaseConnection, DbErr> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Database::connect(&database_url).await
}

async fn establish_hyperliquid_client() -> Result<HyperliquidClient, String> {
    let hl_mnemonic = env::var("HL_MNEMONIC").expect("HL_MNEMONIC must be set");
    let hl_endpoint = env::var("HL_ENDPOINT").expect("HL_ENDPOINT must be set");
    let config = HyperliquidConfig {
        endpoint: hl_endpoint,
        signer: hyperliquid::Signer::from_mnemonic(&hl_mnemonic).map_err(|e| e.to_string())?,
    };
    HyperliquidClient::new(config).await.map_err(|e| e.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let conn = establish_connection().await.expect("Failed to connect to database");
    let hl_client = establish_hyperliquid_client().await.expect("Failed to connect to hyperliquid");

    let app_state = web::Data::new(AppState { conn, hl_client: Arc::new(hl_client) });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(|| async { "Hello, world!" }))
            .service(
                web::scope("/api")
                    .service(handlers::markets::get_all_markets)
                    .service(handlers::markets::get_market_by_id)
                    .service(handlers::markets::create_market)
                    .service(handlers::markets::update_market)
                    .service(handlers::markets::delete_market)
                    .service(handlers::user_positions::get_all_user_positions)
                    .service(handlers::user_positions::get_user_position_by_market)
                    .service(handlers::orders::get_all_market_orders)
                    .service(handlers::orders::get_order_by_id)
                    .service(handlers::orders::create_order)
                    .service(handlers::orders::delete_order)
                    .service(handlers::trades::get_all_market_trades)
                    .service(handlers::user_stats::get_user_stats)
                    .service(handlers::markets::mint_tokens)
                    .service(handlers::markets::burn_tokens)
                    .service(handlers::markets::resolve_market)
                    .service(handlers::user_positions::claim_winnings)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
} 