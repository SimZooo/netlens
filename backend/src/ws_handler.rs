use actix_web::{HttpRequest, web::{self, Data}};
use tokio::sync::broadcast;

pub async fn handle_ws(req: HttpRequest, body: web::Payload, tx: Data<broadcast::Sender<String>>) {
    if let Ok((response, session, msg_stream)) = actix_ws::handle(&req, body) {
        let mut rx = tx.subscribe();

        tokio::spawn(async move  {
            loop {
                
            }
        });
    }
}