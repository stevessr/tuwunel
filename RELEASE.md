# Tuwunel 1.6.2

May 8, 2026

We have started a specification compliance campaign which will continue over the next several releases until synced with 1.18 (or 1.19 if it takes that long). Current status will be kept up to date in our [documentation](https://matrix-construct.github.io/tuwunel/development/compliance/msc.html).

### New Features & Enhancements

- **Policy server support** (MSC4284) ships; two new config keys: `enable_policy_servers` and `policy_server_request_timeout`.

- **Account locking and suspension** (MSC3939, MSC4323, and MSC3823), plus an admin command to mass-reject pending invites.

- MSC2732 Olm fallback keys are implemented and re-issued on each subsequent claim, so clients keep receiving after key exhaustion.

- MSC4380 invite blocking (partial).

- MSC4406 `sender_ignored` on single-event endpoints.

- MSC4383 `/versions` discovery.

- MSC4260 user reports.

- MSC4373 incoming-EDU types over federation.

- MSC4168 `m.space.*` state copied on upgrade.

- MSC4169 backwards-compatible redactions on send.

- MSC3905 local-only users namespace matching for appservices.

- MSC4025 partial erase on `/deactivate`.

- MSC3391 account-data deletion.

- MSC4361 non-federating member auth rules.

- MSC4190 appservices now skip UIA on cross-signing key replacement.

- MSC4254 OIDC revoke handler is tightened across request shape, error codes, and provider lookup.

- MSC4175 Timezone-key routes have been updated to the stabilized form.

- Thanks to @DBendit who opened (#316): a complete list of MSCs Tuwunel supports is now in the docs.

- @dasha-uwu shipped cleanups: appservice file filter, conditional admin lookup, two-member room naming, thumbnail logging, remote media ids.

### Bug Fixes

- Sliding-sync `bump_stamp` is graciously fixed by @lhjt in (#449), so Element X and other clients move rooms in the sidebar on new activity.

- Thanks to @humemm for (#448), where OAuth responses returning `expires_at` as a Unix timestamp tripped login; the upstream DTO is now decoupled.

- Tip of the hat to @maxrdz for the NGINX root-domain delegation example in (#446), with default port and a resilient `$backend` indirection.

- State resolution corrections: knock auth v7-9 (aea509fe5), auth-difference (631c51aa8), mainline 0 (82132eec4), v12 bootstrap-join (aaa6a1a55). A few were [upstreamed to Ruma](https://github.com/ruma/ruma/pull/2480).

- A long-standing `/sync` concurrency heisenbug (b1ac65b60), originally introduced in Conduit and made slightly worse by optimizations which took place in v1.3.0, has finally been zapped.
