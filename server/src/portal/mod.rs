use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use actix_cors::Cors;
use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use log::info;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

pub async fn run_portal() -> std::io::Result<()> {
    info!("🌀 Portal launched at localhost:8686");

    let server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8686))?
    .run();

    let server_handle = server.handle();
    let task_shutdown_marker = Arc::new(AtomicBool::new(false));

    let server_task = tokio::spawn(async move {
        let _ = server.await;
    });

    tokio::spawn(async move {
        // Listen for ctrl-c
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c signal");

        // Start shutdown of tasks
        server_handle.stop(true).await;
        task_shutdown_marker.store(true, Ordering::SeqCst);
    });

    // Await server task completion
    server_task.await.expect("Server task failed");

    Ok(())
}

