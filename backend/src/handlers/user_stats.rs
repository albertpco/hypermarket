use actix_web::{web, Responder, get};
use sea_orm::{DatabaseConnection, DbErr, entity::*};
use serde::{Serialize, Deserialize};

use crate::schema::user_stats;

#[derive(Serialize, Deserialize)]
struct UserStatsResponse {
    user_address: String,
    total_trades: i32,
    total_volume: String,
    total_pnl: String,
    markets_participated: i32,
    created_at: String,
    updated_at: String,
}

#[get("/users/{user_address}/stats")]
pub async fn get_user_stats(conn: web::Data<DatabaseConnection>, user_address: web::Path<String>) -> impl Responder {
    let user_address = user_address.into_inner();
    let result = user_stats::Entity::find_by_id(user_address).one(&**conn).await;

    match result {
        Ok(Some(stats)) => {
            let stats_response = UserStatsResponse {
                user_address: stats.user_address,
                total_trades: stats.total_trades,
                total_volume: stats.total_volume.to_string(),
                total_pnl: stats.total_pnl.to_string(),
                markets_participated: stats.markets_participated,
                created_at: stats.created_at.to_string(),
                updated_at: stats.updated_at.to_string(),
            };
            web::Json(stats_response)
        }
        Ok(None) => {
            web::HttpResponse::NotFound().finish()
        }
        Err(e) => {
            eprintln!("Error fetching user stats: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
} 