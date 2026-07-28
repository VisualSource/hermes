use actix_web::{HttpResponse, Responder, delete, get, patch, post, put, web};
use actix_web_validation::Validated;
use serde::Deserialize;
use sqlx::{SqlitePool, query, query_as};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::roles::Role,
    state::{api_errors::ApplicationError, oauth::jwt::Claims},
};

#[derive(Debug, Deserialize, ToSchema, Validate)]
struct CreateRolePayload {
    #[validate(length(min = 3, max = 255), non_control_character)]
    name: String,

    // #123456
    #[validate(length(equal = 6))]
    fg_color: Option<String>,

    // #123456
    #[validate(length(equal = 6))]
    bg_color: Option<String>,
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "create a role",
    request_body = CreateRolePayload,
    responses(
        (status = 200, description = "role", body = Role),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[post("/server/{server}/role")]
pub async fn create_role(
    db: web::Data<SqlitePool>,
    params: web::Path<Uuid>,
    Validated(web::Json(body)): Validated<web::Json<CreateRolePayload>>,
    user: web::ReqData<Claims>,
) -> Result<web::Json<Role>, ApplicationError> {
    //TODO: validate user can create role

    let server_id = params.into_inner();
    let id = Uuid::now_v7();

    let mask = 0;

    let role = query_as!(
        Role,
        "INSERT INTO roles VALUES (?,?,?,?,?,?) RETURNING *",
        id,
        server_id,
        body.name,
        body.fg_color,
        body.bg_color,
        mask
    )
    .fetch_one(db.get_ref())
    .await?;

    Ok(web::Json(role))
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "delete role",
    responses(
        (status = 201, description = "role"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[delete("/server/{server}/role/{role}")]
pub async fn delete_role(
    db: web::Data<SqlitePool>,
    params: web::Path<(Uuid, Uuid)>,
    user: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    //TODO: validate user can delete role
    let (server_id, role_id) = params.into_inner();

    query!(
        "DELETE FROM roles WHERE id = ? AND server_id = ?",
        &role_id,
        server_id
    )
    .execute(db.get_ref())
    .await?;

    Ok(HttpResponse::Accepted().finish())
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
struct PatchRolePayload {
    #[validate(length(min = 3, max = 255), non_control_character)]
    name: Option<String>,
    #[validate(length(equal = 6))]
    bg_color: Option<String>,
    #[validate(length(equal = 6))]
    fg_color: Option<String>,
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "update a role",
    request_body = PatchRolePayload,
    responses(
        (status = 201, description = "role"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[patch("/server/{server}/role/{role}")]
pub async fn patch_role(
    db: web::Data<SqlitePool>,
    params: web::Path<(Uuid, Uuid)>,
    Validated(web::Json(body)): Validated<web::Json<PatchRolePayload>>,
    user: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    //TODO: validate user can update this role
    let (server_id, role_id) = params.into_inner();

    Ok(HttpResponse::Accepted().finish())
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "get role",
    responses(
        (status = 200, description = "role", body = Role),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/server/{server}/role/{role}")]
pub async fn get_role(
    db: web::Data<SqlitePool>,
    params: web::Path<(Uuid, Uuid)>,
    user: web::ReqData<Claims>,
) -> Result<web::Json<Role>, ApplicationError> {
    //TODO: validate user
    let (server_id, role_id) = params.into_inner();

    let role = query_as!(
        Role,
        "SELECT * FROM roles WHERE id = ? AND server_id = ?",
        role_id,
        &server_id
    )
    .fetch_one(db.get_ref())
    .await?;

    Ok(web::Json(role))
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
struct RoleQuery {
    role_id: Uuid,
    target: Uuid,
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "add role to user",
    request_body = RoleQuery,
    responses(
        (status = 201, description = "adding accepted"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[put("/server/{server}/role")]
pub async fn add_role_to_user(
    db: web::Data<SqlitePool>,
    params: web::Path<Uuid>,
    Validated(web::Json(body)): Validated<web::Json<RoleQuery>>,
    user: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    //TODO: validate user can add role
    //TODO: validate role is validate for server

    query!(
        "INSERT INTO role_members VALUES (?,?)",
        body.role_id,
        body.target
    )
    .execute(db.get_ref())
    .await?;

    Ok(HttpResponse::Accepted().finish())
}

#[utoipa::path(
    tags = ["server","role"], 
    description = "remove role from user",
    request_body = CreateRolePayload,
    responses(
        (status = 201, description = "remove accepted"),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[delete("/server/{server}/role")]
pub async fn remove_role_from_user(
    db: web::Data<SqlitePool>,
    params: web::Path<Uuid>,
    Validated(web::Json(body)): Validated<web::Json<RoleQuery>>,
    user: web::ReqData<Claims>,
) -> Result<impl Responder, ApplicationError> {
    //TODO: validate user can remove role
    //TODO: validate role is in server

    query!(
        "DELETE FROM role_members WHERE role_id = ? AND member_id = ?",
        body.role_id,
        body.target
    )
    .execute(db.get_ref())
    .await?;

    Ok(HttpResponse::Accepted().finish())
}
