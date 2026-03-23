use anyhow::{Context, Result};
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod, ApplicationSecret};

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Token(pub String);

#[derive(Deserialize)]
struct SecretWrapper {
    installed: Option<ApplicationSecret>,
    web: Option<ApplicationSecret>,
}

/// The OAuth 2.0 client secret for the application.
/// This identifies the app to Google's authentication server.
const CLIENT_SECRET: &str = include_str!("../client_secret.json");

/// Authenticates the user via the browser and returns an access token.
pub async fn get_token() -> Result<Token> {
    // Parse the embedded application secret, handling the "installed" wrapper
    let wrapper: SecretWrapper = serde_json::from_str(CLIENT_SECRET)
        .context("Failed to parse client_secret.json - make sure it contains 'installed' or 'web' keys")?;
    
    let secret = wrapper.installed.or(wrapper.web)
        .context("No application secret found inside 'installed' or 'web' keys")?;

    // Define the required scopes
    let scopes = &[
        "https://www.googleapis.com/auth/spreadsheets",
        "https://www.googleapis.com/auth/drive",
    ];

    // Build the authenticator (v11 pattern)
    // We use Interactive method which is very robust for desktop apps
    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::Interactive)
        .build()
        .await
        .context("Failed to build OAuth 2.0 authenticator")?;

    // Request the token
    let token = auth
        .token(scopes)
        .await
        .context("Failed to retrieve user access token")?;

    let token_str = token
        .token()
        .context("Access token is empty")?
        .to_string();

    Ok(Token(token_str))
}
