use uuid::Uuid;

pub const PERM_WRITE: u64 = 1;
pub const PERM_READ: u64 = 2;
pub const PERM_DELETE: u64 = 4;
pub const PERM_CREATE: u64 = 8;

pub const PERM_CHANNEL: u64 = 16;
pub const PERM_MESSAGE: u64 = 32;
pub const PERM_INVITE: u64 = 64;
pub const PERM_ROLE: u64 = 128;
pub const PERM_SERVER: u64 = 256;

pub async fn has_permissions(
    user: Uuid,
    server: Uuid,
    permissions: u64,
) -> Result<bool, sqlx::Error> {
    // fetch user permissions on server

    // validate user can do this

    return Ok(false);
}
