# OAuth 2.0 Configuration Examples

This document provides complete configuration examples for setting up OAuth 2.0 / OIDC authentication with Tuwunel.

## Minimal OAuth Configuration

The minimal configuration required to enable OAuth:

```toml
[global.oauth]
enable = true
issuer = "https://accounts.google.com"
client_id = "your-client-id.apps.googleusercontent.com"
client_secret = "your-client-secret"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
```

## Complete OAuth Configuration

A complete configuration with all available options:

```toml
[global.oauth]
# Enable OAuth 2.0 authentication
enable = true

# OAuth 2.0 issuer URL
issuer = "https://accounts.google.com"

# OAuth 2.0 client credentials
client_id = "your-client-id.apps.googleusercontent.com"
client_secret = "your-client-secret"

# Redirect URI
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"

# Scopes to request
scopes = ["openid", "profile", "email"]

# Auto-register users
register_user = true

# Manual endpoint configuration (optional, usually auto-discovered)
authorization_endpoint = "https://accounts.google.com/o/oauth2/v2/auth"
token_endpoint = "https://oauth2.googleapis.com/token"
userinfo_endpoint = "https://openidconnect.googleapis.com/v1/userinfo"
jwks_uri = "https://www.googleapis.com/oauth2/v3/certs"

# Enable OIDC discovery
enable_discovery = true

# Claim mapping
subject_claim = "sub"
displayname_claim = "name"
```

## Provider-Specific Examples

### Google

```toml
[global.oauth]
enable = true
issuer = "https://accounts.google.com"
client_id = "123456789-abc123.apps.googleusercontent.com"
client_secret = "GOCSPX-xxxxxxxxxxxxxxxxxxxx"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]
register_user = true
```

### Keycloak

```toml
[global.oauth]
enable = true
issuer = "https://keycloak.example.com/realms/matrix"
client_id = "matrix-client"
client_secret = "abcdef12-3456-7890-abcd-ef1234567890"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]
register_user = true
subject_claim = "preferred_username"
```

### Authentik

```toml
[global.oauth]
enable = true
issuer = "https://authentik.example.com/application/o/matrix/"
client_id = "your-client-id"
client_secret = "your-client-secret"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]
register_user = true
```

### Auth0

```toml
[global.oauth]
enable = true
issuer = "https://your-tenant.auth0.com"
client_id = "your-client-id"
client_secret = "your-client-secret"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]
register_user = true
```

### Okta

```toml
[global.oauth]
enable = true
issuer = "https://your-domain.okta.com"
client_id = "your-client-id"
client_secret = "your-client-secret"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]
register_user = true
```

### Microsoft Entra ID (Azure AD)

```toml
[global.oauth]
enable = true
issuer = "https://login.microsoftonline.com/{tenant-id}/v2.0"
client_id = "your-application-id"
client_secret = "your-client-secret"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]
register_user = true
```

### GitLab

```toml
[global.oauth]
enable = true
issuer = "https://gitlab.com"
client_id = "your-application-id"
client_secret = "your-client-secret"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]
register_user = true
# GitLab may require manual endpoint configuration
authorization_endpoint = "https://gitlab.com/oauth/authorize"
token_endpoint = "https://gitlab.com/oauth/token"
userinfo_endpoint = "https://gitlab.com/oauth/userinfo"
```

## Using Environment Variables

You can also configure OAuth using environment variables:

```bash
TUWUNEL_OAUTH__ENABLE=true
TUWUNEL_OAUTH__ISSUER="https://accounts.google.com"
TUWUNEL_OAUTH__CLIENT_ID="your-client-id"
TUWUNEL_OAUTH__CLIENT_SECRET="your-client-secret"
TUWUNEL_OAUTH__REDIRECT_URI="https://matrix.example.com/_matrix/client/v3/login/sso/callback"
```

## Security Best Practices

1. **Never commit secrets**: Keep your `client_secret` out of version control
2. **Use HTTPS**: Always use HTTPS in production
3. **Validate redirect URIs**: Ensure redirect URIs exactly match what's configured
4. **Rotate secrets regularly**: Change your client secret periodically
5. **Monitor access**: Keep logs of OAuth authentication attempts
6. **Secure your config file**: Set appropriate file permissions (e.g., `chmod 600`)

## Testing Your Configuration

After configuration, test your OAuth setup:

1. Start Tuwunel with the new configuration
2. Use a Matrix client that supports SSO (like Element)
3. Try logging in with SSO
4. Check Tuwunel logs for any errors
5. Verify the user is created correctly

## Troubleshooting

See the [OAuth documentation](../oauth.md) for detailed troubleshooting steps.
