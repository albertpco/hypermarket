use actix_web::{web, Responder, get, post, put, delete};
use sea_orm::{DatabaseConnection, DbErr, entity::*, ActiveValue};
use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;
use validator::Validate;
use std::sync::Arc;
use hyperliquid::{HyperliquidClient, HyperliquidConfig};

use crate::schema::markets;

#[derive(Serialize, Deserialize)]
struct MarketResponse {
    id: String,
    question: String,
    expiry_timestamp: i64,
    oracle_id: String,
    collateral_token: String,
    status: String,
    yes_token_address: String,
    no_token_address: String,
    resolved_outcome: Option<bool>,
    created_at: String,
    updated_at: String,
    total_volume: String,
    current_price: String,
    creator_address: String,
}

#[derive(Serialize, Deserialize, Validate)]
struct CreateMarketRequest {
    #[validate(length(min = 1))]
    question: String,
    #[validate(range(min = 1))]
    expiry_timestamp: i64,
    #[validate(length(min = 1))]
    oracle_id: String,
    #[validate(length(min = 1))]
    collateral_token: String,
    #[validate(length(min = 1))]
    yes_token_address: String,
    #[validate(length(min = 1))]
    no_token_address: String,
    #[validate(length(min = 1))]
    creator_address: String,
}

#[derive(Serialize, Deserialize, Validate)]
struct UpdateMarketRequest {
    #[validate(length(min = 1))]
    question: Option<String>,
    #[validate(range(min = 1))]
    expiry_timestamp: Option<i64>,
    #[validate(length(min = 1))]
    oracle_id: Option<String>,
    #[validate(length(min = 1))]
    collateral_token: Option<String>,
    #[validate(length(min = 1))]
    status: Option<String>,
    #[validate(length(min = 1))]
    yes_token_address: Option<String>,
    #[validate(length(min = 1))]
    no_token_address: Option<String>,
    resolved_outcome: Option<bool>,
}

#[derive(Serialize, Deserialize, Validate)]
struct MintBurnRequest {
    #[validate(length(min = 1))]
    user_address: String,
    #[validate(length(min = 1))]
    market_id: String,
    #[validate(length(min = 1))]
    yes_amount: String,
    #[validate(length(min = 1))]
    no_amount: String,
}

#[derive(Serialize, Deserialize, Validate)]
struct ResolveMarketRequest {
    outcome: bool,
}

