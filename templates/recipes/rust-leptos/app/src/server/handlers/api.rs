//! API Routes

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::User;
use crate::server::actors::{CreateSession, GetSession, SessionManager};
use crate::server::AppState;

/// Configure API routes
pub fn api_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/users", web::get().to(list_users))
            .route("/users/{id}", web::get().to(get_user))
            .route("/sessions", web::post().to(create_session))
            .route("/sessions/{id}", web::get().to(get_session))
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// User Handlers
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct UsersResponse {
    users: Vec<User>,
    total: usize,
}

async fn list_users(state: web::Data<AppState>) -> HttpResponse {
    // Example: fetch from database
    let users = sqlx::query_as!(
        User,
        r#"SELECT id, email, name, created_at, updated_at FROM users LIMIT 100"#
    )
    .fetch_all(&*state.db)
    .await;

    match users {
        Ok(users) => HttpResponse::Ok().json(UsersResponse {
            total: users.len(),
            users,
        }),
        Err(e) => {
            tracing::error!("Failed to fetch users: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch users"
            }))
        }
    }
}

async fn get_user(state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse {
    let user_id = path.into_inner();

    let user = sqlx::query_as!(
        User,
        r#"SELECT id, email, name, created_at, updated_at FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_optional(&*state.db)
    .await;

    match user {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        })),
        Err(e) => {
            tracing::error!("Failed to fetch user: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch user"
            }))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Session Handlers
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CreateSessionRequest {
    user_id: Uuid,
}

#[derive(Serialize)]
struct SessionResponse {
    session_id: Uuid,
    user_id: Uuid,
    expires_at: chrono::DateTime<chrono::Utc>,
}

async fn create_session(
    state: web::Data<AppState>,
    body: web::Json<CreateSessionRequest>,
) -> HttpResponse {
    // First, fetch the user
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, email, name, created_at, updated_at FROM users WHERE id = $1"#,
        body.user_id
    )
    .fetch_optional(&*state.db)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "User not found"
            }))
        }
        Err(e) => {
            tracing::error!("Failed to fetch user: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create session"
            }));
        }
    };

    // Create session via actor
    let session_manager = SessionManager::get();
    let result = session_manager
        .send(CreateSession {
            user,
            ttl_seconds: None,
        })
        .await;

    match result {
        Ok(Ok(session)) => HttpResponse::Created().json(SessionResponse {
            session_id: session.id,
            user_id: session.user_id,
            expires_at: session.expires_at,
        }),
        Ok(Err(e)) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
        Err(e) => {
            tracing::error!("Actor mailbox error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Service unavailable"
            }))
        }
    }
}

async fn get_session(path: web::Path<Uuid>) -> HttpResponse {
    let session_id = path.into_inner();

    let session_manager = SessionManager::get();
    let result = session_manager
        .send(GetSession { session_id })
        .await;

    match result {
        Ok(Some(session)) => HttpResponse::Ok().json(SessionResponse {
            session_id: session.id,
            user_id: session.user_id,
            expires_at: session.expires_at,
        }),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Session not found"
        })),
        Err(e) => {
            tracing::error!("Actor mailbox error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Service unavailable"
            }))
        }
    }
}
