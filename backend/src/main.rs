use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use tokio::sync::broadcast;

mod packets;
mod arp_scan;
mod ws_handler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (tx, _) = broadcast::channel(100);
    tokio::spawn(packets::start_capture(tx.clone()));
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tx.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600)
            )
            .route("/api/packets/get_interfaces", web::get().to(packets::get_interfaces))
            .route("/api/packets/ws/capture/{index}", web::get().to(packets::capture_interface_packets))
            .route("/api/scan/discover_hosts/{index}", web::get().to(arp_scan::discover_hosts))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}