use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Router,
    extract::{FromRequest, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
};

use crate::config::Config;
use axum::extract::Form;
use axum::http::header;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, decode, encode};
use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Serialize, Deserialize)]
struct Claims {
    exp: usize,
    iat: usize,
}

/// Token signing key. Includes the password hash so changing the password
/// invalidates every previously issued token.
fn signing_key(config: &Config) -> Vec<u8> {
    let mut key = config.jwt_secret.as_bytes().to_vec();
    if let Some(hash) = &config.password_hash {
        key.extend_from_slice(hash.as_bytes());
    }
    key
}

/// Finds the value of the cookie called exactly `name`.
fn cookie_value<'a>(cookie_header: &'a str, name: &str) -> Option<&'a str> {
    cookie_header.split(';').find_map(|pair| {
        let (k, v) = pair.trim().split_once('=')?;
        (k == name).then_some(v)
    })
}

#[derive(Deserialize)]
struct LoginForm {
    password: String,
}

pub async fn auth_handler(
    State((_, config)): State<(Arc<Mutex<System>>, Arc<Config>)>,
    request: Request,
) -> impl IntoResponse {
    let secure = request
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("https"));

    // Extract form data
    let pass = match Form::<LoginForm>::from_request(request, &()).await {
        Ok(Form(login_form)) => login_form.password,
        Err(_) => "".to_string(),
    };

    // Check if password matches
    // bcrypt is deliberately slow; keep it off the async worker threads.
    let verified = match config.password_hash.clone() {
        Some(hash) => tokio::task::spawn_blocking(move || bcrypt::verify(&pass, &hash))
            .await
            .map(|r| r.unwrap_or(false))
            .unwrap_or(false),
        None => false,
    };

    if verified {
        // Create JWT token
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = Claims {
            exp: now as usize + 60 * 86400, // 60 days
            iat: now as usize,
        };

        let token = match encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&signing_key(&config)),
        ) {
            Ok(t) => t,
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Failed to create session".to_string())
                    .unwrap();
            }
        };

        return Response::builder()
            .status(StatusCode::OK)
            .header(
                header::SET_COOKIE,
                format!(
                    "simon_auth_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=5184000{}",
                    token,
                    if secure { "; Secure" } else { "" }
                ),
            )
            .body("logged in".to_string())
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body("Unauthorized".to_string())
        .unwrap()
}

// Middleware function to check authentication
async fn auth_middleware(
    State(config): State<Arc<Config>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip authentication for root path to allow login page access
    if request.uri().path() == "/auth" {
        return Ok(next.run(request).await);
    }

    // Extract JWT token from cookie
    let token = request
        .headers()
        .get("cookie")
        .and_then(|c| c.to_str().ok())
        .and_then(|c| cookie_value(c, "simon_auth_token"))
        .unwrap_or_default();

    // Verify JWT token (signature and expiry)
    if decode::<Claims>(
        token,
        &DecodingKey::from_secret(&signing_key(&config)),
        &jsonwebtoken::Validation::default(),
    )
    .is_err()
    {
        // API and WebSocket clients can't follow a redirect to the login page.
        let path = request.uri().path();
        if path.starts_with("/api/")
            || path.starts_with("/ws/")
            || path.starts_with("/container_logs/")
        {
            return Ok(StatusCode::UNAUTHORIZED.into_response());
        }
        return Ok(Redirect::temporary("./auth").into_response());
    }

    Ok(next.run(request).await)
}

pub fn apply_auth_middleware(app: Router, config: Arc<Config>) -> Router {
    app.layer(middleware::from_fn_with_state(config, auth_middleware))
}
