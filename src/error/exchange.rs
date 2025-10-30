#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExchangeErrorKind {
    ConnectionFailed,
    OrderFailed,
    InsufficientFunds,
    InvalidOrder,
    RateLimitExceeded,
    ApiError(i32),
    Unknown,
}

impl ExchangeErrorKind {
    pub fn is_retryable(&self) -> bool {
        matches!(self, ExchangeErrorKind::ConnectionFailed | ExchangeErrorKind::RateLimitExceeded)
    }

    pub fn is_client_error(&self) -> bool {
        matches!(self, ExchangeErrorKind::InvalidOrder | ExchangeErrorKind::InsufficientFunds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_kinds() {
        assert!(ExchangeErrorKind::ConnectionFailed.is_retryable());
        assert!(ExchangeErrorKind::RateLimitExceeded.is_retryable());
        assert!(!ExchangeErrorKind::InvalidOrder.is_retryable());
    }

    #[test]
    fn client_error_kinds() {
        assert!(ExchangeErrorKind::InvalidOrder.is_client_error());
        assert!(ExchangeErrorKind::InsufficientFunds.is_client_error());
        assert!(!ExchangeErrorKind::ConnectionFailed.is_client_error());
    }
}
