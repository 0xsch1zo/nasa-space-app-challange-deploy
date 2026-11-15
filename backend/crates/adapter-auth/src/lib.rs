//! Security adapters: JWT (HS256) token service and Argon2 password hashing.

use application::error::{AppError, InfraError};
use application::ports::outbound::{InfraResult, IssuedToken, PasswordHasher, TokenService};
use application::security::Principal;
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher as _, PasswordVerifier};
use chrono::{Duration, Utc};
use domain::crew::{AstronautId, Role, StationId, UserId};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    name: String,
    preferred_username: String,
    role: String,
    station_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    astronaut_id: Option<Uuid>,
    iat: i64,
    exp: i64,
    iss: String,
}

const ISSUER: &str = "astro-health";

pub struct JwtTokenService {
    encoding: EncodingKey,
    decoding: DecodingKey,
    ttl: Duration,
}

impl JwtTokenService {
    pub fn new(secret: &[u8], ttl: Duration) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
            ttl,
        }
    }
}

impl TokenService for JwtTokenService {
    fn issue(&self, p: &Principal) -> InfraResult<IssuedToken> {
        let now = Utc::now();
        let expires_at = now + self.ttl;
        let claims = Claims {
            sub: p.user_id.0,
            name: p.display_name.clone(),
            preferred_username: p.username.clone(),
            role: p.role.as_str().to_owned(),
            station_id: p.station_id.0,
            astronaut_id: p.astronaut_id.map(|a| a.0),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
            iss: ISSUER.to_owned(),
        };
        let token = encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)
            .map_err(InfraError::new)?;
        Ok(IssuedToken { token, expires_at })
    }

    fn verify(&self, token: &str) -> Result<Principal, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[ISSUER]);
        let claims = decode::<Claims>(token, &self.decoding, &validation)
            .map_err(|_| AppError::Unauthenticated)?
            .claims;
        let role = Role::parse(&claims.role).ok_or(AppError::Unauthenticated)?;
        // A token must be internally consistent with the role model.
        if (role == Role::Astronaut) != claims.astronaut_id.is_some() {
            return Err(AppError::Unauthenticated);
        }
        Ok(Principal {
            user_id: UserId(claims.sub),
            username: claims.preferred_username,
            display_name: claims.name,
            role,
            station_id: StationId(claims.station_id),
            astronaut_id: claims.astronaut_id.map(AstronautId),
        })
    }
}

#[derive(Default)]
pub struct Argon2Hasher;

impl PasswordHasher for Argon2Hasher {
    fn hash(&self, plain: &str) -> InfraResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(plain.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(InfraError::new)
    }

    fn verify(&self, plain: &str, hash: &str) -> bool {
        PasswordHash::new(hash)
            .map(|parsed| {
                Argon2::default()
                    .verify_password(plain.as_bytes(), &parsed)
                    .is_ok()
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn principal(role: Role) -> Principal {
        Principal {
            user_id: UserId::new(),
            username: "u".into(),
            display_name: "U".into(),
            role,
            station_id: StationId::new(),
            astronaut_id: (role == Role::Astronaut).then(AstronautId::new),
        }
    }

    #[test]
    fn token_roundtrip_and_tamper_detection() {
        let svc = JwtTokenService::new(b"secret", Duration::minutes(5));
        let p = principal(Role::Astronaut);
        let t = svc.issue(&p).unwrap();
        assert_eq!(svc.verify(&t.token).unwrap(), p);

        let other = JwtTokenService::new(b"other-secret", Duration::minutes(5));
        assert!(other.verify(&t.token).is_err());
    }

    #[test]
    fn expired_token_is_rejected() {
        let svc = JwtTokenService::new(b"secret", Duration::minutes(-10));
        let t = svc.issue(&principal(Role::Supervisor)).unwrap();
        assert!(svc.verify(&t.token).is_err());
    }

    #[test]
    fn argon2_roundtrip() {
        let h = Argon2Hasher.hash("pw").unwrap();
        assert!(Argon2Hasher.verify("pw", &h));
        assert!(!Argon2Hasher.verify("nope", &h));
    }
}
