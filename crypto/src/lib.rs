use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    PasswordHash, PasswordHasher, PasswordVerifier,
};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine as _};
use rand::Rng;

pub fn hash_password(password: &[u8]) -> String {
    let argon2 = argon2::Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    argon2
        .hash_password(password, salt.as_salt())
        .unwrap()
        .to_string()
}

pub fn verify_password(
    password: &[u8],
    hashed_password: &str,
) -> argon2::password_hash::Result<()> {
    let argon2 = argon2::Argon2::default();
    let hash = PasswordHash::new(hashed_password).expect("Invalid password hash.");

    argon2.verify_password(password, &hash)
}

pub fn generate_token() -> String {
    let mut bytes = [0u8; 64];
    rand::thread_rng().fill(&mut bytes);
    BASE64_URL_SAFE_NO_PAD.encode(&bytes)
}
