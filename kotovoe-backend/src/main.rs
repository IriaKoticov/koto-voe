use axum::{
    routing::{get, post, delete},
    extract::{Json, Path, State},
    response::IntoResponse,
    http::StatusCode,
    http::Method,
    Router,
};

use axum_extra::extract::cookie::{Cookie, CookieJar};
use axum_extra::TypedHeader;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE, COOKIE};
use tower_http::cors::AllowOrigin;



use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool , postgres::PgRow,Row};
use std::{net::SocketAddr, sync::Arc};
use chrono::{NaiveDateTime};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use tower_http::cors::{CorsLayer, Any};

use time::{Duration, OffsetDateTime};

use tracing_subscriber::EnvFilter;
use tracing::{info, warn, error, debug};
use tower_http::trace::TraceLayer;

use jsonwebtoken::{encode,decode as dcjwt, Header, EncodingKey, DecodingKey, Validation};


use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

#[derive(Clone)]
struct AppState {
    db: PgPool,
    pub jwt_secret: String,
}


#[derive(Serialize)]
struct CatImg {
    id:i32,
    image: String,
    usr_id: i32,
    data:NaiveDateTime,

}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Usr {
    id: i32,
    email: String,
    pas_hash: String,
    last_log_in: Option<NaiveDateTime>,
    date_cr: NaiveDateTime,
    login: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Cat {
    id: i32,
    image: Vec<u8>,  // BYTEA
    maker_id: i32,
    date_cr: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Swipe {
    id: i32,
    usr_id: i32,
    cat_id: i32,
    date_swipe: NaiveDateTime,
    swipe_chose: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct AlbumUsr {
    image: Vec<u8>,
    id: i32, // между 1 и 10
}
#[derive(Debug, Serialize, Deserialize)]

struct AlbumUsrStr {
    image: String,
    id: i32, // между 1 и 10
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Log {
    id: i32,
    usr_id: i32,
    action: String,
    ts: NaiveDateTime,
}

// Вход для регистрации пользователя
#[derive(Debug, Deserialize)]
struct UserInput {
    login: String,
    email: String,
    pas_hash: String,
}

// Для создания котика (будем принимать base64 закодированное изображение)
#[derive(Debug, Deserialize)]
struct CatInput {
    image_base64: String,
    maker_id: i32,
}

// Для создания свайпа
#[derive(Debug, Deserialize)]
struct SwipeInput {
    usr_id: i32,
    cat_id: i32,
    swipe_chose: bool,
}

// Для добавления в альбом и оценки
#[derive(Debug, Deserialize)]
struct AlbumUsrInput {
    id_usr: i32,
    id_cat: i32,
    usr_grade: i32,
}

// Лог - просто сообщение
#[derive(Debug, Deserialize)]
struct LogInput {
    usr_id: i32,
    action: String,
}

#[derive(Debug, Deserialize, Serialize,FromRow)]
struct CatGradeAll {
    image: Vec<u8>,
    grade_avg: f64,
}

#[derive(Debug, Deserialize,Serialize)]
struct CatGradeAll_to_front {
    image: String,
    grade_avg: f64,
}



#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i32,              
    exp: i64,              
}


// ---Доп Функции ---


fn generate_jwt(user_id: i32, secret: &str) -> String {
    let expiration = OffsetDateTime::now_utc() + Duration::minutes(15); // токен на 1 день

    let claims = Claims {
        sub: user_id,
        exp: expiration.unix_timestamp(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .expect("Token creation failed")
}

fn verify_token(token: &str, secret: &str) -> Option<i32> {
    let data = dcjwt::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ).ok()?;

    Some(data.claims.sub) // user id
}


// --- Эндпоинты ---

// Регистрация пользователя
async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserInput>,
) -> impl IntoResponse {

    let salt = SaltString::generate(&mut OsRng);


    let argon2 = Argon2::default();
    let hash = argon2.hash_password(payload.pas_hash.as_bytes(), &salt)
        .expect("Failed to hash password").to_string();

    let result = sqlx::query!(
        "INSERT INTO usr (email, pas_hash, login) VALUES ($1, $2, $3) RETURNING id",
        payload.email,
        hash,
        payload.login,
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(rec) => (StatusCode::CREATED, Json(rec.id)).into_response(),
        Err(e) => {
            eprintln!("Error inserting user: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user").into_response()
        }
    }
}


#[derive(Serialize)]
struct LoginResponse {
    token: String,
    user_id: i32,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    usr_id: i32,
}

async fn check_user_exists(
    jar: CookieJar,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserInput>,
) -> (CookieJar, impl IntoResponse) {
    
    let result = sqlx::query_as::<_, Usr>("SELECT * FROM usr WHERE login = $1")
        .bind(payload.login)
        .fetch_optional(&state.db)
        .await;


    match result {
        Ok(Some(user)) => {

            let parsed_hash = PasswordHash::new(&user.pas_hash).unwrap();

            if Argon2::default().verify_password(payload.pas_hash.as_bytes(), &parsed_hash).is_ok() {
                
                let _ = sqlx::query!(
                    "UPDATE usr SET last_log_in = now() WHERE id = $1",
                    user.id
                )
                .execute(&state.db)
                .await;


                let token = generate_jwt(user.id, &state.jwt_secret);

                let cookie = Cookie::build(("auth_token",token))
                                            .path("/")
                                            .secure(false)
                                            .http_only(true);


                (jar.add(cookie),
                (StatusCode::OK, Json(AuthResponse { usr_id: user.id })).into_response())
                

            } else {
                (jar,
                (StatusCode::UNAUTHORIZED, Json("Invalid password".to_string())).into_response())
            }
        }
        Ok(None) => {(jar,(StatusCode::NOT_FOUND, "User not found").into_response())},
        Err(e) => {
            eprintln!("DB error in auth_user: {:?}", e);
            (jar,
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())
}
    }
} 

#[derive(Debug,Clone,Serialize,Deserialize)]
struct UsrIdToJWT{
    user_id:i32
}

async fn protected_route(jar: CookieJar, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if let Some(token) = jar.get("auth_token") {
        if let Some(user_id) = verify_token(token.value(), &state.jwt_secret) {
            return (StatusCode::OK, Json(UsrIdToJWT{user_id})).into_response();
        }
    }

    (StatusCode::UNAUTHORIZED, "NotJWTToken").into_response()
}


// Получить пользователя по email
async fn get_user_by_email(
    State(state): State<Arc<AppState>>,
    Path(email): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Usr>("SELECT * FROM usr WHERE email = $1")
        .bind(email)
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DB error").into_response(),
    }
}

// Добавить котика
async fn create_cat(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CatInput>,
) -> impl IntoResponse {
    let image_bytes = match base64::decode(&payload.image_base64) {
        Ok(bytes) => bytes,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid base64 image").into_response(),
    };

    let result = sqlx::query!(
        "INSERT INTO cat (image, maker_id) VALUES ($1, $2) RETURNING id",
        &image_bytes,
        payload.maker_id
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(rec) => (StatusCode::CREATED, Json(rec.id)).into_response(),
        Err(e) => {
            eprintln!("Error inserting cat: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create cat").into_response()
        }
    }
}

// Создать свайп (like/dislike)
async fn create_swipe(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SwipeInput>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "INSERT INTO swipes (usr_id, cat_id, swipe_chose) VALUES ($1, $2, $3) RETURNING id",
        payload.usr_id,
        payload.cat_id,
        payload.swipe_chose
    )
    .fetch_one(&state.db)
    .await;




    match result {
        Ok(rec) => (StatusCode::CREATED, Json(rec.id)).into_response(),
        Err(e) => {
            eprintln!("Error inserting swipe: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create swipe").into_response()
        }
    }
}

// Добавить кота в альбом с оценкой
async fn add_album_entry(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AlbumUsrInput>,
) -> impl IntoResponse {
    if payload.usr_grade < 1 || payload.usr_grade > 10 {
        return (StatusCode::BAD_REQUEST, "usr_grade must be between 1 and 10").into_response();
    }

    // Проверяем, есть ли запись
    let existing = sqlx::query(
        "SELECT usr_grade FROM album_usr WHERE id_usr = $1 AND id_cat = $2"
    )
    .bind(payload.id_usr)
    .bind(payload.id_cat)
    .fetch_optional(&state.db)
    .await;

    let result: Result<PgRow, sqlx::Error> = match existing {
        Ok(Some(_row)) => {
            // Есть запись — увеличиваем usr_grade на 1 и возвращаем id
            tracing::info!("Cat updating");
            sqlx::query(
                "UPDATE album_usr SET usr_grade = usr_grade + 1 WHERE id_usr = $1 AND id_cat = $2 RETURNING id"
            )
            .bind(payload.id_usr)
            .bind(payload.id_cat)
            .fetch_one(&state.db)
            .await
        }
        Ok(None) => {
            // Нет записи — вставляем новую с usr_grade = 5 и возвращаем id
            tracing::info!("New Cat adding");

            sqlx::query(
                "INSERT INTO album_usr (id_usr, id_cat, usr_grade) VALUES ($1, $2, 5) RETURNING id"
            )
            .bind(payload.id_usr)
            .bind(payload.id_cat)
            .fetch_one(&state.db)
            .await
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                tracing::error!("DB error: {}", e)
            ).into_response();
        }
    };

    // Обновляем cat_grade
    let _ = sqlx::query("SELECT update_cat_grade($1)")
        .bind(payload.id_cat)
        .execute(&state.db)
        .await;

    match result {
        Ok(row) => {
            let id: i32 = row.try_get("id").unwrap_or_default();
            (StatusCode::CREATED, Json(id)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to update album: {}", e)
        ).into_response(),
    }
}




// Добавить лог
async fn add_log(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LogInput>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "INSERT INTO logs (usr_id, action) VALUES ($1, $2) RETURNING id",
        payload.usr_id,
        payload.action
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(rec) => (StatusCode::CREATED, Json(rec.id)).into_response(),
        Err(e) => {
            eprintln!("Error inserting log: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add log").into_response()


        }
    }
}

