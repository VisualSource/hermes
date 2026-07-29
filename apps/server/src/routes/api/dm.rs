use actix_web::{get, web};
use sqlx::{SqlitePool, query_as};

use crate::{
    models::channel::DmParticipant,
    state::{api_errors::ApplicationError, oauth::jwt::Claims},
};

#[utoipa::path(
    tags = ["dm","channel"],
     responses(
        (status = 200, description = "new channel", body = Vec<DmParticipant>),
        (status = 401, description = "unauthorized", body = ApplicationError),
        (status = 429, description = "too many request", body = ApplicationError),
        (status = 500, description = "internal server error", body = ApplicationError)
    )
)]
#[get("/dm-channels")]
pub async fn get_dms(
    db: web::Data<SqlitePool>,
    user: web::ReqData<Claims>,
) -> Result<web::Json<Vec<DmParticipant>>, ApplicationError> {
    let channels = query_as!(
        DmParticipant,
        "SELECT * FROM dm_participants WHERE user_a_id = ? OR user_b_id = ?",
        user.sub,
        user.sub
    )
    .fetch_all(db.get_ref())
    .await?;

    Ok(web::Json(channels))
}
