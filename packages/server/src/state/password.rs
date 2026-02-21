use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

pub fn hash_password(psd: &str) -> Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let hash = argon2.hash_password(psd.as_bytes(), &salt)?;

    Ok(hash.to_string())
}

pub fn verify_password(psd: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let argon2 = Argon2::default();

    let psd_hash = PasswordHash::new(hash)?;

    let result = argon2.verify_password(psd.as_bytes(), &psd_hash);

    Ok(!result.is_err())
}
