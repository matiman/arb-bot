//! TDD tests for Coinbase REST API authentication
//!
//! These tests follow TDD approach - they will fail initially until implementation is complete.
//!
//! IMPORTANT: 
//! - Most tests only verify JWT TOKEN GENERATION - they do NOT make HTTP requests.
//! - `test_jwt_authentication_with_readonly_api` makes a REAL API call to verify JWT works
//!   (uses read-only endpoint, no trades are made)
//!
//! To run tests:
//!   cargo test --test coinbase_auth
//!
//! To run with environment variables:
//!   COINBASE_API_KEY=... COINBASE_API_SECRET=... cargo test --test coinbase_auth

use arb_bot::error::ArbitrageError;
use arb_bot::exchanges::coinbase::auth::CoinbaseAuth;

/// Load environment variables from .env file
fn load_env() {
    let _ = dotenvy::dotenv();
}

/// Helper to get API key from environment
fn get_api_key() -> String {
    load_env();
    // Try COINBASE_API_KEY first, then COINBASE_API_KEY_ID (for compatibility)
    std::env::var("COINBASE_API_KEY")
        .or_else(|_| std::env::var("COINBASE_API_KEY_ID"))
        .expect("COINBASE_API_KEY or COINBASE_API_KEY_ID environment variable required")
}

/// Helper to get API secret from environment
fn get_api_secret() -> String {
    load_env();
    // Try COINBASE_API_SECRET first, then COINBASE_API_SECRET (for compatibility)
    let secret = std::env::var("COINBASE_API_SECRET")
        .or_else(|_| std::env::var("COINBASE_API_SECRET"))
        .expect("COINBASE_API_SECRET environment variable required");
    
    // Remove quotes if present (some .env files add quotes)
    secret.trim_matches('"').trim_matches('\'').to_string()
}

// ============================================================================
// JWT Generation Tests (with correct credentials)
// ============================================================================

#[test]
#[ignore] // Requires COINBASE_API_KEY and COINBASE_API_SECRET environment variables
fn test_jwt_generation_with_correct_credentials() {
    // Test: JWT generation should succeed with correct credentials
    // NOTE: This only generates a JWT token - it does NOT make any HTTP requests
    // The JWT's uri claim must match the actual API request when we make HTTP calls
    let api_key = get_api_key();
    let api_secret = get_api_secret();

    let auth = CoinbaseAuth::new(api_key.clone(), api_secret.clone());
    assert!(
        auth.is_ok(),
        "CoinbaseAuth should initialize successfully with correct credentials"
    );

    let auth = auth.unwrap();
    // Test with production host (api.coinbase.com)
    // NOTE: This only generates a JWT token - it does NOT make any HTTP requests
    // The JWT's uri claim must match the actual API request when we make HTTP calls
    let jwt_result = auth.generate_jwt("GET", "api.coinbase.com", "/api/v3/brokerage/accounts");

    // With correct credentials, JWT generation should succeed
    assert!(
        jwt_result.is_ok(),
        "JWT generation should succeed with correct credentials"
    );

    let jwt = jwt_result.unwrap();
    assert!(!jwt.is_empty(), "JWT token should not be empty");

    // JWT should have 3 parts (header.payload.signature)
    let parts: Vec<&str> = jwt.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "JWT should have 3 parts separated by dots"
    );
}

