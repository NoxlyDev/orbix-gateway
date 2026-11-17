use serde::Serialize;

#[derive(Clone, Debug)]
pub struct VerifyWebhookSource;

#[derive(Debug, Clone, Serialize)]
pub struct ConnectorMandateDetails {
    pub connector_mandate_id: orbix_masking::Secret<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectorNetworkTxnId(orbix_masking::Secret<String>);

impl ConnectorNetworkTxnId {
    pub fn new(txn_id: orbix_masking::Secret<String>) -> Self {
        Self(txn_id)
    }
    pub fn get_id(&self) -> &orbix_masking::Secret<String> {
        &self.0
    }
}

/// Associated data about a payment shared by a connector over a webhook.
#[derive(Debug, Clone, Serialize)]
pub struct WebhookAssociatedData {
    pub payment_attempt: PaymentAttemptAssociatedData,
    pub payment_method: Option<crate::payment_method_data::PaymentMethodData>,
}

impl WebhookAssociatedData {
    pub fn is_empty(&self) -> bool {
        let Self {
            payment_attempt,
            payment_method,
        } = self;

        payment_attempt.is_empty() && payment_method.is_none()
    }
}

/// Associated data written to the payment attempt.
#[derive(Debug, Clone, Serialize)]
pub struct PaymentAttemptAssociatedData {
    pub sender_payment_instrument_id: Option<orbix_masking::Secret<String>>,
}

impl PaymentAttemptAssociatedData {
    pub fn is_empty(&self) -> bool {
        let Self {
            sender_payment_instrument_id,
        } = self;

        sender_payment_instrument_id.is_none()
    }
}
