# OAuth 2.0 Implementation Status

This document tracks the implementation status of OAuth 2.0 / OIDC support in Tuwunel.

## Implemented Features ✅

### Configuration
- [x] OAuth 2.0 configuration structure in `src/core/config/mod.rs`
- [x] Support for all major OAuth providers (Google, Keycloak, Auth0, etc.)
- [x] Configurable scopes, claims, and endpoints
- [x] Auto-discovery support for OIDC providers
- [x] Environment variable support

### Core OAuth Flow
- [x] SSO login type in login types endpoint
- [x] OAuth token exchange implementation
- [x] User info fetching from OAuth provider
- [x] Automatic user creation from OAuth claims
- [x] Display name mapping from OAuth claims

### API Endpoints
- [x] OIDC discovery endpoint structure (`src/api/client/oidc.rs`)
- [x] OAuth redirect URL generation
- [x] Token exchange functions
- [x] User info validation

### Documentation
- [x] Comprehensive OAuth setup guide (`docs/oauth.md`)
- [x] Provider-specific examples (`docs/configuration/oauth-examples.md`)
- [x] Security best practices
- [x] Troubleshooting guide
- [x] Documentation index updated

## Partially Implemented 🚧

### Routing
- [ ] OAuth redirect endpoint routing (need to add to router.rs)
- [ ] OAuth callback endpoint routing (need to add to router.rs)
- [ ] OIDC discovery endpoint routing (need to add to router.rs)

### Token Management
- [ ] OAuth state parameter storage and validation
- [ ] Session management for OAuth flow
- [ ] Token refresh support
- [ ] Token revocation endpoint

## Not Yet Implemented ❌

### Advanced Features
- [ ] Concurrent multi-provider support (currently single active provider)
- [ ] Account linking for existing users
- [ ] PKCE (Proof Key for Code Exchange) support
- [ ] Token introspection endpoint
- [ ] Dynamic client registration
- [ ] Advanced custom claim mapping rules (currently supports basic predefined claims)
- [ ] OAuth provider metadata caching

### Testing
- [ ] Unit tests for OAuth flow
- [ ] Integration tests with mock OAuth provider
- [ ] End-to-end tests with real providers
- [ ] Security vulnerability testing

### Security Enhancements
- [ ] Rate limiting on OAuth endpoints
- [ ] Brute force protection
- [ ] CSRF token validation (state parameter)
- [ ] Nonce parameter support
- [ ] Token binding

## Usage

To use OAuth 2.0 authentication, add the following to your `tuwunel.toml`:

```toml
[global.oauth]
enable = true
issuer = "https://your-oauth-provider.com"
client_id = "your-client-id"
client_secret = "your-client-secret"
redirect_uri = "https://your-matrix-domain/_matrix/client/v3/login/sso/callback"
scopes = ["openid", "profile", "email"]
```

See `docs/oauth.md` for detailed configuration instructions.

## Known Limitations

1. **Single Active Provider**: While configuration supports multiple provider types (Google, Keycloak, etc.), only one provider can be active at a time
2. **Manual Routing**: OAuth endpoints need to be manually added to the router
3. **No Account Linking**: Existing users cannot link their OAuth account
4. **State Management**: OAuth state parameter is generated but not validated
5. **Session Storage**: OAuth sessions are not persisted across server restarts
6. **Basic Claim Mapping**: Only predefined claim fields (sub, name, email) are supported; custom mapping rules not yet implemented

## Future Work

1. Complete routing integration
2. Add comprehensive test coverage
3. Implement PKCE for enhanced security
4. Support multiple OAuth providers
5. Add account linking functionality
6. Implement proper session management
7. Add refresh token support
8. Performance optimization and caching

## Contributing

Contributions are welcome! If you'd like to help complete the OAuth implementation:

1. Check the "Not Yet Implemented" section above
2. Open an issue to discuss your planned changes
3. Submit a pull request with tests
4. Update this document with your changes

## Security Notes

⚠️ **Important**: This is a new implementation. Before using in production:

1. Thoroughly test with your OAuth provider
2. Review all security implications
3. Enable HTTPS (required for OAuth)
4. Keep client secrets secure
5. Monitor for unusual authentication patterns
6. Consider additional security layers (2FA at provider level)

## References

- [Matrix Spec: SSO](https://spec.matrix.org/latest/client-server-api/#single-sign-on)
- [OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)
- [OpenID Connect Core](https://openid.net/specs/openid-connect-core-1_0.html)
- [MSC2858: Multiple SSO Identity Providers](https://github.com/matrix-org/matrix-spec-proposals/pull/2858)
- [MSC3861: Matrix + OIDC](https://github.com/matrix-org/matrix-spec-proposals/pull/3861)
