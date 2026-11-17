use std::{collections::HashSet, time::Duration};

use common_enums::{connector_enums::Connector, CardNetwork};
use error_stack::ResultExt;
use orbix_domain_models::router_data::ConnectorAuthType;
use orbix_masking::Secret;
use unified_connector_service_client::payments as payments_grpc;

use crate::{
    configs::settings,
    core::{
        errors::{self, RouterResult},
        unified_connector_service::connector_config::OrbixMetadata,
    },
};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AccountUpdaterCredentialSource {
    None,
    Application,
}

#[derive(Debug, Clone)]
pub enum ResolvedAccountUpdaterConfig {
    Orbix(OrbixConfig),
}

impl ResolvedAccountUpdaterConfig {
    pub fn supported_card_networks(&self) -> &HashSet<CardNetwork> {
        match self {
            Self::Orbix(orbix) => &orbix.supported_card_networks,
        }
    }

    pub fn refresh_timeout(&self) -> Duration {
        match self {
            Self::Orbix(orbix) => orbix.refresh_timeout,
        }
    }

    /// The account updater service backing this config, recorded against every row it writes.
    pub fn service(&self) -> Connector {
        match self {
            Self::Orbix(_) => Connector::Orbix,
        }
    }

    /// Builds the connector, auth type and metadata the connector config is resolved from.
    pub fn build_connector_credentials(
        &self,
    ) -> RouterResult<(Connector, ConnectorAuthType, serde_json::Value)> {
        match self {
            Self::Orbix(orbix) => {
                let metadata = serde_json::to_value(OrbixMetadata {
                    merchant_id: orbix.merchant_id.clone(),
                    base_url: orbix.base_url.to_string(),
                    orbix_encryption_public_key: orbix.euler_encryption_public_key.clone(),
                    response_decryption_private_key: orbix.au_decryption_pvt_key.clone(),
                    card_sync_key_id: orbix.card_sync_key_id.clone(),
                })
                .change_context(errors::ApiErrorResponse::InternalServerError)
                .attach_printable("Failed to serialize the Orbix Account Updater metadata")?;

                Ok((
                    Connector::Orbix,
                    ConnectorAuthType::HeaderKey {
                        api_key: orbix.api_key.clone(),
                    },
                    metadata,
                ))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrbixConfig {
    pub base_url: url::Url,
    pub api_key: Secret<String>,
    pub merchant_id: String,
    pub euler_encryption_public_key: Secret<String>,
    pub au_decryption_pvt_key: Secret<String>,
    pub card_sync_key_id: String,
    pub supported_card_networks: HashSet<CardNetwork>,
    pub refresh_timeout: Duration,
}

impl From<&settings::OrbixAccountUpdaterConfig> for OrbixConfig {
    fn from(config: &settings::OrbixAccountUpdaterConfig) -> Self {
        Self {
            base_url: config.base_url.clone(),
            api_key: config.api_key.clone(),
            merchant_id: config.merchant_id.clone(),
            euler_encryption_public_key: config.euler_encryption_public_key.clone(),
            au_decryption_pvt_key: config.au_decryption_pvt_key.clone(),
            card_sync_key_id: config.card_sync_key_id.clone(),
            supported_card_networks: config.supported_card_networks.clone(),
            refresh_timeout: Duration::from_secs(config.refresh_timeout_in_secs),
        }
    }
}

impl From<&settings::AccountUpdaterConfig> for ResolvedAccountUpdaterConfig {
    fn from(config: &settings::AccountUpdaterConfig) -> Self {
        match config {
            settings::AccountUpdaterConfig::Orbix(orbix) => Self::Orbix(orbix.into()),
        }
    }
}

pub enum RefreshResult {
    Card(CardRefreshedData),
}

pub struct CardRefreshedData {
    pub outcome: CardOutcome,
    pub service: Connector,
}

pub enum CardOutcome {
    AccountUpdated(payments_grpc::CardDetailsWithNoCvc),
    ExpiryUpdated(payments_grpc::CardDetailsWithNoCvc),
    Closed,
    NoChange,
    NotFound,
    ContactIssuer,
    Unspecified,
}

pub enum RefreshedCard {
    CardOpen(Box<payments_grpc::CardDetailsWithNoCvc>),
    CardClosed,
}

impl CardOutcome {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::AccountUpdated(_) => {
                payments_grpc::CardRefreshOutcome::CardRefreshAccountUpdated.as_str_name()
            }
            Self::ExpiryUpdated(_) => {
                payments_grpc::CardRefreshOutcome::CardRefreshExpiryUpdated.as_str_name()
            }
            Self::Closed => payments_grpc::CardRefreshOutcome::CardRefreshClosed.as_str_name(),
            Self::NoChange => payments_grpc::CardRefreshOutcome::CardRefreshNoChange.as_str_name(),
            Self::NotFound => payments_grpc::CardRefreshOutcome::CardRefreshNotFound.as_str_name(),
            Self::ContactIssuer => {
                payments_grpc::CardRefreshOutcome::CardRefreshContactIssuer.as_str_name()
            }
            Self::Unspecified => payments_grpc::CardRefreshOutcome::Unspecified.as_str_name(),
        }
    }

    pub fn get_new_card_details(self) -> Option<RefreshedCard> {
        match self {
            Self::AccountUpdated(card) | Self::ExpiryUpdated(card) => {
                Some(RefreshedCard::CardOpen(Box::new(card)))
            }
            Self::Closed => Some(RefreshedCard::CardClosed),
            Self::NoChange | Self::NotFound | Self::ContactIssuer | Self::Unspecified => None,
        }
    }
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountUpdaterError {
    #[error("Account Updater is not enabled for these dimensions")]
    NotEnabled,
    #[error("Account Updater application config is missing or invalid")]
    MissingApplicationConfig,
    #[error("Payment method is not a card")]
    PaymentMethodNotACard,
    #[error("Payment method is not active")]
    PaymentMethodNotActive,
    #[error("Card network is not supported by Account Updater")]
    UnsupportedNetwork,
    #[error("Account Updater is not supported on the legacy locker")]
    LegacyLockerUnsupported,
    #[error("Account Updater is not supported on an external vault profile")]
    ExternalVaultUnsupported,
    #[error("Could not resolve the provider profile to check the deployment")]
    ProviderProfileUnresolved,
    #[error("Stored card cannot be used for Account Updater")]
    CardUnusable,
    #[error("Account Updater refresh call failed")]
    RefreshCallFailed,
    #[error("Account Updater refresh returned an error")]
    RefreshReturnedError,
    #[error("Account Updater could not store the reported card change")]
    StoreFailed,
}
