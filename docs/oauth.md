# OAuth 2.0 / OIDC Configuration

Tuwunel now supports OAuth 2.0 and OpenID Connect (OIDC) authentication, allowing users to log in using external identity providers like Google, GitHub, Keycloak, or any other OAuth 2.0/OIDC compliant provider.

This implementation includes support for:
- **MSC2965**: OIDC-aware clients discovery
- **MSC3861**: Matrix + OIDC native integration for account management

## Configuration

Add the following section to your `tuwunel.toml` configuration file:

```toml
[global.oauth]
# Enable OAuth 2.0 authentication
enable = true

# OAuth 2.0 issuer URL (the base URL of your OAuth provider)
# Examples:
# - Google: "https://accounts.google.com"
# - Keycloak: "https://keycloak.example.com/realms/myrealm"
# - GitHub: "https://github.com" (note: GitHub doesn't fully support OIDC discovery)
issuer = "https://accounts.google.com"

# OAuth 2.0 client ID (obtained from your OAuth provider)
client_id = "your-client-id"

# OAuth 2.0 client secret (obtained from your OAuth provider)
client_secret = "your-client-secret"

# Redirect URI - must match what you registered with your OAuth provider
# This should be: https://your-matrix-domain/_matrix/client/v3/login/sso/callback
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"

# OAuth 2.0 scopes to request
# Default: ["openid", "profile", "email"]
scopes = ["openid", "profile", "email"]

# Automatically create new users from valid OAuth claims
# Default: true
register_user = true

# Optional: Manually specify endpoints (if auto-discovery is not available)
# These are usually auto-discovered from the issuer's .well-known/openid-configuration
# authorization_endpoint = "https://accounts.google.com/o/oauth2/v2/auth"
# token_endpoint = "https://oauth2.googleapis.com/token"
# userinfo_endpoint = "https://openidconnect.googleapis.com/v1/userinfo"
# jwks_uri = "https://www.googleapis.com/oauth2/v3/certs"

# Enable OIDC discovery (.well-known/openid-configuration)
# Default: true
enable_discovery = true

# Claim to use as the Matrix user ID localpart
# Options: "sub", "email", "preferred_username"
# Default: "sub"
subject_claim = "sub"

# Claim to use as the display name
# Default: "name"
displayname_claim = "name"

# MSC3861: Account management URL (optional)
# URL where users can manage their account settings at the OAuth provider
# account_management_url = "https://accounts.google.com/myaccount"

# MSC3861: Enable experimental OAuth delegation mode (optional)
# When enabled, provides additional OAuth delegation features
# Default: false
# experimental_msc3861 = false
```

## MSC3861: Native OIDC Integration

MSC3861 defines native integration between Matrix and OpenID Connect providers, enabling:

- **Account Management**: Direct links to OAuth provider account pages
- **Unified Authentication**: OAuth provider handles all authentication
- **Better Client Experience**: Seamless integration with OIDC-aware Matrix clients

### Enabling MSC3861 Features

To enable experimental MSC3861 support, add to your configuration:

```toml
[global.oauth]
enable = true
experimental_msc3861 = true
account_management_url = "https://your-provider.com/account"
issuer = "https://your-provider.com"
# ... other OAuth settings
```

This enables:
- `/.well-known/matrix/client` includes authentication issuer information
- `/_matrix/client/unstable/org.matrix.msc3861/account_management` endpoint
- Account management actions in OIDC discovery

## Setting up with common providers

### Google

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Navigate to "APIs & Services" > "Credentials"
4. Create an OAuth 2.0 Client ID
5. Add your redirect URI: `https://your-matrix-domain/_matrix/client/v3/login/sso/callback`
6. Copy the client ID and client secret to your configuration

```toml
[global.oauth]
enable = true
issuer = "https://accounts.google.com"
client_id = "your-client-id.apps.googleusercontent.com"
client_secret = "your-client-secret"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]
```

### Keycloak

1. Log in to your Keycloak admin console
2. Create or select a realm
3. Create a new client
4. Set the redirect URI
5. Copy the client ID and secret

```toml
[global.oauth]
enable = true
issuer = "https://keycloak.example.com/realms/myrealm"
client_id = "matrix-client"
client_secret = "your-client-secret"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]
```

### Generic OIDC Provider

For any OIDC-compliant provider:

```toml
[global.oauth]
enable = true
issuer = "https://your-provider.com"
client_id = "your-client-id"
client_secret = "your-client-secret"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]
```

## How it works

1. Client requests login types via `GET /_matrix/client/v3/login`
2. If OAuth is enabled, the response includes the `m.login.sso` flow
3. Client redirects user to `GET /_matrix/client/v3/login/sso/redirect`
4. User is redirected to the OAuth provider's authorization page
5. After authentication, user is redirected back to the callback URL with an authorization code
6. Server exchanges the code for an access token
7. Server fetches user information from the provider
8. Server creates or updates the Matrix user account
9. Server returns Matrix access token to the client

## Matrix Client Support

Not all Matrix clients support SSO/OAuth login flows. Known clients with support include:

- Element Web
- Element Android
- Element iOS
- SchildiChat

## Security Considerations

- Always use HTTPS for production deployments
- Keep your client secret secure and never commit it to version control
- Use environment variables or secure configuration management
- Regularly rotate your client secrets
- Monitor for suspicious login attempts
- Consider implementing additional security measures like 2FA at the OAuth provider level

## Troubleshooting

### "OAuth login is not enabled"

Make sure `enable = true` is set in the `[global.oauth]` section.

### "OAuth authorization endpoint not configured"

Either:
1. Enable OIDC discovery (`enable_discovery = true`) and ensure your issuer supports it
2. Manually configure the endpoints in your config

### "User registration via OAuth is disabled"

Set `register_user = true` in your OAuth configuration if you want users to be auto-registered.

### Redirect URI mismatch

Ensure the `redirect_uri` in your config exactly matches what you registered with your OAuth provider.

## Future Enhancements

Planned features for OAuth 2.0 support include:

- Token introspection support
- Client registration endpoint
- Multiple OAuth provider support
- Custom claim mapping
- Account linking for existing users
- Refresh token support
- PKCE (Proof Key for Code Exchange) support
