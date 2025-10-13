# MSC3861 Implementation Summary

## What is MSC3861?

MSC3861 defines native integration between Matrix homeservers and OpenID Connect (OIDC) providers, enabling:

- **Unified Authentication**: OAuth provider handles all authentication
- **Account Management**: Direct integration with OAuth provider account pages
- **Better Client Experience**: Seamless SSO for OIDC-aware Matrix clients
- **Native OIDC Flow**: Matrix adopts OAuth 2.0/OIDC as first-class authentication

## Implemented Features

### 1. Configuration (`src/core/config/mod.rs`)

Added MSC3861-specific configuration options:

```rust
pub struct OAuthConfig {
    // ... existing OAuth config ...
    
    /// MSC3861: Account management URL
    pub account_management_url: Option<String>,
    
    /// MSC3861: Enable experimental OAuth delegation mode
    pub experimental_msc3861: bool,
}
```

**Usage:**
```toml
[global.oauth]
account_management_url = "https://accounts.google.com/myaccount"
experimental_msc3861 = true
```

### 2. Account Management Endpoint (`src/api/client/oidc.rs`)

Implemented the MSC3861 account management endpoint:

**Endpoint:** `GET /_matrix/client/unstable/org.matrix.msc3861/account_management`

**Response:**
```json
{
  "account_management_uri": "https://provider.com/account",
  "account_management_actions_supported": [
    "org.matrix.profile",
    "org.matrix.sessions_list",
    "org.matrix.session_view",
    "org.matrix.session_end"
  ]
}
```

This tells Matrix clients where users can manage their accounts at the OAuth provider.

### 3. Enhanced OIDC Discovery

Updated `.well-known/openid-configuration` to include account management:

```json
{
  "issuer": "https://provider.com",
  "authorization_endpoint": "...",
  "token_endpoint": "...",
  "account_management_uri": "https://provider.com/account",
  "account_management_actions_supported": [...]
}
```

### 4. OAuth Issuer Discovery (MSC2965)

Implemented MSC2965 for OIDC-aware client discovery:

**Endpoint:** `GET /_matrix/client/unstable/org.matrix.msc2965/auth_issuer`

**Response:**
```json
{
  "issuer": "https://provider.com",
  "account": "matrix.example.com"
}
```

### 5. Enhanced .well-known/matrix/client

Updated `GET /.well-known/matrix/client` to include OAuth authentication info:

```json
{
  "m.homeserver": {
    "base_url": "https://matrix.example.com"
  },
  "org.matrix.msc2965.authentication": {
    "issuer": "https://provider.com",
    "account": "matrix.example.com"
  }
}
```

This enables clients to discover OAuth authentication automatically.

### 6. Router Integration

All MSC3861-related endpoints are now routed in `src/api/router.rs`:

```rust
// MSC2965: OAuth issuer discovery
.route("/_matrix/client/unstable/org.matrix.msc2965/auth_issuer", 
       get(client::oidc::oauth_issuer_route))

// MSC3861: Account management information
.route("/_matrix/client/unstable/org.matrix.msc3861/account_management", 
       get(client::oidc::msc3861_account_management_route))

// OIDC discovery endpoint
.route("/.well-known/openid-configuration", 
       get(client::oidc::oidc_discovery_route))
```

## Usage Example

### Configuration

```toml
[global.oauth]
enable = true
issuer = "https://accounts.google.com"
client_id = "123456.apps.googleusercontent.com"
client_secret = "your-secret"
redirect_uri = "https://matrix.example.com/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]

# MSC3861 specific
account_management_url = "https://accounts.google.com/myaccount"
experimental_msc3861 = true
```

### Client Flow

1. **Discovery**: Client fetches `/.well-known/matrix/client`
   - Discovers OAuth issuer from `org.matrix.msc2965.authentication`

2. **Authentication**: Client uses OAuth SSO flow
   - Redirects to `/_matrix/client/v3/login/sso/redirect`
   - OAuth provider authenticates user
   - Callback to `/_matrix/client/v3/login/sso/callback`
   - Client receives Matrix access token

3. **Account Management**: Client can link to provider's account page
   - Fetches `/_matrix/client/unstable/org.matrix.msc3861/account_management`
   - Gets `account_management_uri`
   - Shows "Manage Account" button linking to OAuth provider

## Benefits

### For Users
- Single sign-on across Matrix and other services
- Centralized account management
- Familiar login experience
- No separate Matrix password to remember

### For Administrators
- Leverage existing identity infrastructure
- Centralized user management
- OAuth provider handles security (2FA, password policies)
- Reduced authentication complexity

### For Developers
- Standard OAuth 2.0/OIDC implementation
- Clear discovery mechanisms
- Well-defined endpoints
- Compatible with OIDC-aware clients

## Compatibility

### Supported MSC Features

- ✅ MSC2965: OIDC-aware clients (auth issuer discovery)
- ✅ MSC3861: Native OIDC integration (account management)
- ✅ Account management URI
- ✅ Account management actions
- ✅ Enhanced .well-known discovery
- ✅ OIDC metadata with MSC3861 extensions

### Client Support

Clients that support MSC2965/MSC3861:
- Element (with experimental features enabled)
- Other OIDC-aware Matrix clients

### Provider Requirements

OAuth/OIDC providers should support:
- Authorization Code flow
- OIDC discovery (.well-known/openid-configuration)
- Userinfo endpoint
- Standard claims (sub, name, email, etc.)

## Testing

To test MSC3861 implementation:

1. **Check Discovery**:
   ```bash
   curl https://your-server/.well-known/matrix/client
   ```
   Should include `org.matrix.msc2965.authentication`

2. **Check OAuth Issuer**:
   ```bash
   curl https://your-server/_matrix/client/unstable/org.matrix.msc2965/auth_issuer
   ```

3. **Check Account Management**:
   ```bash
   curl https://your-server/_matrix/client/unstable/org.matrix.msc3861/account_management
   ```

4. **Check OIDC Discovery**:
   ```bash
   curl https://your-server/.well-known/openid-configuration
   ```
   Should include `account_management_uri`

## Future Enhancements

Potential future improvements:

- Full OAuth delegation mode (all Matrix operations via OAuth)
- Token introspection endpoint
- Dynamic client registration
- Multiple OAuth provider support
- Advanced account linking
- Refresh token support

## References

- [MSC2965: OIDC-aware clients](https://github.com/matrix-org/matrix-spec-proposals/pull/2965)
- [MSC3861: Matrix + OIDC](https://github.com/matrix-org/matrix-spec-proposals/pull/3861)
- [OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)
- [OpenID Connect Core](https://openid.net/specs/openid-connect-core-1_0.html)

## Summary

This implementation provides a solid foundation for MSC3861-compliant OAuth 2.0/OIDC integration in Tuwunel. All core endpoints are implemented and routed, configuration is complete, and documentation covers the new features. Users can now experience native OAuth integration with their Matrix homeserver, with seamless account management through their OAuth provider.
