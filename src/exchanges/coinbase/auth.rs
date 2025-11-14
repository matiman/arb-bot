//! Coinbase JWT Authentication
//!
//! Implements JWT token generation for Coinbase App API using ES256 algorithm.
//!
//! Based on: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/authentication
//!
//! Key differences from CDP API:
//! - `iss` claim must be "cdp" (not "coinbase-cloud")
//! - JWT must include `uri` claim in payload: "{method} {host}{path}"
//! - JWT header must include `kid` (key ID) and `nonce` (random hex string)

use crate::error::{ArbitrageError, Result};
use base64::engine::Engine;
use chrono::{Duration, Utc};
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use rand::RngCore;
use sec1::DecodeEcPrivateKey;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// JWT claims for Coinbase App API
#[derive(Debug, Serialize, Deserialize)]
struct CoinbaseClaims {
    sub: String, // API key name (e.g., "organizations/org-id/apiKeys/key-id")
    iss: String, // "cdp" (not "coinbase-cloud"!)
    nbf: i64,    // Not before timestamp
    exp: i64,    // Expiration timestamp (2 minutes from nbf)
    uri: String, // Request URI: "{method} {host}{path}" (e.g., "GET api.coinbase.com/api/v3/brokerage/accounts")
}

/// Coinbase JWT authentication handler
pub struct CoinbaseAuth {
    api_key: String,     // Full API key path
    private_key: String, // EC private key in PEM format
}

impl CoinbaseAuth {
    /// Create a new CoinbaseAuth instance
    ///
    /// # Arguments
    /// * `api_key` - Full API key path (e.g., "organizations/org-id/apiKeys/key-id")
    /// * `api_secret` - EC private key in PEM format
    ///
    /// # Returns
    /// Result containing CoinbaseAuth or AuthenticationError if key is invalid
    pub fn new(api_key: String, api_secret: String) -> Result<Self> {
        // Basic validation: check if api_secret looks like a PEM key
        if !api_secret.contains("BEGIN EC PRIVATE KEY") {
            return Err(ArbitrageError::AuthenticationError {
                exchange: "coinbase".to_string(),
                reason: "Invalid private key format. Expected PEM-encoded EC private key."
                    .to_string(),
            });
        }

        Ok(Self {
            api_key,
            private_key: api_secret,
        })
    }

    /// Generate a JWT token for Coinbase App API
    ///
    /// # Arguments
    /// * `method` - HTTP method (e.g., "GET", "POST")
    /// * `host` - API host (e.g., "api.coinbase.com")
    /// * `path` - API path (e.g., "/api/v3/brokerage/accounts")
    ///
    /// # Returns
    /// JWT token string
    ///
    /// # Errors
    /// Returns AuthenticationError if key parsing or JWT generation fails
    pub fn generate_jwt(&self, method: &str, host: &str, path: &str) -> Result<String> {
        let now = Utc::now();

        // Build URI claim: "{method} {host}{path}"
        let uri = format!("{} {}{}", method, host, path);

        // Build JWT claims
        let claims = CoinbaseClaims {
            sub: self.api_key.clone(),
            iss: "cdp".to_string(), // Must be "cdp" for Coinbase App API
            nbf: now.timestamp(),
            exp: (now + Duration::minutes(2)).timestamp(), // 2 minutes expiration
            uri,
        };

        // Generate random nonce (32 hex characters = 16 bytes)
        let mut rng = rand::thread_rng();
        let mut nonce_bytes = [0u8; 16];
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = hex::encode(nonce_bytes);

        // Build JWT header with custom fields (kid and nonce)
        let header = json!({
            "alg": "ES256",
            "typ": "JWT",
            "kid": self.api_key,
            "nonce": nonce
        });

        // Parse PEM-encoded EC private key (SEC1 format from Coinbase)
        // Convert literal \n to actual newlines if needed
        let key_str = if self.private_key.contains("\\n") {
            self.private_key.replace("\\n", "\n")
        } else {
            self.private_key.clone()
        };

        // Coinbase provides SEC1 format keys
        // Parse SEC1 format
        let signing_key = SigningKey::from_sec1_pem(&key_str).map_err(|e| {
            ArbitrageError::AuthenticationError {
                exchange: "coinbase".to_string(),
                reason: format!("Failed to parse SEC1 EC private key: {}", e),
            }
        })?;

        // Manually encode JWT: header.payload.signature
        // 1. Encode header
        let header_json =
            serde_json::to_string(&header).map_err(|e| ArbitrageError::AuthenticationError {
                exchange: "coinbase".to_string(),
                reason: format!("Failed to serialize JWT header: {}", e),
            })?;
        let header_b64 =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header_json.as_bytes());

        // 2. Encode payload
        let claims_json =
            serde_json::to_string(&claims).map_err(|e| ArbitrageError::AuthenticationError {
                exchange: "coinbase".to_string(),
                reason: format!("Failed to serialize JWT claims: {}", e),
            })?;
        let payload_b64 =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims_json.as_bytes());

        // 3. Create message to sign: header.payload
        let message = format!("{}.{}", header_b64, payload_b64);

        // 4. Sign with ES256 (ECDSA P-256)
        let signature: Signature = signing_key.sign(message.as_bytes());
        let signature_bytes = signature.to_bytes();

        // 5. Encode signature (DER format, but we need raw r||s format for JWT)
        // ES256 signature is 64 bytes (32 bytes r + 32 bytes s)
        let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(signature_bytes);

        // 6. Combine: header.payload.signature
        Ok(format!("{}.{}.{}", header_b64, payload_b64, sig_b64))
    }
}
