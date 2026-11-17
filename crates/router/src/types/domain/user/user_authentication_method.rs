use std::sync::LazyLock;

use common_enums::{Owner, UserAuthType};
use diesel_models::UserAuthenticationMethod;

pub static DEFAULT_USER_AUTH_METHOD: LazyLock<UserAuthenticationMethod> =
    LazyLock::new(|| UserAuthenticationMethod {
        id: String::from("orbix_default"),
        auth_id: String::from("orbix"),
        owner_id: String::from("orbix"),
        owner_type: Owner::Tenant,
        auth_type: UserAuthType::Password,
        private_config: None,
        public_config: None,
        allow_signup: true,
        created_at: common_utils::date_time::now(),
        last_modified_at: common_utils::date_time::now(),
        email_domain: String::from("orbix"),
    });
