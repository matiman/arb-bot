//! Coinbase REST API Client
//!
//! Implements REST API client for Coinbase Advanced Trade API with JWT authentication.
//!
//! Based on: https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/guides/authentication

use crate::error::{ArbitrageError, Result};
use crate::exchanges::coinbase::auth::CoinbaseAuth;
use crate::exchanges::coinbase::types::{CoinbaseAccountsResponse, MarketIocConfig};
use crate::exchanges::{Order, OrderResult, OrderSide, OrderType};
use reqwest::Client;
use rust_decimal::Decimal;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

/// Rate limiter for Coinbase API (10 requests per second)
struct RateLimiter {
    max_requests: usize,
    window: Duration,
    last_request: Arc<Mutex<Option<Instant>>>,
    request_count: Arc<Mutex<usize>>,
}

impl RateLimiter {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            last_request: Arc::new(Mutex::new(None)),
            request_count: Arc::new(Mutex::new(0)),
        }
    }

    async fn wait_if_needed(&self) {
        let now = Instant::now();
        let mut last_request = self.last_request.lock().unwrap();
        let mut request_count = self.request_count.lock().unwrap();

        if let Some(last) = *last_request {
            if now.duration_since(last) >= self.window {
                // Window expired, reset counter
                *request_count = 0;
                *last_request = Some(now);
            } else if *request_count >= self.max_requests {
                // Need to wait until window expires
                let wait_time = self.window - now.duration_since(last);
                drop(last_request);
                drop(request_count);
                sleep(wait_time).await;
                // Reset after waiting
                let mut last_request = self.last_request.lock().unwrap();
                let mut request_count = self.request_count.lock().unwrap();
                *request_count = 0;
                *last_request = Some(Instant::now());
            } else {
                *request_count += 1;
            }
        } else {
            *last_request = Some(now);
            *request_count = 1;
        }
    }
}

/// Coinbase REST API client
pub struct CoinbaseRestClient {
    client: Client,
    auth: CoinbaseAuth,
    base_url: String,
    rate_limiter: RateLimiter,
}

impl CoinbaseRestClient {
    /// Create a new Coinbase REST API client
    ///
    /// # Arguments
    /// * `api_key` - Full API key path (e.g., "organizations/org-id/apiKeys/key-id")
    /// * `api_secret` - EC private key in PEM format
    /// * `sandbox` - If true, use sandbox API; otherwise use production
    ///
    /// # Returns
    /// Result containing CoinbaseRestClient or AuthenticationError if credentials are invalid
    pub fn new(api_key: String, api_secret: String, sandbox: bool) -> Result<Self> {
        let auth = CoinbaseAuth::new(api_key, api_secret)?;

        let base_url = if sandbox {
            "https://api-public.sandbox.exchange.coinbase.com".to_string()
        } else {
            "https://api.coinbase.com".to_string()
        };

        Ok(Self {
            client: Client::new(),
            auth,
            base_url,
            rate_limiter: RateLimiter::new(10, Duration::from_secs(1)), // 10 req/sec
        })
    }

    /// Get account balance for a specific currency
    ///
    /// # Arguments
    /// * `asset` - Currency symbol (e.g., "USDC", "SOL")
    ///
    /// # Returns
    /// Available balance as Decimal, or ExchangeError if account not found
    pub async fn get_balance(&self, asset: &str) -> Result<Decimal> {
        self.rate_limiter.wait_if_needed().await;

        let path = "/api/v3/brokerage/accounts";
        let url = format!("{}{}", self.base_url, path);

        let jwt = self
            .auth
            .generate_jwt("GET", &self.base_url.replace("https://", ""), path)?;

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| ArbitrageError::ExchangeError {
                exchange: "coinbase".to_string(),
                message: format!("HTTP request failed: {}", e),
                code: None,
            })?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read response".to_string());

        if !status.is_success() {
            if status == 401 || status == 403 {
                return Err(ArbitrageError::AuthenticationError {
                    exchange: "coinbase".to_string(),
                    reason: format!("Authentication failed: {}", response_text),
                });
            }
            return Err(ArbitrageError::ExchangeError {
                exchange: "coinbase".to_string(),
                message: format!("API error ({}): {}", status, response_text),
                code: Some(status.as_u16() as i32),
            });
        }

        let accounts_response: CoinbaseAccountsResponse = serde_json::from_str(&response_text)
            .map_err(|e| ArbitrageError::ExchangeError {
                exchange: "coinbase".to_string(),
                message: format!("Failed to parse accounts response: {}", e),
                code: None,
            })?;

        // Find account matching the requested asset
        let account = accounts_response
            .accounts
            .iter()
            .find(|acc| acc.currency == asset)
            .ok_or_else(|| ArbitrageError::ExchangeError {
                exchange: "coinbase".to_string(),
                message: format!("Account not found for currency: {}", asset),
                code: None,
            })?;