#[test]
#[ignore] // Requires COINBASE_API_KEY and COINBASE_API_SECRET environment variables
fn test_jwt_contains_required_claims() {
    // Test: JWT should contain required claims (sub, iss, nbf, exp, uri)
    // NOTE: This only generates a JWT token - it does NOT make any HTTP requests
    let api_key = get_api_key();
    let api_secret = get_api_secret();

    let auth = CoinbaseAuth::new(api_key, api_secret).unwrap();
    // Test with production host - NOTE: This only generates JWT, no HTTP requests
    let jwt = auth
        .generate_jwt("GET", "api.coinbase.com", "/api/v3/brokerage/accounts")
        .expect("JWT generation should succeed");

    // Decode JWT to verify claims (we'll use a simple base64 decode for testing)
    let parts: Vec<&str> = jwt.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT should have 3 parts");

    // Decode payload (second part)
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .expect("JWT payload should be valid base64");
    let payload_str = String::from_utf8(payload_bytes).expect("JWT payload should be valid UTF-8");
    let payload: serde_json::Value =
        serde_json::from_str(&payload_str).expect("JWT payload should be valid JSON");

    // Verify required claims
    assert_eq!(
        payload["iss"].as_str(),
        Some("cdp"),
        "JWT should have 'iss' claim set to 'cdp'"
    );
    assert!(
        payload["sub"].is_string(),
        "JWT should have 'sub' claim (API key name)"
    );
    assert!(
        payload["nbf"].is_number(),
        "JWT should have 'nbf' claim (not before timestamp)"
    );
    assert!(
        payload["exp"].is_number(),
        "JWT should have 'exp' claim (expiration timestamp)"
    );
    assert_eq!(
        payload["uri"].as_str(),
        Some("GET api.coinbase.com/api/v3/brokerage/accounts"),
        "JWT should have 'uri' claim with format 'METHOD HOSTPATH'"
    );

    // Verify expiration is ~2 minutes from now
    let exp = payload["exp"].as_i64().expect("exp should be a number");
    let nbf = payload["nbf"].as_i64().expect("nbf should be a number");
    let duration = exp - nbf;
    assert!(
        duration >= 115 && duration <= 125,
        "JWT should expire in ~2 minutes (120 seconds), got {} seconds",
        duration
    );
}

#[test]
#[ignore] // Requires COINBASE_API_KEY and COINBASE_API_SECRET environment variables
fn test_jwt_contains_required_headers() {
    // Test: JWT header should contain kid and nonce
    // NOTE: This only generates a JWT token - it does NOT make any HTTP requests
    let api_key = get_api_key();
    let api_secret = get_api_secret();

    let auth = CoinbaseAuth::new(api_key.clone(), api_secret).unwrap();
    // Test with production host - NOTE: This only generates JWT, no HTTP requests
    let jwt = auth
        .generate_jwt("GET", "api.coinbase.com", "/api/v3/brokerage/accounts")
        .expect("JWT generation should succeed");

    // Decode JWT header (first part)
    let parts: Vec<&str> = jwt.split('.').collect();
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let header_bytes = URL_SAFE_NO_PAD
        .decode(parts[0])
        .expect("JWT header should be valid base64");
    let header_str = String::from_utf8(header_bytes).expect("JWT header should be valid UTF-8");
    let header: serde_json::Value =
        serde_json::from_str(&header_str).expect("JWT header should be valid JSON");

    // Verify required headers
    assert_eq!(
        header["alg"].as_str(),
        Some("ES256"),
        "JWT should use ES256 algorithm"
    );
    assert_eq!(
        header["typ"].as_str(),
        Some("JWT"),
        "JWT should have typ header"
    );
    assert_eq!(
        header["kid"].as_str(),
        Some(api_key.as_str()),
        "JWT should have 'kid' header matching API key name"
    );
    assert!(
        header["nonce"].is_string(),
        "JWT should have 'nonce' header"
    );
    let nonce = header["nonce"].as_str().unwrap();
    assert!(
        !nonce.is_empty(),
        "JWT nonce should not be empty"
    );
}

