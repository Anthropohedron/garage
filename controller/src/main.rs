use std::{env, sync::LazyLock};

use actix_web::{get, http::header::ContentType, post, App, HttpResponse, HttpServer, Responder, web};
mod app;
use app::AppImpl;
type AppData = web::Data<AppImpl>;

const DEFAULT_STATUS_FILENAME: &str = "/var/run/garagemon/status";

pub static STATUS: LazyLock<String> = LazyLock::new(|| {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        panic!("Too many arguments!");
    }
    if args.len() == 2 {
        return args[1].clone();
    }
    return DEFAULT_STATUS_FILENAME.to_string();
});

#[get("/status")]
async fn get_status(data: AppData) -> impl Responder {
    let utils = <AppImpl as Clone>::clone(&data);
    utils.get_status()
}

#[post("/activate")]
async fn activate(data: AppData) -> impl Responder {
    let utils = <AppImpl as Clone>::clone(&data);
    match utils.activate().await {
        Ok(_) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body("{\"success\": true}"),
        Err(msg) => HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .body(format!("{{\"error\": \"{msg}\"}}"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let utils = AppImpl::new(&STATUS);

    HttpServer::new(move || App::new()
            .app_data(utils.clone())
            .service(get_status)
            .service(activate)
        )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}