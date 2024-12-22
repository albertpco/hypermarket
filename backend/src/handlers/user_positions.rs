use actix_web::{web, Responder, get, post};
use sea_orm::{DatabaseConnection, DbErr, entity::*};
use serde::{Serialize, Deserialize};
use validator::Validate;
use std::sync::Arc;

use crate::schema::user_positions;

#[derive(Serialize, Deserialize)]
struct UserPositionResponse {
    id: i32,
    user_address: String,
    market_id: String,
    yes_token_balance: String,
    no_token_balance: String,
    collateral_balance: String,
    last_updated_at: i64,
}

#[derive(Serialize, Deserialize, Validate)]
struct ClaimWinningsRequest {
    #[validate(length(min = 1))]
    market_id: String,
}

#[get("/users/{user_address}/positions")]
pub async fn get_all_user_positions(conn: web::Data<DatabaseConnection>, user_address: web::Path<String>) -> impl Responder {
    let user_address = user_address.into_inner();
    let result = user_positions::Entity::find()
        .filter(user_positions::Column::UserAddress.eq(user_address))
        .all(&**conn)
        .await;

    match result {
        Ok(positions) => {
            let position_responses: Vec<UserPositionResponse> = positions.into_iter().map(|position| {
                UserPositionResponse {
                    id: position.id,
                    user_address: position.user_address,
                    market_id: position.market_id,
                    yes_token_balance: position.yes_token_balance,
                    no_token_balance: position.no_token_balance,
                    collateral_balance: position.collateral_balance,
                    last_updated_at: position.last_updated_at,
                }
            }).collect();
            web::Json(position_responses)
        }
        Err(e) => {
            eprintln!("Error fetching user positions: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/users/{user_address}/positions/{market_id}")]
pub async fn get_user_position_by_market(conn: web::Data<DatabaseConnection>, path: web::Path<(String, String)>) -> impl Responder {
    let (user_address, market_id) = path.into_inner();
    let result = user_positions::Entity::find()
        .filter(user_positions::Column::UserAddress.eq(user_address))
        .filter(user_positions::Column::MarketId.eq(market_id))
        .one(&**conn)
        .await;

    match result {
        Ok(Some(position)) => {
            let position_response = UserPositionResponse {
                id: position.id,
                user_address: position.user_address,
                market_id: position.market_id,
                yes_token_balance: position.yes_token_balance,
                no_token_balance: position.no_token_balance,
                collateral_balance: position.collateral_balance,
                last_updated_at: position.last_updated_at,
            };
            web::Json(position_response)
        }
        Ok(None) => {
            web::HttpResponse::NotFound().finish()
        }
        Err(e) => {
            eprintln!("Error fetching user position: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/users/{user_address}/positions/claim")]
pub async fn claim_winnings(conn: web::Data<DatabaseConnection>, hl_client: web::Data<Arc<HyperliquidClient>>, user_address: web::Path<String>, req: web::Json<ClaimWinningsRequest>) -> impl Responder {
    if let Err(e) = req.validate() {
        return web::HttpResponse::BadRequest().json(format!("Validation error: {}", e));
    }
    let user_address = user_address.into_inner();
    let market_id = req.market_id.clone();

    let claim_result = hl_client.claim_winnings(user_address.clone(), market_id.clone()).await;

    match claim_result {
        Ok(tx_hash) => {
            println!("Claimed winnings for user: {}, market: {}, tx_hash: {}", user_address, market_id, tx_hash);
            
            let user_position = crate::schema::user_positions::Entity::find()
                .filter(crate::schema::user_positions::Column::UserAddress.eq(user_address.clone()))
                .filter(crate::schema::user_positions::Column::MarketId.eq(market_id.clone()))
                .one(&**conn)
                .await;

            match user_position {
                Ok(Some(mut position)) => {
                     let mut position: crate::schema::user_positions::ActiveModel = position.into();
                    // For now, we are not updating the collateral balance, but we could add logic here to do so
                    position.last_updated_at = ActiveValue::Set(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64);
                    let update_result = position.update(&**conn).await;
                    match update_result {
                        Ok(_) => {
                            web::HttpResponse::Ok().finish()
                        }
                        Err(e) => {
                            eprintln!("Error updating user position: {}", e);
                            web::HttpResponse::InternalServerError().json(format!("Error updating user position: {}", e))
                        }
                    }
                }
                Ok(None) => {
                    eprintln!("User position not found for claim: user: {}, market: {}", user_address, market_id);
                    web::HttpResponse::NotFound().json(format!("User position not found for claim: user: {}, market: {}", user_address, market_id))
                }
                Err(e) => {
                    eprintln!("Error fetching user position: {}", e);
                    web::HttpResponse::InternalServerError().json(format!("Error fetching user position: {}", e))
                }
            }
        }
        Err(e) => {
            eprintln!("Error claiming winnings: {}", e);
            web::HttpResponse::InternalServerError().json(format!("Error claiming winnings: {}", e))
        }
    }
} 