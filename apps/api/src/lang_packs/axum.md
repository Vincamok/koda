# Axum Framework Pack

You are working with **Axum 0.7** (Rust web framework built on Tower and Hyper).

## Core patterns

```rust
// Router setup
let app = Router::new()
    .route("/path", get(handler).post(other_handler))
    .with_state(state);

// Handler signature
async fn handler(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateRequest>,
) -> Result<impl IntoResponse, AppError> { ... }
```

## State injection
- Use `#[derive(Clone)]` on `AppState` — it is shared across all handlers
- Prefer `State<AppState>` over `Extension` for app-level state
- Use `Extension` for per-request middleware-injected data (auth, org context)

## Error handling
- Implement `IntoResponse` for your error type
- Use `AppError::NotFound`, `AppError::Forbidden`, `AppError::BadRequest`
- Map `sqlx::Error` → `AppError::Internal` via `?`

## Middleware
- `from_fn` for simple middlewares
- `from_fn_with_state` when the middleware needs `AppState`
- Layer order: outermost applied last (`.layer()` calls stack bottom-up)

## SSE (Server-Sent Events)
```rust
use axum::response::sse::{Event, KeepAlive, Sse};
// Return Sse<impl Stream<Item = Result<Event, Infallible>>>
```

## WebSocket
```rust
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket))
}
```

## Best practices
- Always validate with `validator` crate before touching DB
- Filter by `organization_id` on every DB query (multi-tenant invariant)
- Use `sqlx::query!` macro for compile-time SQL checks
- Return `(StatusCode, Json(...))` for status codes other than 200
