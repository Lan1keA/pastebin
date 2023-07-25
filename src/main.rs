use actix_web::{
    get, http::StatusCode, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use askama::Template;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

#[derive(Template)]
#[template(path = "edit.html")]
struct EditHtml<'a> {
    token: &'a str,
    content: &'a str,
}

static TC: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[get("/")]
async fn readme() -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/readme.html"))
}

fn try_load(token: &str) -> Option<String> {
    Some(TC.lock().ok()?.get(token)?.clone())
}

#[post("/{token}")]
async fn save(web::Path(token): web::Path<String>, content: String) -> impl Responder {
    TC.lock().unwrap().insert(token, content);
    HttpResponse::Ok()
}

#[get("/{token}")]
async fn load(req: HttpRequest, web::Path(token): web::Path<String>) -> impl Responder {
    let content: String = try_load(&token).unwrap_or("".to_string());

    // respond curl
    if let Some(agent) = req.headers().get("User-Agent") {
        if let Ok(agent) = agent.to_str() {
            if agent.starts_with("curl/") {
                return HttpResponse::build(StatusCode::OK)
                    .content_type("text/plain; charset=utf-8")
                    .body(content);
            }
        }
    }

    // respond browser
    let html = EditHtml {
        token: &token,
        content: &content,
    }
    .render()
    .unwrap();

    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(readme).service(load).service(save))
        .bind("0.0.0.0:90")?
        .run()
        .await
}
