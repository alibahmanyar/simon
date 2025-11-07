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

#[derive(Deserialize)]
struct LoginForm {
    password: String,
}

pub async fn auth_handler(
    State((_, config)): State<(Arc<Mutex<System>>, Config)>,
    request: Request,
) -> impl IntoResponse {
    // Extract form data
    let pass = match Form::<LoginForm>::from_request(request, &()).await {
        Ok(form) => form.password.clone(),
        Err(_) => "".to_string(),
    };

    // Check if password matches
    if bcrypt::verify(pass, &config.password_hash.unwrap_or_default()).unwrap_or(false) {
        // Create JWT token
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = Claims {
            exp: now as usize + 60 * 86400, // 60 days
            iat: now as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .unwrap_or_default();

        return Response::builder()
            .status(StatusCode::OK)
            .header(
                header::SET_COOKIE,
                format!(
                    "simon_auth_token={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=5184000",
                    token
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
    State(config): State<Config>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip authentication for root path to allow login page access
    if request.uri().path() == "/auth" {
        return Ok(next.run(request).await);
    }

    // Extract JWT token from cookie
    let token = match request.headers().get("cookie") {
        Some(cookie) => {
            let cookie_str = cookie.to_str().unwrap_or_default();
            let token = cookie_str
                .split(';')
                .find(|c| c.contains("simon_auth_token"))
                .unwrap_or_default()
                .split('=')
                .nth(1)
                .unwrap_or_default();

            token.to_string()
        }
        None => return Ok(Redirect::temporary("./auth").into_response()),
    };

    // Verify JWT token
    let token_data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(data) => data.claims,
        Err(_) => return Ok(Redirect::temporary("./auth").into_response()),
    };

    // Check if token is expired
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    if token_data.exp < now {
        return Ok(Redirect::temporary("./auth").into_response());
    }

    Ok(next.run(request).await)
}

pub fn apply_auth_middleware(app: Router, config: Config) -> Router {
    app.layer(middleware::from_fn_with_state(config, auth_middleware))
}