        account.available_balance_decimal()
    }

    /// Place a market order (IOC - Immediate or Cancel)
    ///
    /// # Arguments
    /// * `order` - Order to place (must be Market order type)
    ///
    /// # Returns
    /// OrderResult with order details, or error if order placement fails
    pub async fn place_market_order(&self, order: Order) -> Result<OrderResult> {
        // Validate order type
        if !matches!(order.order_type, OrderType::Market) {
            return Err(ArbitrageError::ExchangeError {
                exchange: "coinbase".to_string(),
                message: "Only market orders are supported".to_string(),
                code: None,
            });
        }

        self.rate_limiter.wait_if_needed().await;

        // Convert pair format: "SOL/USDC" -> "SOL-USDC"
        let product_id = order.pair.replace("/", "-");

        // Convert side: OrderSide -> "BUY" or "SELL"
        let side = match order.side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };

        // For BUY orders: use quote_size (amount in quote currency, e.g., USDC)
        // For SELL orders: use base_size (amount in base currency, e.g., SOL)
        // Coinbase requires specific precision: 2 decimal places for quote_size
        let market_ioc = match order.side {
            OrderSide::Buy => {
                // Round to 2 decimal places for USDC
                let rounded = (order.quantity * Decimal::from(100)).round() / Decimal::from(100);
                MarketIocConfig {
                    quote_size: Some(format!("{:.2}", rounded)),
                    base_size: None,
                }
            }
            OrderSide::Sell => {
                // Round SOL to 8 decimal places (typical precision for crypto)
                let rounded = (order.quantity * Decimal::from(100_000_000)).round()
                    / Decimal::from(100_000_000);
                MarketIocConfig {
                    quote_size: None,
                    base_size: Some(
                        format!("{:.8}", rounded)
                            .trim_end_matches('0')
                            .trim_end_matches('.')
                            .to_string(),
                    ),
                }
            }
        };

        // Generate a unique client_order_id (Coinbase seems to require it)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let client_order_id = format!("arb-bot-{}", timestamp);

        // Build request JSON manually to avoid sending null values
        let mut request_json = serde_json::json!({
            "product_id": product_id,
            "side": side,
            "client_order_id": client_order_id,
            "order_configuration": {
                "market_market_ioc": {}
            }
        });

        // Add only non-null values to market_market_ioc
        if let Some(obj) = request_json
            .get_mut("order_configuration")
            .and_then(|oc| oc.get_mut("market_market_ioc"))
            .and_then(|m| m.as_object_mut())
        {
            if let Some(quote_size) = market_ioc.quote_size {
                obj.insert(
                    "quote_size".to_string(),
                    serde_json::Value::String(quote_size),
                );
            }
            if let Some(base_size) = market_ioc.base_size {
                obj.insert(
                    "base_size".to_string(),
                    serde_json::Value::String(base_size),
                );
            }
        }

        let path = "/api/v3/brokerage/orders";
        let url = format!("{}{}", self.base_url, path);

        let jwt = self
            .auth
            .generate_jwt("POST", &self.base_url.replace("https://", ""), path)?;

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .header("Content-Type", "application/json")
            .json(&request_json)
            .send()
            .await
            .map_err(|e| ArbitrageError::ExchangeError {
                exchange: "coinbase".to_string(),
                message: format!("HTTP request failed: {}", e),
                code: None,
            })?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read response".to_string());

        if !status.is_success() {
            if status == 401 || status == 403 {
                return Err(ArbitrageError::AuthenticationError {
                    exchange: "coinbase".to_string(),
                    reason: format!("Authentication failed: {}", response_text),
                });
            }
            return Err(ArbitrageError::ExchangeError {
                exchange: "coinbase".to_string(),
                message: format!("Order placement failed ({}): {}", status, response_text),
                code: Some(status.as_u16() as i32),
            });
        }

        let wrapper: crate::exchanges::coinbase::types::CoinbaseOrderResponseWrapper =
            serde_json::from_str(&response_text).map_err(|e| ArbitrageError::ExchangeError {
                exchange: "coinbase".to_string(),
                message: format!(
                    "Failed to parse order response: {}. Response was: {}",
                    e, response_text
                ),
                code: None,
            })?;

        if !wrapper.success {
            let error_msg = wrapper
                .error_response
                .map(|e| format!("{}: {}", e.error, e.message))
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(ArbitrageError::ExchangeError {
                exchange: "coinbase".to_string(),
                message: format!("Order placement failed: {}", error_msg),
                code: None,
            });
        }

        let order_response =
            wrapper
                .success_response
                .ok_or_else(|| ArbitrageError::ExchangeError {
                    exchange: "coinbase".to_string(),
                    message: "Order response missing success_response".to_string(),
                    code: None,
                })?;

        // For market IOC orders, status is typically "FILLED" immediately
        // If status is not in response, assume FILLED for market orders
        let mut response_with_status = order_response;
        if response_with_status.status.is_none() {
            response_with_status.status = Some("FILLED".to_string());
        }

        response_with_status.try_into()
    }
}
