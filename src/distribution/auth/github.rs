use crate::error::*;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, DeviceAuthorizationUrl, TokenUrl};

pub fn get_token() -> Result<String> {
    // "ocipkg" GitHub OAuth App
    // https://github.com/settings/applications/1952947
    let _client = BasicClient::new(
        ClientId::new("d9ec21750789e17a0c5c".to_string()),
        None,
        AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap()),
    )
    .set_device_authorization_url(
        DeviceAuthorizationUrl::new("https://github.com/login/device/code".to_string()).unwrap(),
    );

    todo!()
}