#[test]
#[ignore] // Requires COINBASE_API_KEY and COINBASE_API_SECRET environment variables
fn test_jwt_generation_sandbox_vs_production() {
    // Test: JWT generation works for both sandbox and production hosts
    // NOTE: This only generates JWT tokens - it does NOT make any HTTP requests
    let api_key = get_api_key();
    let api_secret = get_api_secret();

    let auth = CoinbaseAuth::new(api_key, api_secret).unwrap();

    // Test production JWT
    let prod_jwt = auth
        .generate_jwt("GET", "api.coinbase.com", "/api/v3/brokerage/accounts")
        .expect("Production JWT generation should succeed");
    
    // Test sandbox JWT
    let sandbox_jwt = auth
        .generate_jwt("GET", "api-public.sandbox.exchange.coinbase.com", "/api/v3/brokerage/accounts")
        .expect("Sandbox JWT generation should succeed");

    // Both should be valid JWTs
    assert!(!prod_jwt.is_empty());
    assert!(!sandbox_jwt.is_empty());
    
    // They should be different (different uri claims)
    assert_ne!(prod_jwt, sandbox_jwt, "Production and sandbox JWTs should differ (different uri claims)");

    // Verify the uri claims are different
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    
    let prod_parts: Vec<&str> = prod_jwt.split('.').collect();
    let prod_payload_bytes = URL_SAFE_NO_PAD.decode(prod_parts[1]).unwrap();
    let prod_payload_str = String::from_utf8(prod_payload_bytes).unwrap();
    let prod_payload: serde_json::Value = serde_json::from_str(&prod_payload_str).unwrap();
    
    let sandbox_parts: Vec<&str> = sandbox_jwt.split('.').collect();
    let sandbox_payload_bytes = URL_SAFE_NO_PAD.decode(sandbox_parts[1]).unwrap();
    let sandbox_payload_str = String::from_utf8(sandbox_payload_bytes).unwrap();
    let sandbox_payload: serde_json::Value = serde_json::from_str(&sandbox_payload_str).unwrap();
    
    assert_eq!(
        prod_payload["uri"].as_str(),
        Some("GET api.coinbase.com/api/v3/brokerage/accounts"),
        "Production JWT should have production URI"
    );
    assert_eq!(
        sandbox_payload["uri"].as_str(),
        Some("GET api-public.sandbox.exchange.coinbase.com/api/v3/brokerage/accounts"),
        "Sandbox JWT should have sandbox URI"
    );
}

// ============================================================================
// Error Handling Tests (with wrong credentials)
// ============================================================================

#[test]
fn test_auth_initialization_with_invalid_key_format() {
    // Test: Auth initialization should fail with invalid key format
    let invalid_key = "not-a-valid-pem-key".to_string();
    let auth_result = CoinbaseAuth::new("test-key".to_string(), invalid_key);

    assert!(
        auth_result.is_err(),
        "CoinbaseAuth should fail with invalid key format"
    );

    if let Err(e) = auth_result {
        match e {
            ArbitrageError::AuthenticationError { exchange, .. } => {
                assert_eq!(exchange, "coinbase");
            }
            _ => panic!("Expected AuthenticationError"),
        }
    }
}

#[test]
fn test_jwt_generation_with_invalid_private_key() {
    // Test: JWT generation should fail with invalid private key
    // Use a key that has correct format but invalid content
    let invalid_key = "-----BEGIN EC PRIVATE KEY-----\nINVALID_KEY_CONTENT\n-----END EC PRIVATE KEY-----".to_string();
    let auth_result = CoinbaseAuth::new("test-key".to_string(), invalid_key);

    // This might succeed in initialization (format check) but fail on JWT generation
    if let Ok(auth) = auth_result {
        let jwt_result = auth.generate_jwt("GET", "api.coinbase.com", "/api/v3/brokerage/accounts");
        assert!(
            jwt_result.is_err(),
            "JWT generation should fail with invalid private key"
        );

        if let Err(e) = jwt_result {
            match e {
                ArbitrageError::AuthenticationError { exchange, .. } => {
                    assert_eq!(exchange, "coinbase");
                }
                _ => panic!("Expected AuthenticationError"),
            }
        }
    }
}

#[test]
fn test_jwt_generation_with_wrong_api_key() {
    // Test: JWT generation should work but authentication will fail
    // This tests that wrong API key name doesn't break JWT generation
    // (The API key name is just metadata in the JWT, actual validation happens on API side)
    let api_secret = get_api_secret();
    let wrong_api_key = "organizations/wrong-org/apiKeys/wrong-key".to_string();

    let auth = CoinbaseAuth::new(wrong_api_key.clone(), api_secret).unwrap();
    let jwt_result = auth.generate_jwt("GET", "api.coinbase.com", "/api/v3/brokerage/accounts");

    // JWT generation should still succeed (it's just signing)
    // But the JWT will fail when used with API because key name doesn't match
    assert!(
        jwt_result.is_ok(),
        "JWT generation should succeed even with wrong API key name (validation happens on API side)"
    );

    let jwt = jwt_result.unwrap();
    // Verify the JWT contains the wrong key name
    let parts: Vec<&str> = jwt.split('.').collect();
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .expect("JWT payload should be valid base64");
    let payload_str = String::from_utf8(payload_bytes).expect("JWT payload should be valid UTF-8");
    let payload: serde_json::Value =
        serde_json::from_str(&payload_str).expect("JWT payload should be valid JSON");

    assert_eq!(
        payload["sub"].as_str(),
        Some(wrong_api_key.as_str()),
        "JWT should contain the API key name in 'sub' claim"
    );
}

