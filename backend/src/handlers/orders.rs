use actix_web::{web, Responder, get, post, delete};
use sea_orm::{DatabaseConnection, DbErr, entity::*, ActiveValue};
use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;

use crate::schema::{orders, OrderSide, OrderStatus};

#[derive(Serialize, Deserialize)]
struct OrderResponse {
    id: i32,
    market_id: String,
    user_address: String,
    side: String,
    price: String,
    amount: String,
    filled_amount: String,
    status: String,
    created_at: String,
    updated_at: String,
    tx_hash: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CreateOrderRequest {
    market_id: String,
    user_address: String,
    side: String,
    price: String,
    amount: String,
}

#[get("/markets/{market_id}/orders")]
pub async fn get_all_market_orders(conn: web::Data<DatabaseConnection>, market_id: web::Path<String>) -> impl Responder {
    let market_id = market_id.into_inner();
    let result = orders::Entity::find()
        .filter(orders::Column::MarketId.eq(market_id))
        .all(&**conn)
        .await;

    match result {
        Ok(orders) => {
            let order_responses: Vec<OrderResponse> = orders.into_iter().map(|order| {
                OrderResponse {
                    id: order.id,
                    market_id: order.market_id,
                    user_address: order.user_address,
                    side: order.side.to_string(),
                    price: order.price.to_string(),
                    amount: order.amount.to_string(),
                    filled_amount: order.filled_amount.to_string(),
                    status: order.status.to_string(),
                    created_at: order.created_at.to_string(),
                    updated_at: order.updated_at.to_string(),
                    tx_hash: order.tx_hash,
                }
            }).collect();
            web::Json(order_responses)
        }
        Err(e) => {
            eprintln!("Error fetching market orders: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/orders/{order_id}")]
pub async fn get_order_by_id(conn: web::Data<DatabaseConnection>, order_id: web::Path<i32>) -> impl Responder {
    let order_id = order_id.into_inner();
    let result = orders::Entity::find_by_id(order_id).one(&**conn).await;

    match result {
        Ok(Some(order)) => {
            let order_response = OrderResponse {
                id: order.id,
                market_id: order.market_id,
                user_address: order.user_address,
                side: order.side.to_string(),
                price: order.price.to_string(),
                amount: order.amount.to_string(),
                filled_amount: order.filled_amount.to_string(),
                status: order.status.to_string(),
                created_at: order.created_at.to_string(),
                updated_at: order.updated_at.to_string(),
                tx_hash: order.tx_hash,
            };
            web::Json(order_response)
        }
        Ok(None) => {
            web::HttpResponse::NotFound().finish()
        }
        Err(e) => {
            eprintln!("Error fetching order: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/markets/{market_id}/orders")]
pub async fn create_order(conn: web::Data<DatabaseConnection>, req: web::Json<CreateOrderRequest>) -> impl Responder {
    let new_order = orders::ActiveModel {
        id: ActiveValue::NotSet,
        market_id: ActiveValue::Set(req.market_id.clone()),
        user_address: ActiveValue::Set(req.user_address.clone()),
        side: ActiveValue::Set(OrderSide::from_str(&req.side).unwrap()),
        price: ActiveValue::Set(Decimal::from_str(&req.price).unwrap()),
        amount: ActiveValue::Set(Decimal::from_str(&req.amount).unwrap()),
        filled_amount: ActiveValue::Set(Decimal::from(0)),
        status: ActiveValue::Set(OrderStatus::Open),
        created_at: ActiveValue::Set(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64),
        updated_at: ActiveValue::Set(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64),
        tx_hash: ActiveValue::Set(None),
    };

    let result = orders::Entity::insert(new_order).exec(&**conn).await;

    match result {
        Ok(insert_result) => {
            let order_id = insert_result.last_insert_id;
            let order = orders::Entity::find_by_id(order_id).one(&**conn).await.unwrap().unwrap();
            let order_response = OrderResponse {
                id: order.id,
                market_id: order.market_id,
                user_address: order.user_address,
                side: order.side.to_string(),
                price: order.price.to_string(),
                amount: order.amount.to_string(),
                filled_amount: order.filled_amount.to_string(),
                status: order.status.to_string(),
                created_at: order.created_at.to_string(),
                updated_at: order.updated_at.to_string(),
                tx_hash: order.tx_hash,
            };
            web::Json(order_response)
        }
        Err(e) => {
            eprintln!("Error creating order: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}

#[delete("/orders/{order_id}")]
pub async fn delete_order(conn: web::Data<DatabaseConnection>, order_id: web::Path<i32>) -> impl Responder {
    let order_id = order_id.into_inner();
    let result = orders::Entity::delete_by_id(order_id).exec(&**conn).await;

    match result {
        Ok(delete_result) => {
            if delete_result.rows_affected > 0 {
                web::HttpResponse::Ok().finish()
            } else {
                web::HttpResponse::NotFound().finish()
            }
        }
        Err(e) => {
            eprintln!("Error deleting order: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
} 