// Получить все логи пользователя
async fn get_logs_for_user(
    State(state): State<Arc<AppState>>,
    Path(usr_id): Path<i32>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Log>("SELECT * FROM logs WHERE usr_id = $1 ORDER BY ts DESC")
        .bind(usr_id)
        .fetch_all(&state.db)
        .await;

    match result {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch logs").into_response(),
    }
}


async fn get_cat(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CatImg>, StatusCode> {
    let cat: Cat = sqlx::query_as("SELECT * FROM cat ORDER BY RANDOM() LIMIT 1")
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CatImg {
        id: cat.id,
        image:  format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(cat.image)),
        usr_id: cat.maker_id,
        data: cat.date_cr,
    }))
}

async fn get_top_cats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CatGradeAll_to_front>>, StatusCode> {
    
    let cats: Vec<CatGradeAll> = sqlx::query_as::<_, CatGradeAll>(
        "
        SELECT cat.image, cat_grade_all.grade_avg
        FROM cat_grade_all
        JOIN cat ON cat.id = cat_grade_all.cat_id
        ORDER BY cat_grade_all.grade_avg DESC
        LIMIT 3
        ",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| {
        eprintln!("Database error: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let rez: Vec<CatGradeAll_to_front> = cats
        .into_iter()
        .map(|c| CatGradeAll_to_front {
            image: format!("data:image/jpeg;base64,{}", STANDARD.encode(c.image)),
            grade_avg: c.grade_avg,
        })
        .collect();

    Ok(Json(rez))
}


async fn get_albums(
    Path(user_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AlbumUsrStr>>, StatusCode> {
    let albums: Vec<AlbumUsr> = sqlx::query_as::<_, AlbumUsr>(
        "SELECT cat.image, album_usr.id
            FROM album_usr
            JOIN cat ON cat.id = album_usr.id_cat
            WHERE album_usr.id_usr = $1
            ORDER BY album_usr.usr_grade DESC;"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("DB error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;


    let rez = albums
        .into_iter()
        .map(|a| AlbumUsrStr {
            image: format!("data:image/jpeg;base64,{}", STANDARD.encode(&a.image)),
            id: a.id,
        })
        .collect();


    Ok(Json(rez))
}

// --- delete resp ---

async fn delete_album_entry(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "DELETE FROM album_usr WHERE id = $1",
        id
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (StatusCode::OK, "Deleted successfully").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete").into_response(),
    }
}


// --- main ---

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()) // читается из RUST_LOG
        .init();

    dotenv::dotenv().ok(); // грузим .env с DATABASE_URL

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    let state = Arc::new(AppState { db: pool, jwt_secret:jwt_secret });

    let cors = CorsLayer::new()
        .allow_origin("http://127.0.0.1:8080".parse::<axum::http::HeaderValue>().unwrap())
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, COOKIE])
        .allow_methods([Method::GET, Method::POST, Method::DELETE]);

    let app = Router::new()
        .route("/user", post(create_user))
        .route("/auth", post(check_user_exists))
        .route("/protect", get(protected_route))
        .route("/user/{email}", get(get_user_by_email))
        .route("/cat", post(create_cat))
        .route("/api/cat", get(get_cat))
        .route("/tops", get(get_top_cats))
        .route("/swipe", post(create_swipe))
        .route("/album/{user_id}", get(get_albums))
        .route("/album/d/{id}", delete(delete_album_entry))
        .route("/album/", post(add_album_entry))
        .route("/log", post(add_log))
        .route("/logs/{usr_id}", get(get_logs_for_user))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    tracing::info!("Приложение запущено");

    tracing::info!("Listening on http://{}", addr);
    axum::serve(listener,app.into_make_service())
        .await
        .unwrap();
}