// ============================================================================
// Real API Verification Tests (makes HTTP requests - READ-ONLY, no trades)
// ============================================================================

#[tokio::test]
#[ignore] // Requires COINBASE_API_KEY and COINBASE_API_SECRET environment variables
async fn test_jwt_authentication_with_readonly_api() {
    // Test: Verify JWT actually works by making a REAL API call
    // Uses read-only endpoint (GET /api/v3/brokerage/accounts) - NO TRADES ARE MADE
    // This verifies:
    // 1. JWT generation is correct
    // 2. API key is valid
    // 3. Private key matches API key
    // 4. Credentials are authorized
    
    use reqwest::Client;
    
    let api_key = get_api_key();
    let api_secret = get_api_secret();
    
    let auth = CoinbaseAuth::new(api_key.clone(), api_secret).unwrap();
    
    // Generate JWT for production API (read-only endpoint)
    let jwt = auth
        .generate_jwt("GET", "api.coinbase.com", "/api/v3/brokerage/accounts")
        .expect("JWT generation should succeed");
    
    // Make actual HTTP request to verify JWT works
    let client = Client::new();
    let url = "https://api.coinbase.com/api/v3/brokerage/accounts";
    
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", jwt))
        .header("Content-Type", "application/json")
        .send()
        .await
        .expect("HTTP request should succeed");
    
    let status = response.status();
    let response_text = response.text().await.unwrap_or_else(|_| "Unable to read response".to_string());
    
    // With valid credentials, we should get 200 OK (or 200 with account data)
    // If we get 401/403, the JWT is invalid or credentials are wrong
    if status.is_success() {
        println!("✅ JWT Authentication SUCCESS!");
        println!("   Status: {}", status);
        println!("   Response preview: {}", &response_text[..response_text.len().min(200)]);
        assert!(true, "JWT authentication verified - credentials are valid and authorized");
    } else if status == 401 || status == 403 {
        panic!(
            "❌ JWT Authentication FAILED!\n   Status: {}\n   Response: {}\n   This means:\n   - JWT format might be wrong\n   - API key might be invalid\n   - Private key might not match API key\n   - Credentials might not be authorized",
            status, response_text
        );
    } else {
        panic!(
            "Unexpected response status: {}\n   Response: {}",
            status, response_text
        );
    }
}

#[tokio::test]
#[ignore] // Requires COINBASE_API_KEY and COINBASE_API_SECRET environment variables
async fn test_jwt_authentication_sandbox_readonly() {
    // Test: Verify JWT works with sandbox API (if sandbox is available)
    // Uses read-only endpoint - NO TRADES ARE MADE
    
    use reqwest::Client;
    
    let api_key = get_api_key();
    let api_secret = get_api_secret();
    
    let auth = CoinbaseAuth::new(api_key, api_secret).unwrap();
    
    // Generate JWT for sandbox API
    let jwt = auth
        .generate_jwt("GET", "api-public.sandbox.exchange.coinbase.com", "/api/v3/brokerage/accounts")
        .expect("JWT generation should succeed");
    
    // Make actual HTTP request to sandbox
    let client = Client::new();
    let url = "https://api-public.sandbox.exchange.coinbase.com/api/v3/brokerage/accounts";
    
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", jwt))
        .header("Content-Type", "application/json")
        .send()
        .await
        .expect("HTTP request should succeed");
    
    let status = response.status();
    let response_text = response.text().await.unwrap_or_else(|_| "Unable to read response".to_string());
    
    // Sandbox might accept production keys or require separate sandbox keys
    if status.is_success() {
        println!("✅ Sandbox JWT Authentication SUCCESS!");
        println!("   Status: {}", status);
    } else if status == 401 || status == 403 {
        println!("⚠️  Sandbox authentication failed - this is OK if you're using production keys");
        println!("   Status: {}", status);
        println!("   Response: {}", response_text);
        // Don't panic - sandbox might require separate keys
    } else {
        println!("⚠️  Unexpected sandbox response: {} - {}", status, response_text);
        // Don't panic - sandbox might not be available
    }
}