#[get("/markets")]
pub async fn get_all_markets(conn: web::Data<DatabaseConnection>) -> impl Responder {
    let result = markets::Entity::find().all(&**conn).await;

    match result {
        Ok(markets) => {
            let market_responses: Vec<MarketResponse> = markets.into_iter().map(|market| {
                MarketResponse {
                    id: market.id,
                    question: market.question,
                    expiry_timestamp: market.expiry_timestamp,
                    oracle_id: market.oracle_id,
                    collateral_token: market.collateral_token,
                    status: market.status.to_string(),
                    yes_token_address: market.yes_token_address,
                    no_token_address: market.no_token_address,
                    resolved_outcome: market.resolved_outcome,
                    created_at: market.created_at.to_string(),
                    updated_at: market.updated_at.to_string(),
                    total_volume: market.total_volume.to_string(),
                    current_price: market.current_price.to_string(),
                    creator_address: market.creator_address,
                }
            }).collect();
            web::Json(market_responses)
        }
        Err(e) => {
            eprintln!("Error fetching markets: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/markets/{market_id}")]
pub async fn get_market_by_id(conn: web::Data<DatabaseConnection>, market_id: web::Path<String>) -> impl Responder {
    let result = markets::Entity::find_by_id(market_id.into_inner()).one(&**conn).await;

    match result {
        Ok(Some(market)) => {
            let market_response = MarketResponse {
                id: market.id,
                question: market.question,
                expiry_timestamp: market.expiry_timestamp,
                oracle_id: market.oracle_id,
                collateral_token: market.collateral_token,
                status: market.status.to_string(),
                yes_token_address: market.yes_token_address,
                no_token_address: market.no_token_address,
                resolved_outcome: market.resolved_outcome,
                created_at: market.created_at.to_string(),
                updated_at: market.updated_at.to_string(),
                total_volume: market.total_volume.to_string(),
                current_price: market.current_price.to_string(),
                creator_address: market.creator_address,
            };
            web::Json(market_response)
        }
        Ok(None) => {
            web::HttpResponse::NotFound().finish()
        }
        Err(e) => {
            eprintln!("Error fetching market: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/markets")]
pub async fn create_market(conn: web::Data<DatabaseConnection>, req: web::Json<CreateMarketRequest>) -> impl Responder {
    if let Err(e) = req.validate() {
        return web::HttpResponse::BadRequest().json(format!("Validation error: {}", e));
    }
    let new_market = markets::ActiveModel {
        id: ActiveValue::NotSet,
        question: ActiveValue::Set(req.question.clone()),
        expiry_timestamp: ActiveValue::Set(req.expiry_timestamp),
        oracle_id: ActiveValue::Set(req.oracle_id.clone()),
        collateral_token: ActiveValue::Set(req.collateral_token.clone()),
        status: ActiveValue::Set(crate::schema::MarketStatus::Active),
        yes_token_address: ActiveValue::Set(req.yes_token_address.clone()),
        no_token_address: ActiveValue::Set(req.no_token_address.clone()),
        resolved_outcome: ActiveValue::Set(None),
        created_at: ActiveValue::Set(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64),
        updated_at: ActiveValue::Set(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64),
        total_volume: ActiveValue::Set(Decimal::from(0)),
        current_price: ActiveValue::Set(Decimal::from(0)),
        creator_address: ActiveValue::Set(req.creator_address.clone()),
    };

    let result = markets::Entity::insert(new_market).exec(&**conn).await;

    match result {
        Ok(insert_result) => {
            let market_id = insert_result.last_insert_id;
            let market = markets::Entity::find_by_id(market_id).one(&**conn).await.unwrap().unwrap();
            let market_response = MarketResponse {
                id: market.id,
                question: market.question,
                expiry_timestamp: market.expiry_timestamp,
                oracle_id: market.oracle_id,
                collateral_token: market.collateral_token,
                status: market.status.to_string(),
                yes_token_address: market.yes_token_address,
                no_token_address: market.no_token_address,
                resolved_outcome: market.resolved_outcome,
                created_at: market.created_at.to_string(),
                updated_at: market.updated_at.to_string(),
                total_volume: market.total_volume.to_string(),
                current_price: market.current_price.to_string(),
                creator_address: market.creator_address,
            };
            web::Json(market_response)
        }
        Err(e) => {
            eprintln!("Error creating market: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}

#[put("/markets/{market_id}")]
pub async fn update_market(conn: web::Data<DatabaseConnection>, market_id: web::Path<String>, req: web::Json<UpdateMarketRequest>) -> impl Responder {
    if let Err(e) = req.validate() {
        return web::HttpResponse::BadRequest().json(format!("Validation error: {}", e));
    }
    let market_id = market_id.into_inner();
    let market = markets::Entity::find_by_id(market_id.clone()).one(&**conn).await;

    match market {
        Ok(Some(market)) => {
            let mut market: markets::ActiveModel = market.into();
            if let Some(question) = &req.question {
                market.question = ActiveValue::Set(question.clone());
            }
            if let Some(expiry_timestamp) = req.expiry_timestamp {
                market.expiry_timestamp = ActiveValue::Set(expiry_timestamp);
            }
            if let Some(oracle_id) = &req.oracle_id {
                market.oracle_id = ActiveValue::Set(oracle_id.clone());
            }
            if let Some(collateral_token) = &req.collateral_token {
                market.collateral_token = ActiveValue::Set(collateral_token.clone());
            }
            if let Some(status) = &req.status {
                market.status = ActiveValue::Set(crate::schema::MarketStatus::from_str(status).unwrap());
            }
            if let Some(yes_token_address) = &req.yes_token_address {
                market.yes_token_address = ActiveValue::Set(yes_token_address.clone());
            }
            if let Some(no_token_address) = &req.no_token_address {
                market.no_token_address = ActiveValue::Set(no_token_address.clone());
            }
            if let Some(resolved_outcome) = req.resolved_outcome {
                market.resolved_outcome = ActiveValue::Set(Some(resolved_outcome));
            }
            market.updated_at = ActiveValue::Set(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64);

            let result = market.update(&**conn).await;

            match result {
                Ok(market) => {
                    let market_response = MarketResponse {
                        id: market.id,
                        question: market.question,
                        expiry_timestamp: market.expiry_timestamp,
                        oracle_id: market.oracle_id,
                        collateral_token: market.collateral_token,
                        status: market.status.to_string(),
                        yes_token_address: market.yes_token_address,
                        no_token_address: market.no_token_address,
                        resolved_outcome: market.resolved_outcome,
                        created_at: market.created_at.to_string(),
                        updated_at: market.updated_at.to_string(),
                        total_volume: market.total_volume.to_string(),
                        current_price: market.current_price.to_string(),
                        creator_address: market.creator_address,
                    };
                    web::Json(market_response)
                }
                Err(e) => {
                    eprintln!("Error updating market: {}", e);
                    web::HttpResponse::InternalServerError().finish()
                }
            }
        }
        Ok(None) => {
            web::HttpResponse::NotFound().finish()
        }
        Err(e) => {
            eprintln!("Error fetching market: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}

#[delete("/markets/{market_id}")]
pub async fn delete_market(conn: web::Data<DatabaseConnection>, market_id: web::Path<String>) -> impl Responder {
    let market_id = market_id.into_inner();
    let result = markets::Entity::delete_by_id(market_id).exec(&**conn).await;

    match result {
        Ok(delete_result) => {
            if delete_result.rows_affected > 0 {
                web::HttpResponse::Ok().finish()
            } else {
                web::HttpResponse::NotFound().finish()
            }
        }
        Err(e) => {
            eprintln!("Error deleting market: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/markets/{market_id}/mint")]
pub async fn mint_tokens(conn: web::Data<DatabaseConnection>, hl_client: web::Data<Arc<HyperliquidClient>>, market_id: web::Path<String>, req: web::Json<MintBurnRequest>) -> impl Responder {
    if let Err(e) = req.validate() {
        return web::HttpResponse::BadRequest().json(format!("Validation error: {}", e));
    }
    let market_id = market_id.into_inner();
    let user_address = req.user_address.clone();
    let yes_amount = Decimal::from_str(&req.yes_amount).unwrap();
    let no_amount = Decimal::from_str(&req.no_amount).unwrap();

    let mint_result = hl_client.mint_tokens(user_address.clone(), market_id.clone(), yes_amount, no_amount).await;

    match mint_result {
        Ok(tx_hash) => {
            println!("Minted tokens for user: {}, market: {}, yes_amount: {}, no_amount: {}, tx_hash: {}", user_address, market_id, yes_amount, no_amount, tx_hash);
            
            let user_position = crate::schema::user_positions::Entity::find()
                .filter(crate::schema::user_positions::Column::UserAddress.eq(user_address.clone()))
                .filter(crate::schema::user_positions::Column::MarketId.eq(market_id.clone()))
                .one(&**conn)
                .await;

            match user_position {
                Ok(Some(mut position)) => {
                    let mut position: crate::schema::user_positions::ActiveModel = position.into();
                    position.yes_token_balance = ActiveValue::Set((Decimal::from_str(&position.yes_token_balance.to_string()).unwrap() + yes_amount).to_string());
                    position.no_token_balance = ActiveValue::Set((Decimal::from_str(&position.no_token_balance.to_string()).unwrap() + no_amount).to_string());
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
                    let new_position = crate::schema::user_positions::ActiveModel {
                        id: ActiveValue::NotSet,
                        user_address: ActiveValue::Set(user_address.clone()),
                        market_id: ActiveValue::Set(market_id.clone()),
                        yes_token_balance: ActiveValue::Set(yes_amount.to_string()),
                        no_token_balance: ActiveValue::Set(no_amount.to_string()),
                        collateral_balance: ActiveValue::Set(Decimal::from(0).to_string()),
                        last_updated_at: ActiveValue::Set(std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64),
                    };
                    let insert_result = crate::schema::user_positions::Entity::insert(new_position).exec(&**conn).await;
                    match insert_result {
                        Ok(_) => {
                            web::HttpResponse::Ok().finish()
                        }
                        Err(e) => {
                            eprintln!("Error creating user position: {}", e);
                            web::HttpResponse::InternalServerError().json(format!("Error creating user position: {}", e))
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching user position: {}", e);
                    web::HttpResponse::InternalServerError().json(format!("Error fetching user position: {}", e))
                }
            }
        }
        Err(e) => {
            eprintln!("Error minting tokens: {}", e);
            web::HttpResponse::InternalServerError().json(format!("Error minting tokens: {}", e))
        }
    }
}

#[post("/markets/{market_id}/burn")]
pub async fn burn_tokens(conn: web::Data<DatabaseConnection>, hl_client: web::Data<Arc<HyperliquidClient>>, market_id: web::Path<String>, req: web::Json<MintBurnRequest>) -> impl Responder {
    if let Err(e) = req.validate() {
        return web::HttpResponse::BadRequest().json(format!("Validation error: {}", e));
    }
    let market_id = market_id.into_inner();
    let user_address = req.user_address.clone();
    let yes_amount = Decimal::from_str(&req.yes_amount).unwrap();
    let no_amount = Decimal::from_str(&req.no_amount).unwrap();

    let burn_result = hl_client.burn_tokens(user_address.clone(), market_id.clone(), yes_amount, no_amount).await;

    match burn_result {
        Ok(tx_hash) => {
             println!("Burned tokens for user: {}, market: {}, yes_amount: {}, no_amount: {}, tx_hash: {}", user_address, market_id, yes_amount, no_amount, tx_hash);
            
            let user_position = crate::schema::user_positions::Entity::find()
                .filter(crate::schema::user_positions::Column::UserAddress.eq(user_address.clone()))
                .filter(crate::schema::user_positions::Column::MarketId.eq(market_id.clone()))
                .one(&**conn)
                .await;

            match user_position {
                Ok(Some(mut position)) => {
                    let mut position: crate::schema::user_positions::ActiveModel = position.into();
                    position.yes_token_balance = ActiveValue::Set((Decimal::from_str(&position.yes_token_balance.to_string()).unwrap() - yes_amount).to_string());
                    position.no_token_balance = ActiveValue::Set((Decimal::from_str(&position.no_token_balance.to_string()).unwrap() - no_amount).to_string());
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
                    eprintln!("User position not found for burn: user: {}, market: {}", user_address, market_id);
                    web::HttpResponse::NotFound().json(format!("User position not found for burn: user: {}, market: {}", user_address, market_id))
                }
                Err(e) => {
                    eprintln!("Error fetching user position: {}", e);
                    web::HttpResponse::InternalServerError().json(format!("Error fetching user position: {}", e))
                }
            }
        }
        Err(e) => {
            eprintln!("Error burning tokens: {}", e);
            web::HttpResponse::InternalServerError().json(format!("Error burning tokens: {}", e))
        }
    }
}

#[post("/markets/{market_id}/resolve")]
pub async fn resolve_market(conn: web::Data<DatabaseConnection>, hl_client: web::Data<Arc<HyperliquidClient>>, market_id: web::Path<String>, req: web::Json<ResolveMarketRequest>) -> impl Responder {
    if let Err(e) = req.validate() {
         return web::HttpResponse::BadRequest().json(format!("Validation error: {}", e));
    }
    let market_id = market_id.into_inner();
    let outcome = req.outcome;

    let resolve_result = hl_client.resolve_market(market_id.clone(), outcome).await;

    match resolve_result {
        Ok(tx_hash) => {
            println!("Resolved market: {}, outcome: {}, tx_hash: {}", market_id, outcome, tx_hash);
            
            let market = crate::schema::markets::Entity::find_by_id(market_id.clone()).one(&**conn).await;

            match market {
                Ok(Some(mut market)) => {
                    let mut market: crate::schema::markets::ActiveModel = market.into();
                    market.status = ActiveValue::Set(crate::schema::MarketStatus::Resolved);
                    market.resolved_outcome = ActiveValue::Set(Some(outcome));
                    let update_result = market.update(&**conn).await;
                    match update_result {
                        Ok(_) => {
                            web::HttpResponse::Ok().finish()
                        }
                        Err(e) => {
                            eprintln!("Error updating market: {}", e);
                            web::HttpResponse::InternalServerError().json(format!("Error updating market: {}", e))
                        }
                    }
                }
                Ok(None) => {
                    eprintln!("Market not found for resolve: market: {}", market_id);
                    web::HttpResponse::NotFound().json(format!("Market not found for resolve: market: {}", market_id))
                }
                Err(e) => {
                    eprintln!("Error fetching market: {}", e);
                    web::HttpResponse::InternalServerError().json(format!("Error fetching market: {}", e))
                }
            }
        }
        Err(e) => {
            eprintln!("Error resolving market: {}", e);
            web::HttpResponse::InternalServerError().json(format!("Error resolving market: {}", e))
        }
    }
} 