//! Coinbase-specific types for REST API
//!
//! Types for Coinbase Advanced Trade API request/response structures.

use crate::exchanges::{OrderResult, OrderStatus};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Coinbase order request
#[derive(Debug, Serialize)]
pub struct CoinbaseOrderRequest {
    pub product_id: String, // e.g., "SOL-USDC"
    pub side: String,       // "BUY" or "SELL"
    #[serde(rename = "order_configuration")]
    pub order_configuration: OrderConfiguration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>, // Optional client-provided order ID
}

#[derive(Debug, Serialize)]
pub struct OrderConfiguration {
    #[serde(rename = "market_market_ioc")]
    pub market_ioc: MarketIocConfig,
}

#[derive(Debug, Serialize)]
pub struct MarketIocConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_size: Option<String>, // Quote currency amount (e.g., "20" USDC for buy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_size: Option<String>, // Base currency amount (e.g., "0.1" SOL for sell)
}

/// Coinbase order response (wrapped in success_response)
#[derive(Debug, Deserialize)]
pub struct CoinbaseOrderResponseWrapper {
    pub success: bool,
    #[serde(default)]
    pub success_response: Option<CoinbaseOrderResponse>,
    #[serde(default)]
    pub error_response: Option<CoinbaseErrorResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CoinbaseErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(default)]
    pub error_details: String,
}

/// Coinbase order response
#[derive(Debug, Deserialize)]
pub struct CoinbaseOrderResponse {
    pub order_id: String,
    pub product_id: String,
    pub side: String,
    #[serde(default)]
    pub client_order_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>, // "FILLED", "PENDING", "CANCELLED", etc. (may not be in initial response)
    #[serde(rename = "average_filled_price")]
    #[serde(default)]
    pub average_filled_price: Option<String>,
    #[serde(rename = "filled_size")]
    #[serde(default)]
    pub filled_size: Option<String>,
    #[serde(default)]
    pub fees: Option<String>,
    #[serde(rename = "number_of_fills")]
    #[serde(default)]
    pub number_of_fills: Option<u32>,
    #[serde(rename = "created_time")]
    #[serde(default)]
    pub created_time: Option<String>,
}

impl TryFrom<CoinbaseOrderResponse> for OrderResult {
    type Error = crate::error::ArbitrageError;

    fn try_from(response: CoinbaseOrderResponse) -> Result<Self, Self::Error> {
        let status = match response.status.as_deref().unwrap_or("FILLED") {
            "FILLED" => OrderStatus::Filled,
            "PENDING" => OrderStatus::Pending,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "CANCELLED" => OrderStatus::Cancelled,
            _ => OrderStatus::Failed,
        };

        let filled_quantity = response
            .filled_size
            .as_ref()
            .and_then(|s| Decimal::from_str(s).ok())
            .unwrap_or(Decimal::ZERO);

        let average_price = response
            .average_filled_price
            .as_ref()
            .and_then(|s| Decimal::from_str(s).ok());

        // Parse fees (Coinbase returns fees as a string, e.g., "0.5")
        let fee = response
            .fees
            .as_ref()
            .and_then(|s| Decimal::from_str(s).ok())
            .unwrap_or(Decimal::ZERO);

        // Fee asset is typically the quote currency (USDC for SOL/USDC)
        let fee_asset = response
            .product_id
            .split('-')
            .nth(1)
            .unwrap_or("USDC")
            .to_string();

        Ok(OrderResult {
            order_id: response.order_id,
            status,
            filled_quantity,
            average_price,
            fee,
            fee_asset,
            timestamp: response
                .created_time
                .as_ref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
        })
    }
}

/// Coinbase account response
#[derive(Debug, Deserialize)]
pub struct CoinbaseAccountsResponse {
    pub accounts: Vec<CoinbaseAccount>,
}

#[derive(Debug, Deserialize)]
pub struct CoinbaseAccount {
    pub uuid: String,
    pub name: String,
    pub currency: String,
    #[serde(rename = "available_balance")]
    pub available_balance: CoinbaseBalance,
    #[serde(default)]
    pub hold: Option<CoinbaseBalance>,
}

#[derive(Debug, Deserialize)]
pub struct CoinbaseBalance {
    pub value: String,
    pub currency: String,
}

impl CoinbaseAccount {
    /// Get available balance as Decimal
    pub fn available_balance_decimal(&self) -> Result<Decimal, crate::error::ArbitrageError> {
        Decimal::from_str(&self.available_balance.value).map_err(|e| {
            crate::error::ArbitrageError::ExchangeError {
                exchange: "coinbase".to_string(),
                message: format!("Failed to parse balance: {}", e),
                code: None,
            }
        })
    }
}
