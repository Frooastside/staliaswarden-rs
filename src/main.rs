use axum::{extract::{Json, State}, http::{StatusCode}, middleware, middleware::Next, response::IntoResponse, routing::post, Router};
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr, sync::Arc};
use axum::extract::Request;
use uuid::Uuid;
use reqwest::Client;
use dotenvy::dotenv;
use middleware::from_fn_with_state;

#[derive(Clone)]
struct AppConfig {
    port: u16,
    api_token: String,
    alias_domain: String,
    forward_to: String,
    stalwart_url: String,
    stalwart_username: String,
    stalwart_password: String,
    http_client: Client,
}

#[derive(Deserialize)]
struct AliasRequest {
    domain: Option<String>,
}

#[derive(Serialize)]
struct AliasResponse {
    data: AliasResponseData,
}

#[derive(Serialize)]
struct AliasResponseData {
    id: u64,
    email: String,
    local_part: String,
    domain: String,
    description: Option<String>,
    enabled: bool,
}

struct Alias {
    username: String,
    domain: String,
    address: String
}

impl Alias {
    fn new(username: String, domain: String) -> Self {
        let address = format!("{}@{}", username, domain);
        Self {username, domain,address }
    }
}

#[tokio::main]
async fn main() {
    let _path = dotenv().ok();

    let cfg = Arc::new(AppConfig {
        port: env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000),
        api_token: env::var("API_TOKEN").unwrap(),
        alias_domain: env::var("ALIAS_DOMAIN").unwrap(),
        forward_to: env::var("FORWARD_TO").unwrap(),
        stalwart_url: env::var("STALWART_URL").unwrap(),
        stalwart_username: env::var("STALWART_USERNAME").unwrap(),
        stalwart_password: env::var("STALWART_PASSWORD").unwrap(),
        http_client: Client::new(),
    });

    let app = Router::new()
        .route("/api/v1/aliases", post(create_alias))
        .with_state(cfg.clone())
        .layer(from_fn_with_state(cfg.clone(), auth));

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    println!("Alias service running on port {}", cfg.port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn auth(
    State(cfg): State<Arc<AppConfig>>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let Some(auth_header) = request.headers().get("authorization") else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let auth_str = auth_header.to_str().unwrap_or("");
    let token = auth_str.strip_prefix("Bearer ").unwrap_or("");

    if token != cfg.api_token {
        return StatusCode::UNAUTHORIZED.into_response()
    }

    next.run(request).await
}

async fn create_alias(
    State(cfg): State<Arc<AppConfig>>,
    Json(payload): Json<AliasRequest>,
) -> impl IntoResponse {
    let alias = generate_alias(payload.domain, &cfg);

    if let Err(err) = add_alias_stalwart(&alias, &cfg).await {
        eprintln!("Failed to add alias: {:?}", err);
        return Err(
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    let now = chrono::Utc::now().timestamp_millis() as u64;

    let resp = AliasResponse {
        data: AliasResponseData {
            id: now,
            email: alias.address,
            local_part: alias.username,
            domain: alias.domain,
            description: None,
            enabled: true,
        },
    };

    Ok((StatusCode::CREATED, Json(resp)).into_response())
}

fn generate_alias(domain: Option<String>, config: &AppConfig) -> Alias {
    let id = Uuid::new_v4().to_string();
    let short = id.split('-').next().unwrap();

    let selected_domain = match domain {
        Some(ref domain) if domain != "random" => domain.clone(),
        _ => config.alias_domain.clone(),
    };

    Alias::new(short.to_owned(), selected_domain)
}

async fn add_alias_stalwart(alias: &Alias, config: &AppConfig) -> Result<(), reqwest::Error> {
    let url = format!("{}/principal/{}", config.stalwart_url, config.forward_to);


    let body = serde_json::json!([
        {
            "action": "addItem",
            "field": "emails",
            "value": alias.address
        }
    ]);

    let res = config
        .http_client
        .patch(url)
        .basic_auth(&config.stalwart_username, Some(&config.stalwart_password))
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        eprintln!("Stalwart error: {:?}", res.text().await.ok());
    }

    println!("Alias {} added to Stalwart", alias.address);
    Ok(())
}
