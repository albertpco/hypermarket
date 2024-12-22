use actix_web::{web, Responder, get};
use sea_orm::{DatabaseConnection, DbErr, entity::*};
use serde::{Serialize, Deserialize};

use crate::schema::trades;

#[derive(Serialize, Deserialize)]
struct TradeResponse {
    id: i32,
    market_id: String,
    maker_address: String,
    taker_address: String,
    side: String,
    price: String,
    amount: String,
    created_at: String,
    tx_hash: String,
}

#[get("/markets/{market_id}/trades")]
pub async fn get_all_market_trades(conn: web::Data<DatabaseConnection>, market_id: web::Path<String>) -> impl Responder {
    let market_id = market_id.into_inner();
    let result = trades::Entity::find()
        .filter(trades::Column::MarketId.eq(market_id))
        .all(&**conn)
        .await;

    match result {
        Ok(trades) => {
            let trade_responses: Vec<TradeResponse> = trades.into_iter().map(|trade| {
                TradeResponse {
                    id: trade.id,
                    market_id: trade.market_id,
                    maker_address: trade.maker_address,
                    taker_address: trade.taker_address,
                    side: trade.side.to_string(),
                    price: trade.price.to_string(),
                    amount: trade.amount.to_string(),
                    created_at: trade.created_at.to_string(),
                    tx_hash: trade.tx_hash,
                }
            }).collect();
            web::Json(trade_responses)
        }
        Err(e) => {
            eprintln!("Error fetching market trades: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
} 