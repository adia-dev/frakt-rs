use std::sync::{Arc, Mutex};

use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use shared::networking::server::Server;

#[derive(Debug, Deserialize)]
pub struct DirectionQuery {
    direction: String,
}

#[derive(Debug, Deserialize)]
pub struct CycleQuery {
    direction: String,
}

pub async fn cycle_fractal(
    body: web::Json<CycleQuery>,
    server: web::Data<Arc<Mutex<Server>>>,
) -> impl Responder {
    let mut server = server.lock().unwrap();
    match body.direction.to_lowercase().as_str() {
        "previous" => server.previous_fractal(),
        "next" => server.cycle_fractal(),
        _ => {
            server.cycle_fractal();
        }
    }

    HttpResponse::Ok().body(format!("Cycled fractal {:?}", body.direction))
}

pub async fn move_fractal(
    body: web::Json<DirectionQuery>,
    server: web::Data<Arc<Mutex<Server>>>,
) -> impl Responder {
    let mut server = server.lock().unwrap();
    match body.direction.to_lowercase().as_str() {
        "up" => server.move_up(),
        "right" => server.move_right(),
        "down" => server.move_down(),
        "left" => server.move_left(),
        _ => {
            return HttpResponse::BadRequest().body(format!("The direction {} is not supported, please enter one of `top`, `left`, `right`, `bottom`.", body.direction));
        }
    }

    HttpResponse::Ok().body(format!("Moved fractal {:?}", body.direction))
}
