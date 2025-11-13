//! Binance-specific response types

use crate::exchanges::{OrderResult, OrderStatus};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::Deserialize;

/// Binance order response
#[derive(Debug, Deserialize)]
pub struct BinanceOrderResponse {
    #[serde(rename = "orderId")]
    pub order_id: u64,
    pub symbol: String,
    pub status: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: String,
}

impl From<BinanceOrderResponse> for OrderResult {
    fn from(response: BinanceOrderResponse) -> Self {
        let executed_qty = Decimal::from_str_exact(&response.executed_qty)
            .unwrap_or(Decimal::ZERO);
        let cumulative_quote_qty = Decimal::from_str_exact(&response.cumulative_quote_qty)
            .unwrap_or(Decimal::ZERO);

        // Map Binance status to our OrderStatus
        let status = match response.status.as_str() {
            "FILLED" => OrderStatus::Filled,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "NEW" | "ACCEPTED" => OrderStatus::Pending,
            "CANCELED" => OrderStatus::Cancelled,
            "REJECTED" | "EXPIRED" => OrderStatus::Failed,
            _ => OrderStatus::Pending,
        };

        OrderResult {
            order_id: response.order_id.to_string(),
            status,
            filled_quantity: executed_qty,
            average_price: if executed_qty > Decimal::ZERO {
                Some(cumulative_quote_qty / executed_qty)
            } else {
                None
            },
            fee: Decimal::ZERO, // Binance fee info comes from separate endpoint
            fee_asset: "USDC".to_string(), // Default, should be determined from asset
            timestamp: Utc::now(),
        }
    }
}

/// Binance account information
#[derive(Debug, Deserialize)]
pub struct BinanceAccountInfo {
    pub balances: Vec<BinanceBalance>,
}

/// Binance balance for an asset
#[derive(Debug, Deserialize)]
pub struct BinanceBalance {
    pub asset: String,
    #[serde(deserialize_with = "decimal_from_str")]
    pub free: Decimal,
    #[serde(deserialize_with = "decimal_from_str")]
    pub locked: Decimal,
}

fn decimal_from_str<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Decimal::from_str_exact(&s).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_response_conversion() {
        let response = BinanceOrderResponse {
            order_id: 12345,
            symbol: "SOLUSDC".to_string(),
            status: "FILLED".to_string(),
            executed_qty: "10.0".to_string(),
            cumulative_quote_qty: "1435.0".to_string(),
        };

        let order_result: OrderResult = response.into();
        assert_eq!(order_result.order_id, "12345");
        assert!(matches!(order_result.status, OrderStatus::Filled));
        assert_eq!(order_result.filled_quantity, Decimal::from_str_exact("10.0").unwrap());
        assert_eq!(order_result.average_price, Some(Decimal::from_str_exact("143.5").unwrap()));
    }
}

