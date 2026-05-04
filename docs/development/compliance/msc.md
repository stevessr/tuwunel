# Tuwunel MSC Implementation Status

## Columns

- **Inv**: inventory status (matrix-spec-proposals state). One of
  `merged`, `open`, `closed`, `unknown`.
- **Status**: 🟢 `yes` / 🟡 `partial` / ⭕️ `no` / ⚫ `n/a`
- **Correct/Impl**: two absolute percentages of the total proposal,
  e.g. `70/80`. Correct is the share of the proposal's requirements
  Tuwunel adheres to correctly; Impl is the share that has any code
  path attempting adherence. By definition Correct <= Impl. Either
  may be `?`. Proposals are loosely normative, so this is NOT just
  MUST/SHOULD: every requirement-shaped statement counts ("the server
  returns X", "this field is added to Y", etc.).
- **Conf**: `H` / `M` / `L` / `?`. Confidence in the row, not in the
  implementation.

## Counts

- 🟢 `yes`: 209
- 🟡 `partial`: 87
- ⭕️ `no`: 428
- ⚫ `n/a`: 289

### Status by inventory bucket

| Inv | yes | partial | no | n/a | total |
|---|---|---|---|---|---|
| merged | 145 | 35 | 19 | 57 | 256 |
| open | 56 | 51 | 373 | 172 | 652 |
| closed | 8 | 1 | 36 | 52 | 97 |
| unknown | 0 | 0 | 0 | 8 | 8 |

## Merged

Sorted by MSC number, highest first.

| MSC | Status | Correct/Impl | Conf | Title | Note |
|---|---|---|---|---|---|
| MSC4381 | ⚫ | ?/? | M | Remove plaintext sender key | Removal of plaintext sender_key is client-side; server is opaque |
| MSC4380 | 🟡 | 70/70 | H | Invite blocking | phase A landed (invite-creating endpoints gated, M_INVITE_BLOCKED 403); phase... |
| MSC4376 | 🟢 | 100/100 | H | Remove /v1/send_join and /v1/send_leave | v1 send_join and v1 send_leave routes are not registered |
| MSC4356 | ⚫ | ?/? | H | Recently used emoji | Pure client-side account data convention; no server work |
| MSC4341 | ⭕️ | 0/0 | H | Support for RFC 8628 Device Authorization Grant | OAuth Device Authorization Grant (RFC 8628) not advertised |
| MSC4335 | ⭕️ | 0/0 | H | M_USER_LIMIT_EXCEEDED error code | M_USER_LIMIT_EXCEEDED error code not used |
| MSC4326 | 🟢 | 100/100 | H | Device masquerading for appservices | appservice query device_id asserted; M_UNKNOWN_DEVICE-equivalent on missing |
| MSC4323 | ⭕️ | 0/0 | H | User suspension &amp; locking endpoints | client admin suspend/lock endpoints not implemented |
| MSC4313 | ⚫ | ?/? | H | Require HTML `&lt;ol&gt;` `start` Attribute support | client HTML rendering requirement; not applicable to homeserver |
| MSC4312 | 🟢 | 90/100 | H | Resetting cross-signing keys in the OAuth world | cross-signing reset issues m.oauth flow with account-management URL |
| MSC4311 | 🟡 | 0/? | M | Ensuring the create event is available on invites | complement: 0p/1f |
| MSC4307 | 🟢 | 100/100 | H | Validate that `auth_events` are in the correct room | auth_event room_id mismatch rejected |
| MSC4304 | 🟢 | 90/100 | H | Room Version 12 | V12 supported as stable; default is V11 |
| MSC4297 | 🟢 | 100/100 | H | State Resolution v2.1 | src/service/rooms/state_res/resolve.rs:257 conflicted state subgraph; tests pass |
| MSC4291 | 🟡 | 80/90 | H | Room IDs as hashes of the create event | hydra.11 room id format and auth rules in event_auth, pdu format checks |
| MSC4289 | 🟢 | 100/100 | H | Explicitly privilege room creators | src/service/tests/state_res/fixtures/MSC4297-problem-A/pdus-hydra.json:5; com... |
| MSC4287 | ⚫ | ?/? | H | Sharing key backup preference between clients | client-side account data for key backup preference |
| MSC4284 | ⭕️ | 0/0 | H | Policy Servers | no policy server /sign, no m.room.policy state handling |
| MSC4277 | 🟡 | 30/40 | M | Harmonizing the reporting endpoints | event and room report endpoints exist; user report endpoint absent |
| MSC4268 | ⚫ | ?/? | H | Sharing room keys for past messages | client-only E2EE key sharing; server only relays to-device and stores media |
| MSC4267 | 🟢 | 100/100 | H | Automatically forgetting rooms on leave | forget_forced_upon_leave config honored on Leave or Ban; capability advertised |
| MSC4260 | 🟢 | 100/100 | H | Reporting users (Client-Server API) | src/api/client/report.rs:63; admin notification, 404 M_NOT_FOUND on unknown u... |
| MSC4254 | 🟢 | 100/100 | H | Usage of [RFC7009] Token Revocation for Matrix client logout | src/api/oidc/revoke.rs:37; RFC7009 form-urlencoded; revokes both tokens; 200 ... |
| MSC4239 | 🟢 | 100/100 | H | Room version 11 as the default room version | default_default_room_version = V11 |
| MSC4230 | 🟢 | 100/100 | H | 'Animated' flag for images | event-only; passthrough; merged in spec |
| MSC4225 | 🟡 | 50/50 | M | Specification of an order in which one-time-keys should be issued | OTKs scanned in lexicographic key-id order; not strict upload order |
| MSC4222 | ⭕️ | 0/0 | H | Adding `state_after` to `/sync` | MSC4222 commit lives only on the `4222` feature branch, not dev |
| MSC4213 | 🟢 | 90/90 | H | Remove `server_name` parameter | join/knock use via; server_name still accepted via Ruma fallback |
| MSC4210 | 🟢 | 100/100 | H | Remove legacy mentions | deprecated mention push rules removed at /pushrules read time |
| MSC4191 | 🟡 | 50/80 | M | Account management for OAuth 2.0 API | metadata wired but action names diverge from MSC |
| MSC4190 | 🟢 | 100/100 | H | Device management for application services | appservices with device_management can create, update, delete devices without... |
| MSC4189 | 🟢 | 80/100 | M | Allowing guests to access uploaded media | guest tokens accepted on authenticated media routes |
| MSC4183 | ⚫ | ?/? | H | Additional Error Codes for submitToken endpoints | identity service API; Tuwunel is not an IS |
| MSC4180 | 🟢 | 100/100 | H | Add a stable flag to MSC3916 | stable feature flag for MSC3916 advertised |
| MSC4178 | 🟡 | 10/30 | M | Error codes for requestToken | requestToken returns ThreepidDenied; not the new codes |
| MSC4175 | 🟢 | 100/100 | H | Profile field for user time zone | timezone PUT/DELETE/GET routes; m.tz aliased in profile and over federation |
| MSC4170 | 🟢 | 100/100 | M | 403 error responses for profile APIs | profile lookup unrestricted; MUST minimum satisfied |
| MSC4169 | 🟢 | 100/100 | H | Backwards-compatible redaction sending using `/send` | src/api/client/send.rs:42; lifts content.redacts into PduBuilder.redacts; adv... |
| MSC4163 | 🟢 | 100/100 | H | Make ACLs apply to EDUs | ACLs applied on receipt and typing EDUs |
| MSC4159 | ⚫ | ?/? | H | Remove the deprecated name attribute on HTML anchor elements | client-side HTML rendering recommendation |
| MSC4156 | 🟢 | 100/100 | H | Migrate `server_name` to `via` | via parameter handled via Ruma |
| MSC4153 | ⚫ | ?/? | H | Exclude non-cross-signed devices | client-side cross-signing enforcement and to-device filtering |
| MSC4151 | 🟢 | 100/100 | H | Reporting rooms (Client-Server API) | POST /rooms/{roomId}/report implemented and routed |
| MSC4147 | ⚫ | ?/? | H | Including device keys with Olm-encrypted to-device messages | sender_device_keys in Olm plaintext is client-side |
| MSC4142 | ⚫ | ?/? | H | Remove unintentional intentional mentions in replies | client-side guidance for m.mentions in replies |
| MSC4138 | 🟢 | 100/100 | H | Update allowed HTTP methods in CORS responses | CORS METHODS list includes HEAD and PATCH; excludes CONNECT/TRACE |
| MSC4133 | 🟢 | 90/90 | H | Extending User Profile API with Custom Key:Value Pairs | GET/PUT/DELETE profile field endpoints routed at unstable prefix |
| MSC4132 | ⚫ | ?/? | H | Deprecate Linking to an Event Against a Room Alias. | deprecation of event-on-room-alias URIs is client-only |
| MSC4126 | 🟢 | 100/100 | H | Deprecation of query string auth | deprecation of query string auth; server still accepts both |
| MSC4115 | ⭕️ | 0/0 | H | membership metadata on events | unsigned.membership not populated on events served to clients |
| MSC4077 | ⚫ | ?/? | H | Improved process for handling deprecated HTML features | process MSC for HTML feature deprecation; no server work |
| MSC4041 | 🟢 | 90/90 | M | Use http header Retry-After to enable library-assisted retry handling | Ruma error type emits Retry-After header for LimitExceeded responses. |
| MSC4040 | 🟢 | 100/100 | H | Update SRV service name to IANA registration | Tuwunel queries _matrix-fed first then falls back to _matrix. |
| MSC4026 | 🟢 | 80/90 | M | Allow /versions to optionally accept authentication | versions endpoint accepts optional auth via Ruma |
| MSC4025 | 🟡 | 50/50 | H | Local user erasure requests | phase A landed (account-data wipe); phase B (per-event visibility gate) deferred |
| MSC4010 | 🟢 | 100/100 | H | Push rules and account data | m.push_rules and m.fully_read rejected on /account_data |
| MSC4009 | 🟢 | 100/100 | H | Expanding the Matrix ID grammar to enable E.164 IDs | E.164 + character allowed via Ruma localpart validation |
| MSC3989 | 🟢 | 100/100 | H | Redact `origin` property on events | V11 redaction drops origin via Ruma RedactionRules |
| MSC3987 | 🟢 | 90/90 | H | Push actions clean-up | unknown push actions ignored as no-ops |
| MSC3981 | 🟢 | 100/100 | H | `/relations` recursion | /relations recurse parameter implemented with depth 3 |
| MSC3980 | ⭕️ | 0/0 | M | Dotted Field Consistency | event_fields filter escaping not enforced |
| MSC3970 | 🟢 | 90/100 | H | Scope transaction IDs to devices | transaction IDs scoped per (user, device, txn_id) |
| MSC3967 | 🟢 | 100/100 | H | Do not require UIA when first uploading cross signing keys | keys/device_signing/upload skips UIA when user has no existing cross-signing ... |
| MSC3966 | 🟢 | 100/100 | H | `event_property_contains` push rule condition | event_property_contains supported via Ruma push conditions |
| MSC3958 | 🟢 | 100/100 | H | Suppress notifications from message edits | SuppressEdits push rule provided via Ruma server_default ruleset |
| MSC3952 | 🟢 | 80/90 | M | Intentional Mentions | Intentional mentions push rules ride on Ruma server_default; flag advertised. |
| MSC3943 | 🟢 | 100/100 | H | Partial joins to nameless rooms should include heroes' memberships. | send_join partial-state response includes hero memberships and their auth chains |
| MSC3939 | ⭕️ | 0/0 | H | Account locking | Account locking (M_USER_LOCKED, soft_logout) not implemented. |
| MSC3938 | 🟢 | 80/80 | M | Remove deprecated `keyId` parameters from `/keys` endpoints | New /key/v2/server (no keyId) implemented; deprecated form retained for compat. |
| MSC3930 | 🟡 | 0/? | M | Polls push rules/notifications | complement: 0p/2f |
| MSC3925 | 🟡 | 50/50 | M | m.replace aggregation with full event | Tuwunel doesn't replace content (good) but also lacks bundled m.replace aggre... |
| MSC3923 | ⚫ | 0/0 | H | Bringing Matrix into the IETF process | Spec-process MSC about IETF coordination; no homeserver code. |
| MSC3916 | 🟢 | 90/100 | H | Authentication for media access, and new endpoint names | New /client/v1/media and /federation/v1/media auth endpoints implemented. |
| MSC3905 | 🟢 | 100/100 | H | Application services should only be interested in local users | src/service/appservice/append.rs:66; local-user guard at the three event-inte... |
| MSC3882 | 🟢 | 90/100 | H | Allow an existing session to sign in a new session | POST /login/get_token implemented with UIA |
| MSC3873 | 🟢 | 100/100 | H | event_match dotted keys | dotted-key escape semantics handled in ruma flattened JSON |
| MSC3861 | 🟡 | 60/70 | M | Next-generation auth for Matrix, based on OAuth 2.0/OIDC | OIDC core endpoints implemented but not advertised as MSC3861 itself |
| MSC3860 | 🟡 | 30/40 | M | Media Download Redirects | forwards allow_redirect to remote fetch but does not emit own redirect |
| MSC3856 | 🟡 | 40/60 | M | Threads List API | GET /threads route present but participated filter and latest-event order mis... |
| MSC3844 | 🟢 | 100/100 | H | Remove "Mjolnir" (policy room) sharing mechanism | removal of unused Mjolnir share endpoint; Tuwunel never implemented it |
| MSC3828 | 🟢 | 100/100 | H | Content Repository Cross Origin Resource Policy (CORP) Headers | media endpoints return Cross-Origin-Resource-Policy: cross-origin |
| MSC3827 | 🟢 | 100/100 | H | Filtering of `/publicRooms` by room type | /publicRooms supports room_types filter and returns room_type |
| MSC3824 | 🟡 | 60/60 | M | OAuth 2.0 API aware clients | oauth_aware_preferred set in /login; SSO redirect action param ignored |
| MSC3823 | ⭕️ | 0/0 | H | Account Suspension | no M_USER_SUSPENDED errcode or suspension behavior |
| MSC3821 | 🟢 | 90/100 | H | Update redaction rules, again | redact_in_place uses Ruma RedactionRules.V11 with keep third_party_invite.signed |
| MSC3820 | 🟢 | 90/100 | H | Room Version 11 | v11 stable; redaction and auth rules dispatch via Ruma RoomVersionRules |
| MSC3818 | 🟢 | 100/100 | H | Copy room type on upgrade | upgrade reuses old m.room.create content; type preserved by default |
| MSC3816 | 🟡 | 20/40 | M | Clarify Thread Participation | BundledThread.current_user_participated hardcoded true on first reply only |
| MSC3787 | 🟡 | 71/? | H | Allowing knocks to restricted rooms | complement: 34p/14f |
| MSC3786 | 🟢 | 100/100 | H | Add a default push rule to ignore `m.room.server_acl` events | server_acl predefined push rule via Ruma defaults |
| MSC3783 | ⚫ | 0/0 | H | Fixed base64 for SAS verification | SAS MAC scheme is client-to-client crypto |
| MSC3773 | ⭕️ | 0/10 | H | Notifications for threads | sync returns empty unread_thread_notifications map |
| MSC3771 | ⭕️ | 0/10 | H | Read receipts for threads | thread_id discarded; receipts always Unthreaded |
| MSC3765 | 🟡 | 30/40 | M | Rich text in room topics | topic_block accepted via Ruma; createRoom only writes plain topic |
| MSC3758 | 🟢 | 90/100 | H | Add `event_property_is` push rule condition kind | event_property_is dispatched via Ruma Ruleset::get_actions |
| MSC3743 | 🟢 | 90/100 | H | Standardized error response for unknown endpoints | M_UNRECOGNIZED 404/405 fallback wired in router |
| MSC3715 | 🟢 | 100/100 | H | Add a pagination direction parameter to `/relations` | dir parameter on /relations is parsed and used |
| MSC3706 | 🟢 | 90/100 | H | Extensions to `/_matrix/federation/v2/send_join/{roomId}/{eventId}` for parti... | send_join supports omit_members, members_omitted, servers_in_room |
| MSC3700 | ⚫ | 0/0 | M | Deprecate plaintext sender_key | client-side ignoring of sender_key/device_id; server is transparent |
| MSC3676 | ⚫ | 0/0 | M | Transitioning away from reply fallbacks. | client-side reply-fallback transition rules; no server gate |
| MSC3667 | 🟢 | 100/100 | H | Enforce integer power levels | integer_power_levels enforced via RoomVersionRules from V10+ |
| MSC3666 | ⭕️ | 0/0 | H | Bundled aggregations for server side search | search results do not include bundled aggregations |
| MSC3604 | 🟢 | 100/100 | H | Room Version 10 | V10 supported; integer_power_levels and knock_restricted enforced |
| MSC3589 | 🟢 | 100/100 | H | Room version 9 as a default | default_room_version defaults to V11 (exceeds V9) |
| MSC3582 | 🟢 | 100/100 | H | Remove m.room.message.feedback | feedback removal; tuwunel never produces or dispatches on m.room.message.feed... |
| MSC3567 | 🟢 | 100/100 | H | Allow requesting events from the start/end of the room history | from is optional; defaults to start/end based on dir |
| MSC3550 | 🟡 | 50/50 | M | Add HTTP 403 to possible profile lookup responses | federation 403 returned; client /profile still 404 only |
| MSC3442 | 🟢 | 100/100 | H | move the `prev_content` key to `unsigned` | prev_content placed under unsigned in created/appended PDUs |
| MSC3440 | 🟢 | 80/80 | H | MSC3440 Threading via `m.thread` relation | [→ MSC3856] thread bundling, /threads, /relations with rel_type filter |
| MSC3419 | ⭕️ | 0/0 | M | Guest State Events | guests still cannot send arbitrary state events |
| MSC3383 | 🟢 | 100/100 | H | Include destination in X-Matrix Auth Header | X-Matrix destination field validated on inbound federation |
| MSC3381 | 🟡 | 0/? | M | Chat Polls | complement: 0p/2f |
| MSC3375 | 🟢 | 100/100 | H | Room Version 9 | room v9 stable; redaction keeps join_authorised_via_users_server |
| MSC3316 | 🟢 | 100/100 | H | Proposal to add timestamp massaging to the spec | appservice ts honored on /send and /state |
| MSC3291 | ⚫ | ?/? | H | Muting in VoIP calls | server passes call events opaquely; ruma has the type |
| MSC3289 | 🟢 | 100/100 | H | Room Version 8 | room v8 listed stable; restricted join rule auth implemented |
| MSC3288 | ⭕️ | 0/0 | H | Add room type to `/_matrix/identity/v2/store-invite` API | no /_matrix/identity/v2/store-invite call site; no room_type forwarded |
| MSC3283 | 🟡 | 30/30 | M | Expose enable_set_displayname, enable_set_avatar_url and enable_3pid_changes ... | only m.3pid_changes capability set; set_displayname/set_avatar_url absent |
| MSC3267 | 🟡 | 50/50 | M | reference relationships | reference relations queryable via /relations; no m.relations bundling |
| MSC3266 | 🟢 | 100/100 | H | Room Summary API | summary endpoint routed at unstable and (via Ruma) stable paths |
| MSC3231 | 🟢 | 100/100 | H | Token Authenticated Registration | registration token UIA + validity endpoint implemented |
| MSC3226 | ⚫ | ?/? | H | Per-room spell check | per-room spellcheck language is account_data; no server logic |
| MSC3173 | 🟢 | 100/100 | H | Expose stripped state events to any potential joiner | summary_stripped includes recommended events incl create |
| MSC3122 | ⚫ | ?/? | H | Deprecate starting key verifications without requesting first | client-only deprecation of to-device verification start |
| MSC3083 | 🟢 | 100/100 | H | Restricting room membership based on membership in other rooms | restricted_join_rule auth via RoomVersionRules; v8/v9 |
| MSC3077 | ⚫ | 0/0 | H | Support for multi-stream VoIP | merged; sdp_stream_metadata is event content |
| MSC3069 | 🟢 | 80/100 | M | Allow guests to use /account/whoami | whoami returns is_guest; uses is_deactivated heuristic |
| MSC3030 | 🟡 | 60/80 | H | Jump to date API endpoint | client and federation timestamp_to_event handlers; no remote fallback when lo... |
| MSC2998 | 🟢 | 100/100 | H | Room Version 7 | V7 listed in STABLE_ROOM_VERSIONS; full knock support present |
| MSC2967 | 🟢 | 80/90 | H | API scopes | urn:matrix:client:device:* scope honored; api:* scope advertised |
| MSC2966 | 🟢 | 80/90 | H | Usage of OAuth 2.0 Dynamic Client Registration in Matrix | dynamic client registration endpoint |
| MSC2965 | 🟢 | 90/100 | H | OAuth 2.0 Authorization Server Metadata discovery | auth_issuer and auth_metadata routes return OAuth provider metadata |
| MSC2964 | 🟢 | 90/100 | H | Usage of OAuth 2.0 authorization code grant and refresh token grant | OAuth2 authorize/token/refresh implemented |
| MSC2946 | 🟢 | 90/100 | H | Spaces Summary | client and federation hierarchy endpoints implemented |
| MSC2918 | 🟢 | 90/100 | H | Refresh tokens | /refresh, expires_in_ms, refresh_token in /login and /register |
| MSC2874 | ⚫ | ?/? | H | Single SSSS | client interpretation of SSSS default key; account data passthrough |
| MSC2870 | 🟢 | 100/100 | M | Protect server ACLs from redaction | redaction dispatches on RoomVersionRules.redaction; ruma MSC2870 enabled |
| MSC2867 | 🟢 | 100/100 | M | Marking rooms as unread | client convention; account data type stored generically |
| MSC2858 | 🟢 | 100/100 | H | Multiple SSO Identity Providers | identity_providers in /login flows; /login/sso/redirect/{idpId} routed |
| MSC2844 | 🟢 | 90/90 | H | Using a global version number for the entire specification | src/api/client/versions.rs advertises v1.1 through v1.15 |
| MSC2832 | 🟢 | 100/100 | H | Homeserver -&gt; Application Service authorization header | src/service/appservice/request.rs sends Bearer header and query |
| MSC2801 | ⚫ | ?/? | H | Make it explicit that event bodies are untrusted data | spec note: clients should treat events as untrusted |
| MSC2788 | 🟢 | 100/100 | H | Room version 6 as a default | default_default_room_version is V11 in src/core/config/mod.rs:3676 |
| MSC2781 | ⚫ | ?/? | H | Remove reply fallbacks from the specification | removes client-side reply fallback; client behavior change |
| MSC2778 | 🟢 | 100/100 | H | Providing authentication method for appservice users | src/api/client/session/appservice.rs implements m.login.application_service |
| MSC2774 | ⚫ | ?/? | H | Giving widgets their ID so they can communicate | client widget URL template variable |
| MSC2765 | ⚫ | ?/? | H | Widget avatars | client-side widget definition field |
| MSC2758 | ⚫ | ?/? | H | Common grammar for textual identifiers | meta grammar guideline for future identifiers; not directly implementable |
| MSC2746 | 🟡 | 40/40 | L | Improved Signalling for 1:1 VoIP | Events relayed; no specific server hooks |
| MSC2732 | 🟢 | 100/100 | H | Olm fallback keys | src/api/client/keys/claim_keys.rs:86; upload, claim-fallback, sync-unused-lis... |
| MSC2713 | ⚫ | 0/0 | H | Remove deprecated Identity Service endpoints | Identity Service endpoints; not a homeserver feature |
| MSC2705 | 🟡 | 30/40 | M | Animated thumbnails | animated param accepted; thumbnails always PNG static |
| MSC2702 | 🟢 | 100/100 | H | `Content-Disposition` usage in the media repo | Content-Disposition and inline allowlist enforced for media downloads, thumbn... |
| MSC2701 | 🟢 | 90/90 | M | Media and the `Content-Type` relationship | Optional Content-Type accepted; stored and returned |
| MSC2689 | 🟢 | 100/100 | M | Allow guests to operate in encrypted rooms | Auth treats guests like users; /members open |
| MSC2677 | 🟢 | 80/90 | H | Annotations and Reactions | Duplicate annotation rejected; reactions plumbed |
| MSC2676 | 🟡 | 50/60 | H | Message editing | edits accepted/relayed; no m.replace bundle or new_content apply |
| MSC2675 | 🟡 | 50/60 | H | Serverside aggregations of message relationships | /relations exists; only m.thread bundling, no m.replace bundle |
| MSC2674 | 🟢 | 90/100 | H | Event relationships | relates_to handled in append; rel_type tracked |
| MSC2666 | 🟢 | 90/100 | H | Get rooms in common with another user | src/api/client/unstable.rs:21 GET /unstable/uk.half-shot.msc2666/user/mutual_... |
| MSC2663 | 🟢 | 100/100 | H | Errors for dealing with non-existent push rules | src/api/client/push.rs all 7 endpoints return NotFound |
| MSC2659 | 🟢 | 100/100 | H | Application service ping endpoint | src/api/client/appservice.rs:11 calls AS /_matrix/app/v1/ping |
| MSC2630 | ⚫ | ?/? | M | Checking public keys in SAS verification | Client SAS verification crypto; server transports key.verification events |
| MSC2611 | 🟢 | 100/100 | H | Remove `m.login.token` User-Interactive Authentication type from the specific... | AuthType::Token UIAA not advertised; m.login.token login is unrelated |
| MSC2610 | 🟢 | 100/100 | H | Remove `m.login.oauth2` User-Interactive Authentication type from the specifi... | AuthType::OAuth2 not advertised; only Password/Sso/Jwt flows |
| MSC2604 | ⚫ | ?/? | M | Parameters for Login Fallback | Client login fallback HTML page; Tuwunel does not serve /login/fallback |
| MSC2582 | ⚫ | ?/? | M | Remove `mimetype` from `EncryptedFile` object | Removes mimetype example from spec; pure spec/client cleanup |
| MSC2557 | ⚫ | ?/? | M | Clarifications on spoilers | Client-only spoiler rendering clarification |
| MSC2540 | 🟡 | 20/40 | M | Stricter event validation: JSON compliance | ruma exposes strict_canonical_json flag; Tuwunel does not enforce floats reje... |
| MSC2530 | ⚫ | ?/? | M | Body field as media caption | Client rendering of body+filename for media msgtypes |
| MSC2526 | 🟢 | 100/100 | H | Add ability to delete key backups | src/api/client/backup.rs:134 delete_backup_version_route |
| MSC2472 | ⚫ | ?/? | M | Symmetric SSSS | Client-side SSSS crypto; server only stores account_data |
| MSC2457 | 🟢 | 100/100 | H | Invalidating devices during password modification | src/api/client/account.rs:41 honors body.logout_devices |
| MSC2454 | 🟢 | 90/90 | H | User-Interactive Authentication for SSO-backed homeserver | src/api/router/auth/uiaa.rs:53 sso_flow; sso/uiaa.rs serves fallback |
| MSC2451 | 🟢 | 100/100 | H | Remove the `query_auth` federation endpoint | No /query_auth route registered in src/api/router.rs |
| MSC2432 | 🟢 | 80/90 | M | Updated semantics for publishing room aliases | alt_aliases wired; canonical_alias resolve check; rooms/{}/aliases route present |
| MSC2422 | ⚫ | ?/? | M | Allow `color` as attribute for `&lt;font&gt;` in messages | Client HTML sanitizer change for &lt;font color&gt; |
| MSC2414 | 🟢 | 100/100 | H | Make `reason` and `score` optional for reporting content | reason and score are Option in ruma report types; route accepts both |
| MSC2409 | 🟡 | 70/70 | H | Proposal to send typing, presence and receipts to appservices | typing+receipt EDUs sent to AS; presence not forwarded |
| MSC2403 | 🟢 | 90/90 | H | Add "knock" feature | Knock CS+SS endpoints, sync key, public-rooms join_rule all wired |
| MSC2399 | ⚫ | ?/? | M | Reporting that decryption keys are withheld | Client-only m.room_key.withheld to-device event |
| MSC2367 | 🟢 | 100/100 | H | Allowing Reasons in all Membership Events | reason field handled in invite/leave/kick/ban/unban/join membership routes |
| MSC2366 | ⚫ | ?/? | M | Key verification flow additions: `m.key.verification.ready` and `m.key.verifi... | Client-side verification flow over to-device; server transports |
| MSC2334 | 🟢 | 100/100 | H | [MSC2334](https://github.com/matrix-org/matrix-doc/pull/2334) - Change defaul... | Default room version is V11, well past V5 |
| MSC2324 | ⚫ | ?/? | H | Facilitating early releases of software dependent on spec | Process change about FCP and stable prefixes |
| MSC2320 | ⚫ | ?/? | H | Versions information for identity servers | Identity server endpoint, not homeserver |
| MSC2313 | ⚫ | ?/? | H | Moderation policies as rooms (ban lists) | State events m.policy.rule.*; no homeserver enforcement |
| MSC2312 | ⚫ | ?/? | H | URI scheme for Matrix | Client-side URI scheme; no homeserver endpoint required |
| MSC2290 | ⭕️ | 0/0 | H | Separate Endpoints for Binding Threepids | No /account/3pid/add or /bind handlers; 3PID generally not impl |
| MSC2285 | 🟢 | 90/100 | H | Private read receipts | src/api/client/read_marker.rs handles ReadPrivate via private_read_set |
| MSC2284 | ⚫ | ?/? | H | Making the identity server optional during discovery | Client-side .well-known FAIL_PROMPT behavior |
| MSC2265 | ⚫ | ?/? | M | Proposal for mandating case folding when processing e-mail addresses | Email casefold only relevant inside 3PID code path; 3PID not impl |
| MSC2264 | ⚫ | ?/? | H | Add an unstable feature flag to MSC2140 for clients to detect support | Process amendment to MSC2140 only |
| MSC2263 | ⚫ | ?/? | M | Give homeservers the ability to handle their own 3PID registrations/password ... | 3PID flow not implemented; threepid endpoints return ThreepidDenied |
| MSC2249 | 🟢 | 90/100 | H | Require users to have visibility on an event when submitting reports | src/api/client/report.rs:113 verifies sender is room member; PDU lookup gated |
| MSC2246 | 🟢 | 100/100 | H | Asynchronous media uploads | async media routes wired; create_pending, upload_pending, error codes present |
| MSC2244 | ⭕️ | 0/0 | H | Mass redactions | Single-target redactions only; no array redacts handling |
| MSC2241 | ⚫ | ?/? | M | Key verification in DMs | Client-side verification flow over m.room.message; server passes events trans... |
| MSC2240 | 🟢 | 100/100 | H | Room Version 6 | V6 in STABLE_ROOM_VERSIONS; v6 auth rules and rules engine implemented |
| MSC2230 | ⚫ | 0/0 | M | Store Identity Server in Account Data | client behavior over generic account data; HS already supports account data |
| MSC2229 | ⚫ | 0/0 | H | Allowing 3PID Owners to Rebind | [→ MSC2290] obsoleted by MSC2290; tuwunel disables 3PID |
| MSC2209 | 🟢 | 100/100 | H | Update auth rules to check notifications key in m.room.power_levels | limit_notifications_power_levels enforced for v6+ |
| MSC2197 | 🟢 | 100/100 | H | Search Filtering in Public Room Directory over Federation | POST /_matrix/federation/v1/publicRooms with filter implemented |
| MSC2191 | ⚫ | 0/0 | H | Markup for mathematical messages | client formatted_body rendering only |
| MSC2184 | ⚫ | 0/0 | H | Allow the HTML `&lt;details&gt;` tag in messages | client HTML rendering; no server impact |
| MSC2181 | 🟢 | 100/100 | H | Add an Error Code for Signaling a Deactivated User | M_USER_DEACTIVATED returned by login paths |
| MSC2176 | 🟢 | 100/100 | H | Update the redaction rules | redact_in_place uses room_version_rules.redaction |
| MSC2175 | 🟢 | 100/100 | H | Remove the `creator` field from `m.room.create` events | creator() falls back to sender when use_room_create_sender |
| MSC2174 | 🟢 | 100/100 | H | move the `redacts` property to `content` | src/core/matrix/event/redact.rs handles redacts move per room rules |
| MSC2140 | ⚫ | 0/0 | H | Terms of Service API for Identity Servers and Integration Managers | IS+IM ToS API; HS-side 3pid/unbind+delete absent but 3PID disabled |
| MSC2134 | ⚫ | 0/0 | H | Identity Hash Lookups | identity-server only; tuwunel is HS |
| MSC2078 | ⭕️ | 0/0 | H | Sending Third-Party Request Tokens via the Homeserver | 3PID requestToken handlers return ThreepidDenied; submit_url not added |
| MSC2077 | 🟢 | 100/100 | H | Room version 5 | src/core/config/room_version.rs:7; v5 unstable but supported |
| MSC2076 | 🟡 | 40/50 | M | Enforce key-validity periods when validating event signatures | minimum_valid_until_ts passed for fetches; per-event ts check absent |
| MSC2033 | 🟢 | 100/100 | H | Proposal to include device IDs in `/account/whoami` | src/api/client/account.rs:74 returns device_id in whoami response |
| MSC2010 | ⚫ | 0/0 | H | MSC 2010: Proposal to add client-side spoilers | client-side rendering of data-mx-spoiler in formatted_body |
| MSC2002 | 🟢 | 100/100 | H | MSC 2002 - Rooms V4 | v4 in supported_room_versions; ruma rules implement v4 |
| MSC1983 | 🟢 | 100/100 | H | Proposal to add reasons for leaving a room | src/api/client/membership/leave.rs:21 passes body.reason to leave |
| MSC1961 | ⚫ | 0/0 | H | Integration manager authentication | merged; integration-manager auth API is on the manager, not homeserver |
| MSC1960 | 🟡 | 40/40 | M | OpenID Connect information exchange for widgets | server openid endpoint exists; widget exchange is client-side |
| MSC1957 | ⭕️ | 0/0 | M | Integration manager discovery | merged; m.integrations not present in /.well-known/matrix/client |
| MSC1954 | 🟢 | 100/100 | H | Remove prev_content from the essential keys list | merged; identical to MSC1953; ruma redact omits prev_content |
| MSC1946 | 🟢 | 80/90 | M | Secure Secret Storage and Sharing | generic account_data + to-device pipe carry secret storage/sharing |
| MSC1930 | 🟢 | 100/100 | H | Proposal to add a default push rule for m.room.tombstone events | ruma Ruleset::server_default includes ConditionalPushRule::tombstone() |
| MSC1929 | 🟡 | 60/80 | H | MSC1929 Homeserver Admin Contact and Support page | /.well-known/matrix/support implemented; only single contact via config (no a... |
| MSC1915 | 🟡 | 40/40 | M | MSC 1915 - Add unbind 3PID APIs | deactivate returns no-support; 3pid stack not implemented |
| MSC1884 | 🟢 | 100/100 | H | Proposal to replace slashes in event IDs | room v4 supported via ruma EventIdFormatVersion::V3 (URL-safe base64) |
| MSC1866 | 🟡 | 60/70 | L | MSC 1866 - Unsupported Room Version Error Code for Invites | federation invite errors propagated; not explicitly mapped |
| MSC1831 | 🟢 | 100/100 | H | Proposal to do SRV lookups after .well-known to discover homeservers | src/service/resolver/actual.rs:79 well-known before SRV |
| MSC1819 | 🟢 | 100/100 | H | Remove references to presence lists | duplicate of MSC1818; presence lists not implemented |
| MSC1812 | 🟢 | 100/100 | H | MSC 1813 - Federation Make Membership Room Version | src/api/server/make_leave.rs:34 and make_join.rs:52 set room_version |
| MSC1804 | 🟢 | 100/100 | H | Proposal for advertising capable room versions to clients | src/api/client/capabilities.rs sets RoomVersionsCapability |
| MSC1802 | 🟢 | 100/100 | H | Remove the '200' value from some federation responses | src/api/server/send_join.rs:30 and send_leave.rs:15 handle v2 |
| MSC1794 | 🟢 | 100/100 | H | MSC 1794 - Federation v2 Invite API | src/api/server/invite.rs:27 implements PUT /federation/v2/invite |
| MSC1779 | ⚫ | ?/? | H | Proposal for Open Governance of Matrix.org | governance/foundation document; not a homeserver feature |
| MSC1772 | 🟢 | 90/90 | H | Proposal for Matrix "spaces" (formerly known as "groups as rooms (take 2)") | spaces implemented; src/api/client/space.rs hierarchy + room create with type |
| MSC1767 | ⭕️ | 0/0 | M | Extensible events in Matrix | no extensible-events handling; relies on generic event relay |
| MSC1759 | 🟡 | 50/50 | M | MSC 1759 - Rooms V2 | v2 algorithm in use for v3+; v2 itself not in supported_room_versions |
| MSC1756 | 🟢 | 90/100 | H | Cross-signing devices with device signing keys | src/api/client/keys/upload_signing_keys.rs and upload_signatures.rs implement... |
| MSC1753 | 🟢 | 100/100 | H | client-server capabilities API | src/api/client/capabilities.rs handles GET /capabilities incl m.change_password |
| MSC1730 | 🟢 | 100/100 | H | Mechanism for redirecting to an alternative server during login | src/api/client/session/mod.rs:176 sets well_known on login response |
| MSC1721 | 🟢 | 100/100 | H | Rename `m.login.cas` to `m.login.sso` | src/api/client/session/sso.rs and uiaa.rs advertise m.login.sso |
| MSC1719 | ⚫ | ?/? | H | Olm unwedging | client-only behavior (m.dummy, session re-creation rate-limit) |
| MSC1717 | 🟢 | 90/100 | M | Key verification mechanisms | to_device transport carries m.key.verification.* events |
| MSC1711 | 🟢 | 100/100 | M | X.509 certificate verification for federation connections | reqwest+rustls; tls_fingerprints not exposed; standard CA validation |
| MSC1708 | 🟢 | 100/100 | H | .well-known support for server name resolution | src/service/resolver/well_known.rs; resolver/actual.rs ordering matches spec |
| MSC1704 | 🟢 | 100/100 | H | matrix.to permalink navigation | server-side requirement is via= on /join; src/api/client/membership/join.rs:84 |
| MSC1693 | 🟢 | 100/100 | H | Specify how to handle rejected events in new state res | rejected event handling in iterative auth check matches MSC1442 amendment |
| MSC1692 | 🟡 | 40/80 | M | Terms of service at registration | AuthType::Terms exists in Ruma but Tuwunel's register flow does not advertise... |
| MSC1659 | 🟢 | 90/100 | H | Changing Event IDs to be Hashes | reference_hash event IDs; v3 in UNSTABLE_ROOM_VERSIONS; auth_events as list-o... |
| MSC1544 | ⚫ | ?/? | H | Key verification using QR codes | amendment PR to MSC1543; no separate proposal text |
| MSC1543 | ⚫ | ?/? | H | Bi-directional Key verification using QR codes | client-only QR verification over send-to-device; server is opaque |
| MSC1501 | 🟢 | 90/90 | H | Room version upgrades | upgrade endpoint present; tombstone, predecessor, PL freeze all implemented |
| MSC1466 | 🟢 | 100/100 | H | Soft Remote Logout Proposal | soft_logout=true returned for expired tokens in 401 responses |
| MSC1442 | 🟢 | 90/100 | H | State Resolution: Reloaded | state res v2 implemented in src/service/rooms/state_res/resolve.rs |
| MSC1219 | 🟢 | 90/100 | H | Storing megolm keys serverside | key backup endpoints fully implemented in src/api/client/backup.rs |

## Spec compliance gaps

Merged MSCs (in the live Matrix spec) that Tuwunel does not
fully implement. These are the highest-priority items to fix
for spec compliance.

| MSC | Status | Correct/Impl | Conf | Title | Note |
|---|---|---|---|---|---|
| MSC4291 | 🟡 | 80/90 | H | Room IDs as hashes of the create event | hydra.11 room id format and auth rules in event_auth, pdu format checks |
| MSC3787 | 🟡 | 71/? | H | Allowing knocks to restricted rooms | complement: 34p/14f |
| MSC2409 | 🟡 | 70/70 | H | Proposal to send typing, presence and receipts to appservices | typing+receipt EDUs sent to AS; presence not forwarded |
| MSC4380 | 🟡 | 70/70 | H | Invite blocking | phase A landed (invite-creating endpoints gated, M_INVITE_BLOCKED 403); phase... |
| MSC1866 | 🟡 | 60/70 | L | MSC 1866 - Unsupported Room Version Error Code for Invites | federation invite errors propagated; not explicitly mapped |
| MSC1929 | 🟡 | 60/80 | H | MSC1929 Homeserver Admin Contact and Support page | /.well-known/matrix/support implemented; only single contact via config (no a... |
| MSC3030 | 🟡 | 60/80 | H | Jump to date API endpoint | client and federation timestamp_to_event handlers; no remote fallback when lo... |
| MSC3824 | 🟡 | 60/60 | M | OAuth 2.0 API aware clients | oauth_aware_preferred set in /login; SSO redirect action param ignored |
| MSC3861 | 🟡 | 60/70 | M | Next-generation auth for Matrix, based on OAuth 2.0/OIDC | OIDC core endpoints implemented but not advertised as MSC3861 itself |
| MSC1759 | 🟡 | 50/50 | M | MSC 1759 - Rooms V2 | v2 algorithm in use for v3+; v2 itself not in supported_room_versions |
| MSC2675 | 🟡 | 50/60 | H | Serverside aggregations of message relationships | /relations exists; only m.thread bundling, no m.replace bundle |
| MSC2676 | 🟡 | 50/60 | H | Message editing | edits accepted/relayed; no m.replace bundle or new_content apply |
| MSC3267 | 🟡 | 50/50 | M | reference relationships | reference relations queryable via /relations; no m.relations bundling |
| MSC3550 | 🟡 | 50/50 | M | Add HTTP 403 to possible profile lookup responses | federation 403 returned; client /profile still 404 only |
| MSC3925 | 🟡 | 50/50 | M | m.replace aggregation with full event | Tuwunel doesn't replace content (good) but also lacks bundled m.replace aggre... |
| MSC4025 | 🟡 | 50/50 | H | Local user erasure requests | phase A landed (account-data wipe); phase B (per-event visibility gate) deferred |
| MSC4191 | 🟡 | 50/80 | M | Account management for OAuth 2.0 API | metadata wired but action names diverge from MSC |
| MSC4225 | 🟡 | 50/50 | M | Specification of an order in which one-time-keys should be issued | OTKs scanned in lexicographic key-id order; not strict upload order |
| MSC1692 | 🟡 | 40/80 | M | Terms of service at registration | AuthType::Terms exists in Ruma but Tuwunel's register flow does not advertise... |
| MSC1915 | 🟡 | 40/40 | M | MSC 1915 - Add unbind 3PID APIs | deactivate returns no-support; 3pid stack not implemented |
| MSC1960 | 🟡 | 40/40 | M | OpenID Connect information exchange for widgets | server openid endpoint exists; widget exchange is client-side |
| MSC2076 | 🟡 | 40/50 | M | Enforce key-validity periods when validating event signatures | minimum_valid_until_ts passed for fetches; per-event ts check absent |
| MSC2746 | 🟡 | 40/40 | L | Improved Signalling for 1:1 VoIP | Events relayed; no specific server hooks |
| MSC3856 | 🟡 | 40/60 | M | Threads List API | GET /threads route present but participated filter and latest-event order mis... |
| MSC2705 | 🟡 | 30/40 | M | Animated thumbnails | animated param accepted; thumbnails always PNG static |
| MSC3283 | 🟡 | 30/30 | M | Expose enable_set_displayname, enable_set_avatar_url and enable_3pid_changes ... | only m.3pid_changes capability set; set_displayname/set_avatar_url absent |
| MSC3765 | 🟡 | 30/40 | M | Rich text in room topics | topic_block accepted via Ruma; createRoom only writes plain topic |
| MSC3860 | 🟡 | 30/40 | M | Media Download Redirects | forwards allow_redirect to remote fetch but does not emit own redirect |
| MSC4277 | 🟡 | 30/40 | M | Harmonizing the reporting endpoints | event and room report endpoints exist; user report endpoint absent |
| MSC2540 | 🟡 | 20/40 | M | Stricter event validation: JSON compliance | ruma exposes strict_canonical_json flag; Tuwunel does not enforce floats reje... |
| MSC3816 | 🟡 | 20/40 | M | Clarify Thread Participation | BundledThread.current_user_participated hardcoded true on first reply only |
| MSC4178 | 🟡 | 10/30 | M | Error codes for requestToken | requestToken returns ThreepidDenied; not the new codes |
| MSC3381 | 🟡 | 0/? | M | Chat Polls | complement: 0p/2f |
| MSC3930 | 🟡 | 0/? | M | Polls push rules/notifications | complement: 0p/2f |
| MSC4311 | 🟡 | 0/? | M | Ensuring the create event is available on invites | complement: 0p/1f |
| MSC1767 | ⭕️ | 0/0 | M | Extensible events in Matrix | no extensible-events handling; relies on generic event relay |
| MSC1957 | ⭕️ | 0/0 | M | Integration manager discovery | merged; m.integrations not present in /.well-known/matrix/client |
| MSC2078 | ⭕️ | 0/0 | H | Sending Third-Party Request Tokens via the Homeserver | 3PID requestToken handlers return ThreepidDenied; submit_url not added |
| MSC2244 | ⭕️ | 0/0 | H | Mass redactions | Single-target redactions only; no array redacts handling |
| MSC2290 | ⭕️ | 0/0 | H | Separate Endpoints for Binding Threepids | No /account/3pid/add or /bind handlers; 3PID generally not impl |
| MSC3288 | ⭕️ | 0/0 | H | Add room type to `/_matrix/identity/v2/store-invite` API | no /_matrix/identity/v2/store-invite call site; no room_type forwarded |
| MSC3419 | ⭕️ | 0/0 | M | Guest State Events | guests still cannot send arbitrary state events |
| MSC3666 | ⭕️ | 0/0 | H | Bundled aggregations for server side search | search results do not include bundled aggregations |
| MSC3771 | ⭕️ | 0/10 | H | Read receipts for threads | thread_id discarded; receipts always Unthreaded |
| MSC3773 | ⭕️ | 0/10 | H | Notifications for threads | sync returns empty unread_thread_notifications map |
| MSC3823 | ⭕️ | 0/0 | H | Account Suspension | no M_USER_SUSPENDED errcode or suspension behavior |
| MSC3939 | ⭕️ | 0/0 | H | Account locking | Account locking (M_USER_LOCKED, soft_logout) not implemented. |
| MSC3980 | ⭕️ | 0/0 | M | Dotted Field Consistency | event_fields filter escaping not enforced |
| MSC4115 | ⭕️ | 0/0 | H | membership metadata on events | unsigned.membership not populated on events served to clients |
| MSC4222 | ⭕️ | 0/0 | H | Adding `state_after` to `/sync` | MSC4222 commit lives only on the `4222` feature branch, not dev |
| MSC4284 | ⭕️ | 0/0 | H | Policy Servers | no policy server /sign, no m.room.policy state handling |
| MSC4323 | ⭕️ | 0/0 | H | User suspension &amp; locking endpoints | client admin suspend/lock endpoints not implemented |
| MSC4335 | ⭕️ | 0/0 | H | M_USER_LIMIT_EXCEEDED error code | M_USER_LIMIT_EXCEEDED error code not used |
| MSC4341 | ⭕️ | 0/0 | H | Support for RFC 8628 Device Authorization Grant | OAuth Device Authorization Grant (RFC 8628) not advertised |

## Open

Sorted by MSC number, highest first.

| MSC | Status | Correct/Impl | Conf | Title | Note |
|---|---|---|---|---|---|
| MSC4460 | ⭕️ | 0/0 | H | Extensible Events - Alternative unstable support | Client-side hybrid extensible-events rendering rules; no Tuwunel dispatch. |
| MSC4459 | ⭕️ | 0/0 | H | Image pack references | Client-side image pack reference field; homeserver passes events through tran... |
| MSC4458 | 🟢 | 80/80 | M | Handling incoming JSON in the server-server API | Incoming PDUs deserialized via serde_json into CanonicalJsonObject |
| MSC4457 | ⭕️ | 0/0 | H | Generic reporting API | No /_matrix/client/v1/safety/report endpoint |
| MSC4456 | ⚫ | 0/0 | H | Harms taxonomy | Pure spec appendix listing harm identifiers |
| MSC4455 | ⚫ | 0/0 | H | Catch-all property for spaces | Client-only space catch-all state event; MSC says servers not required |
| MSC4454 | ⚫ | 0/0 | H | Deprecating Spoiler Fallback In Media Repository | Client-side spoiler text behavior; no server change |
| MSC4453 | 🟡 | 20/30 | H | Deprecate old room versions | v3-v5 marked unstable; v6-v9 still stable; create/upgrade not gated |
| MSC4452 | ⭕️ | 0/0 | H | Preview URL capabilities API | No m.preview_url or io.element.msc4452.preview_url capability |
| MSC4451 | ⚫ | 0/0 | H | Deprecate notifications endpoint | Spec-only deprecation; endpoint still served per MSC |
| MSC4450 | 🟡 | 10/20 | H | Identity Provider selection for User-Interactive Authentication with Legacy S... | UIAA SSO fallback derives idp from session, not idp_id query |
| MSC4449 | 🟡 | 20/30 | H | Updated /members filtering | Single membership filter only; no array support, no mutual-exclusion error |
| MSC4448 | ⭕️ | 0/0 | H | Preview URL Site Logos | No matrix:site_logo or msc4448:site_logo in preview_url response |
| MSC4447 | ⭕️ | 0/0 | H | Move OpenID userinfo endpoint out of `/_matrix/federation` | Old /federation/v1/openid/userinfo present; new /_matrix/openid/v1/userinfo n... |
| MSC4446 | 🟡 | 10/30 | H | Allow moving the fully read marker to older events | No allow_backward field; no monotonicity check on m.fully_read |
| MSC4445 | ⭕️ | 0/0 | M | Clarify `/sync` timeline order | No msc4445 unstable_features flags advertised |
| MSC4441 | ⚫ | 0/0 | H | Encrypted User Profile Annotations via Account Data | Client-side only encrypted account data convention |
| MSC4440 | ⭕️ | 0/0 | H | Profile Biography via Global Profiles | Generic MSC4133 passthrough only; no m.biography validation |
| MSC4439 | ⭕️ | 0/0 | H | Encryption key URIs in `/.well-known/matrix/support` | No pgp_key field on /.well-known/matrix/support contacts |
| MSC4438 | 🟢 | 100/100 | H | Message bookmarks via account data | Pure account-data convention; existing endpoints store arbitrary types |
| MSC4437 | ⭕️ | 0/0 | H | Endpoint to replace entire profile | No PUT /_matrix/client/v3/profile/{userId} replace-all endpoint |
| MSC4436 | 🟢 | 100/100 | H | Make server ACLs case insensitive | Ruma is_allowed uses WildMatch::new_case_insensitive |
| MSC4435 | ⭕️ | 0/0 | H | Room slowmode | No m.room.slowmode handling |
| MSC4433 | ⭕️ | 0/0 | H | Image Packs and Room Upgrades | Room upgrade does not transfer m.room.image_pack or update m.image_pack.rooms |
| MSC4432 | ⭕️ | 0/0 | H | Server-wide room name overrides | No m.room.name.server_wide propagation; no capability |
| MSC4431 | ⭕️ | 0/0 | H | Personalised room name overrides | Server side passively allows m.room.name.private as account data |
| MSC4430 | ⭕️ | 0/0 | H | Member Keys | No member-key room version, no /member_key federation endpoint |
| MSC4429 | ⭕️ | 0/0 | H | Profile Updates for Legacy Sync | No top-level users field in /sync; no profile_fields filter |
| MSC4428 | ⭕️ | 0/0 | H | Stable identifiers for Room Members | No member_info or unsigned.stable_id added to events or sync |
| MSC4427 | ⭕️ | 0/0 | H | Custom banners for user profiles | No m.banner_url or chat.commet.profile_banner support |
| MSC4426 | 🟡 | 50/60 | M | User Status Profile Fields | Profile keys passthrough via MSC4133 endpoints; no specific m.status/m.call v... |
| MSC4425 | ⭕️ | 0/0 | H | Ephemeral media | no ephemeral query param; no DELETE on /_matrix/client/v1/media/.../.... |
| MSC4423 | 🟢 | 100/100 | H | Undefine order of room directory | undefines /publicRooms ordering; Tuwunel's existing order is now compatible. |
| MSC4420 | ⭕️ | 0/0 | H | Duplicate one-time key error response for /keys/upload | add_one_time_key silently overwrites; no M_DUPLICATE_ONE_TIME_KEY emitted. |
| MSC4418 | 🟢 | 100/100 | H | Make `destination` a required server authentication field | destination required on inbound and outbound; cited verbatim in MSC. |
| MSC4417 | ⭕️ | 0/0 | H | URL Previews via Appservices | client preview_url exists; no appservice fan-out or namespace check. |
| MSC4416 | ⭕️ | 0/0 | H | Optionally requiring policy server signatures in a room | depends on MSC4284; no policy-server signature checks anywhere. |
| MSC4414 | ⚫ | 0/0 | H | Design decision - Errors | design-direction proposal with no technical changes. |
| MSC4413 | 🟢 | 100/100 | M | Remove `private` join_rule | private join_rule treated as unknown; effective semantics already aligned. |
| MSC4412 | ⚫ | 0/0 | H | Widget Base PostMessage API | widget postMessage protocol; entirely client/host-widget. |
| MSC4411 | ⚫ | 0/0 | H | Widget State Event | widget state event schema only; server stores the state event opaquely. |
| MSC4409 | ⚫ | 0/0 | H | Clarify thumbnailing behavior in E2EE | clarifies client thumbnail behavior in E2EE; no server change. |
| MSC4407 | ⚫ | 0/0 | H | Sticky Events (Widget API) | widget API for sticky events; no homeserver involvement beyond MSC4354. |
| MSC4406 | 🟡 | 70/70 | H | `M_SENDER_IGNORED` error code | src/api/client/{room/event.rs:74,context.rs:86,relations.rs:175}; M_SENDER_IG... |
| MSC4405 | ⚫ | 0/0 | H | Deprecate the emoji method for SAS verification | deprecates emoji SAS in favor of decimal; client-side method choice. |
| MSC4404 | ⚫ | 0/0 | H | Compare emoji by name rather than image | adds accept_languages to to-device verification; client SAS UI guidance. |
| MSC4403 | ⭕️ | 0/0 | H | Forbid `event_id` on PDUs received over federation | new room version forbidding event_id on PDUs; com.nhjkl.msc4403.opt2 absent. |
| MSC4402 | ⚫ | 0/0 | H | Consistent redirects for .well-known-files | [→ MSC2499?] client-side guidance to follow 30x on /.well-known/matrix/client. |
| MSC4401 | ⭕️ | 0/0 | M | Publishing client capabilities via profiles | generic profile keys exist; logout cleanup of client_capability missing. |
| MSC4400 | ⭕️ | 0/0 | H | Remove the depth field from PDUs | new room version removing depth field; com.nhjkl.msc4400.opt1 absent. |
| MSC4397 | ⚫ | 0/0 | H | Tags as Spaces | account_data key m.tag_space points at a private space; server is opaque. |
| MSC4396 | ⭕️ | 0/0 | H | Inline linked media | no multipart/mixed event-with-media; no m.media mixin or M_GONE wired. |
| MSC4392 | ⚫ | 0/0 | H | Encrypted reactions and replies | client puts m.relates_to inside encrypted payload; server forwards untouched. |
| MSC4391 | ⚫ | 0/0 | H | Simplified in-room bot commands | in-room bot command UI; state and message events forwarded opaquely. |
| MSC4390 | ⭕️ | 0/0 | H | Room Blocking API | [→ MSC4375?] no client admin endpoints for room block/delete; only federation... |
| MSC4389 | ⚫ | 0/0 | H | Image ordering within packs | image pack ordering is account-data; server passes through opaque blobs. |
| MSC4388 | ⭕️ | 0/0 | H | Secure out-of-band channel for sign in with QR | no /_matrix/client/v1/rendezvous endpoints; rendezvous API absent. |
| MSC4387 | ⭕️ | 0/0 | H | `M_SAFETY` error code | M_SAFETY errcode not used anywhere in src/; no harms field handling. |
| MSC4386 | ⚫ | 0/0 | H | Automatically sharing secrets after device verification | client-to-client to-device verification protocol; server forwards opaque events. |
| MSC4385 | ⚫ | ?/? | H | Pushing secrets to other devices | Client-side to-device event convention |
| MSC4384 | 🟡 | ?/50 | M | Supporting alternative room directory sorting | Largest-first sort is hardcoded; no alt-sort hook |
| MSC4383 | 🟢 | 100/100 | H | Client-Server Discovery of Server Version | src/api/client/versions.rs:33; populates Server { name, version, compiler } o... |
| MSC4382 | ⭕️ | 0/0 | H | Peppered hash verification for E2EE content moderation | No verification_hash check on report endpoint |
| MSC4377 | ⚫ | ?/? | H | Clarify Image Pack Ordering | Image pack ordering is client-side account/state data convention |
| MSC4375 | ⭕️ | 0/0 | H | Admin Room Management | No /_matrix/client/v1/admin/rooms/* endpoints |
| MSC4373 | 🟢 | 80/80 | H | Server opt-out of specific EDU types | src/api/server/edu_types.rs:9; advertises types tied to allow_incoming_* conf... |
| MSC4371 | ⭕️ | 0/0 | H | On the elimination of federation transactions. | No PUT /_matrix/federation/v2/send/{eventId\|eduId} endpoint |
| MSC4370 | ⭕️ | 0/0 | H | Federation endpoint for retrieving current extremities | No /_matrix/federation/v1/extremities endpoint |
| MSC4369 | ⭕️ | 0/10 | H | M_CAPABILITY_NOT_ENABLED error code for when capability is not enabled on an ... | Endpoints exist but return M_FORBIDDEN/Unknown not M_CAPABILITY_NOT_ENABLED |
| MSC4368 | ⭕️ | 0/0 | H | Combine definitions of M_RESOURCE_LIMIT_EXCEEDED error code and m.server_noti... | M_RESOURCE_LIMIT_EXCEEDED unused; no limit_type field |
| MSC4367 | ⭕️ | 0/0 | H | via routes in the published room directory | PublishedRoomsChunk has no via field |
| MSC4366 | ⭕️ | 0/0 | H | Resident servers in and around the room directory | publicRooms not filtered to rooms with joined members |
| MSC4365 | ⭕️ | 0/0 | H | Canonical ignore list rooms | No ignored_user_list_rooms server-side filtering |
| MSC4363 | ⭕️ | 0/0 | H | OAuth step up authentication | No M_INSUFFICIENT_USER_AUTHENTICATION error or acr_values |
| MSC4362 | ⭕️ | 0/0 | H | Simplified Encrypted State Events | No encrypt_state_events handling in m.room.encryption |
| MSC4361 | 🟢 | 100/100 | H | Non-federating Membership Authorization Rule Amendments | src/service/rooms/state_res/event_auth/room_member.rs:56; reject m.room.membe... |
| MSC4360 | ⭕️ | 0/0 | H | Threads extension to Sliding Sync | No /thread_updates endpoint or threads sliding sync extension |
| MSC4359 | ⚫ | ?/? | H | "Do not Disturb" notification settings | Client-side account data event; no server behavior required |
| MSC4358 | ⭕️ | 0/0 | H | Out of room server discovery | No /discover_common_rooms federation endpoint |
| MSC4357 | ⚫ | ?/? | H | Live Messages via Event Replacement | Client-only convention reusing m.replace; no server work |
| MSC4354 | ⭕️ | 0/0 | H | Sticky Events | No sticky events handling on send or sync |
| MSC4353 | ⭕️ | 0/0 | H | Per-origin linear chain | No origin_predecessor field or per-origin chain validation |
| MSC4352 | ⭕️ | 0/0 | H | Customizable HTTPS permalink base URLs via server discovery | No permalink_base_url in /.well-known/matrix/client output |
| MSC4351 | 🟢 | 100/100 | H | Odd Context Limits | Context handler biases remainder to events_after via div_ceil(2) |
| MSC4350 | ⭕️ | 0/0 | H | Permitting encryption impersonation for appservices | No impersonator field in device keys, no /keys/query handling |
| MSC4349 | ⭕️ | 0/0 | H | Causal barriers and enforcement | causal barrier terminology and deferred authorization not adopted |
| MSC4348 | ⭕️ | 0/0 | H | Portable and serverless accounts in rooms | portable accounts (account keys); not implemented |
| MSC4347 | ⚫ | ?/? | H | Emoji verification images | client-side emoji image rendering for SAS verification; not server |
| MSC4345 | ⭕️ | 0/0 | H | Server key identity and room membership | server key as room identity; massive auth-rule changes; not implemented |
| MSC4344 | ⭕️ | 0/0 | H | Strike deprecated SRV service name. | deprecated _matrix._tcp SRV still queried |
| MSC4343 | ⭕️ | 0/0 | H | Making mass redactions use a new event type | m.room.redactions (mass redactions) event not used; depends on MSC2244 |
| MSC4342 | ⭕️ | 0/0 | H | Limiting the number of devices per user ID | 30-device limit and M_TOO_MANY_DEVICES not enforced |
| MSC4340 | ⭕️ | 0/0 | H | Prompts and partial commands for in room commands. | bot command prompts; client-side concern, no server changes |
| MSC4339 | ⭕️ | 0/0 | H | Allow the user directory to return full profiles | user_directory v4 with profile_fields not implemented |
| MSC4337 | ⭕️ | 0/0 | H | Appservice API to supplement user profiles | appservice profile supplement endpoint not queried |
| MSC4334 | ⭕️ | 0/0 | H | Add `m.room.language` state event. | m.room.language state event; not whitelisted/handled specially |
| MSC4333 | ⭕️ | 0/0 | H | Room state API for moderation bots | moderation bot state event; client-side concern |
| MSC4332 | ⭕️ | 0/0 | H | In-room bot commands | in-room bot commands; client-side concern, no server changes |
| MSC4331 | ⭕️ | 0/0 | H | Device Account Data | per-device account data routes not implemented |
| MSC4330 | 🟡 | 50/50 | M | specify HTTP and TLS versions which must be supported | HTTP/2 via axum/hyper available; TLS 1.2+ via rustls; not enforced as MUST |
| MSC4329 | ⭕️ | 0/0 | H | Inviting with authorization | federation /v3/invite with create event in `state` not implemented |
| MSC4325 | ⭕️ | 0/0 | H | Presence privacy | presence privacy filtering by m.presence_sharing_config not implemented |
| MSC4324 | 🟢 | 80/80 | M | Fixing MSC4289's power level for tombstones | tombstone PL=150 set; matches highest-anchored intent for default config |
| MSC4322 | ⭕️ | 0/0 | H | Simple Media Self-Redaction | [→ MSC3911?] media self-redaction; no /media/redact endpoint or EDU |
| MSC4321 | ⭕️ | 0/0 | H | Policy Room Upgrade Semantics | policy room upgrade `move`/`transition` semantics not handled |
| MSC4320 | ⭕️ | 0/0 | H | Rich Presence | Rich Presence m.rpc; no support for activity/media profile field |
| MSC4319 | ⭕️ | 0/0 | H | Room member events for invite and knock rooms in the `/sync` response | `state` key in InvitedRoom/KnockedRoom; not added to /sync responses |
| MSC4310 | 🟡 | 30/30 | M | MatrixRTC decline `m.rtc.notification` | event-only MSC; ruma feature enabled, no homeserver-specific behavior |
| MSC4309 | ⭕️ | 0/0 | H | Finalised delayed events on sync | finalised delayed events on /sync; depends on MSC4140; no impl |
| MSC4308 | 🟡 | 0/? | M | Thread Subscriptions extension to Sliding Sync | complement: 0p/3f |
| MSC4306 | 🟡 | 8/? | H | Thread Subscriptions | complement: 1p/12f |
| MSC4305 | ⭕️ | 0/0 | H | Pushed Authorization Requests (PARs) for OAuth authentication | OIDC auth_metadata lacks PAR endpoint fields |
| MSC4303 | ⭕️ | 0/0 | H | Disallowing non-compliant user IDs in rooms | no future room version banning non-compliant user IDs |
| MSC4302 | ⚫ | ?/? | H | Exchanging FHIR resources via Matrix events | new event type for FHIR, no server logic |
| MSC4300 | ⚫ | ?/? | H | Processing status requests &amp; responses | client-to-client status request/response in events |
| MSC4299 | ⚫ | ?/? | M | trusted users | foundation MSC; defines account-data only, no concrete server behavior |
| MSC4298 | ⭕️ | 0/0 | H | Room version components for 'Redact on ban' | no future room version protecting redact_events from redaction |
| MSC4296 | ⚫ | ?/? | H | Mentions for device IDs | client-side mentions field extension |
| MSC4295 | ⚫ | ?/? | H | Bot bounce limit - a better loop prevention mechanism | bot/client behavior; servers relay events unmodified |
| MSC4293 | ⭕️ | 0/0 | H | Redact on kick/ban | MSC4293 commit lives only on Continuwuity branches; current tree has no redac... |
| MSC4292 | ⚫ | 0/0 | H | Handling incompatible room versions in clients | [→ MSC4331] |
| MSC4286 | ⚫ | ?/? | H | App store compliant handling of payment links within events | client-side HTML rendering attribute |
| MSC4283 | ⚫ | ?/? | H | Distinction between Ignore and Block | terminology MSC, no implementation surface |
| MSC4282 | ⭕️ | 0/0 | H | Hint that a /rooms/{room_id}/messages request is interactive | no interactive query parameter on /messages |
| MSC4279 | ⭕️ | 0/0 | H | Server notice rooms | no notice room presets, no leave_rules, no server_notice room type filter |
| MSC4278 | ⚫ | ?/? | H | Media preview controls | client-side account data preferences |
| MSC4276 | ⭕️ | 0/0 | H | Soft unfailure for self redactions | no self-redaction soft-fail bypass |
| MSC4274 | ⚫ | ?/? | H | Inline media galleries via msgtypes | new client msgtype m.gallery, no server logic |
| MSC4273 | ⚫ | ?/? | H | Approve and Disapprove ratings for moderation policies | new event type for moderation tools, no server logic |
| MSC4271 | ⭕️ | 0/0 | M | Recommended enabled-ness for default push rules | no admin override knob; uses Ruma defaults verbatim |
| MSC4270 | ⚫ | ?/? | H | Matrix Glossary | glossary/spec doc proposal, not an implementation feature |
| MSC4269 | ⚫ | ?/? | H | Unambiguous mentions in body | client-side message body composition |
| MSC4266 | ⭕️ | 0/0 | H | Policies in /.well-known/matrix/support | policies field not added to /.well-known/matrix/support |
| MSC4265 | 🟡 | 50/50 | M | Data Protection Officer contact in /.well-known/matrix/support | support_role configurable; MSC role string accepted as Custom |
| MSC4264 | ⭕️ | 0/0 | H | Tokens for Contacting Accounts or Joining Semi-Public Rooms | Tokens for contact / semi-public-room joins not implemented |
| MSC4263 | 🟡 | 50/50 | M | Preventing MXID enumeration via key queries | MUST floor met implicitly; MAY restriction unused |
| MSC4262 | ⭕️ | 0/0 | H | Sliding Sync Extension: Profile Updates | Sliding-sync profiles extension not implemented |
| MSC4261 | ⚫ | 0/0 | H | "Do not encrypt for device" flag | do_not_encrypt is a client-only device key flag |
| MSC4259 | ⭕️ | 0/0 | H | Profile Update EDUs for Federation | m.profile EDU broadcast not implemented |
| MSC4258 | ⭕️ | 0/0 | H | Federated User Directory | Federated user_directory/search not implemented |
| MSC4257 | ⭕️ | 0/0 | H | Profiles Arent Auth: Move profile contents to a separate event | m.room.member.profile separate event not supported |
| MSC4256 | ⭕️ | 0/0 | H | RFC 9420 MLS mode Matrix | MLS mode rooms not implemented |
| MSC4255 | ⭕️ | 0/0 | H | Bulk Profile Updates | Bulk PUT/PATCH /profile not implemented |
| MSC4253 | ⚫ | 0/0 | H | Modifying or rejecting accepted MSCs | Spec process MSC; no implementable behavior |
| MSC4252 | ⚫ | 0/0 | H | Extensible Events modification: State event handling | Client-side guidance for extensible state events |
| MSC4250 | ⭕️ | 0/0 | H | Authenticated media v2 (Cookie authentication for Client-Server API) | set_auth_cookie media auth not implemented |
| MSC4249 | 🟢 | 100/100 | H | Removal of legacy media endpoints | allow_legacy_media defaults to false; legacy disabled |
| MSC4247 | 🟡 | 40/40 | M | User Pronouns | MSC4133 generic profile fields cover m.pronouns transparently |
| MSC4246 | ⭕️ | 0/0 | H | Sending to-device messages as/to a server | Empty-localpart server addressing for to-device absent |
| MSC4245 | ⭕️ | 0/0 | H | Immutable encryption algorithm | encryption_algorithm in m.room.create not honored |
| MSC4244 | ⭕️ | 0/0 | H | RFC 9420 MLS for Matrix | MLS for Matrix not implemented |
| MSC4243 | ⭕️ | 0/0 | H | User ID localparts as Account Keys | Account keys / federation query/accounts not implemented |
| MSC4242 | ⭕️ | 0/0 | H | State DAGs | State DAGs not implemented; uses standard auth chain |
| MSC4238 | ⚫ | 0/0 | H | Pinned events read marker | Client-set m.read.pinned_events account data only |
| MSC4235 | ⭕️ | 0/0 | H | `via` query param for hierarchy endpoint | hierarchy endpoint lacks via query parameter |
| MSC4234 | ⭕️ | 0/0 | H | Update app badge counts when rooms are read | cleared_notifs read-receipt flag not handled |
| MSC4233 | ⭕️ | 0/0 | H | Remembering which server a user knocked through | knock_servers field in /sync not added; no via tracking |
| MSC4232 | ⭕️ | 0/0 | H | Attribute-Based Access Control (ABAC) | ABAC permissions model; no room version implements it |
| MSC4231 | ⚫ | 0/0 | H | Backwards compatibility for media captions | Client-side caption fallback rendering; no server work |
| MSC4229 | ⚫ | ?/? | H | Pass through `unsigned` data from `/keys/upload` to `/keys-query` | template/example proposal; no real change |
| MSC4228 | ⭕️ | 0/0 | H | Search Redirection | optional 403 search redirection not used |
| MSC4227 | ⭕️ | 0/0 | H | Audio based quick login | no MSC4108 rendezvous support; audio/DTMF login absent |
| MSC4226 | ⭕️ | 0/0 | H | Reports as rooms | reports-as-rooms (m.report room type) not implemented |
| MSC4224 | ⭕️ | 0/0 | H | CBOR Serialization | application/cbor content negotiation not implemented |
| MSC4223 | ⭕️ | 0/0 | H | Error code for disallowing threepid unbinding | 3pid unbind/delete endpoints not implemented at all |
| MSC4221 | 🟢 | 100/100 | H | Room Banners | event-only; passthrough |
| MSC4220 | ⭕️ | 0/0 | H | Local call rejection (m.call.reject_locally) | event-only; m.call.reject_locally not interpreted |
| MSC4218 | ⭕️ | 0/0 | H | Improving performance of profile changes | synthetic events / m.room.user_profile not implemented |
| MSC4211 | 🟢 | 100/100 | H | WebXDC on Matrix | event-only; passthrough |
| MSC4209 | ⚫ | ?/? | H | Updating endpoints in-place | deprecation policy clarification; no code |
| MSC4208 | 🟡 | 40/50 | M | Adding User-Defined Custom Fields to User Global Profiles | custom profile fields work; u.* namespace not validated |
| MSC4207 | ⭕️ | 0/0 | H | Media identifier moderation policy | m.policy.rule.mxc not interpreted |
| MSC4206 | ⭕️ | 0/0 | H | Moderation policy auditing and context | m.policy.rule.context not interpreted server-side |
| MSC4205 | ⭕️ | 0/0 | H | Hashed moderation policy entities | hashed entity policies not interpreted |
| MSC4204 | ⭕️ | 0/0 | H | `m.takedown` moderation policy recommendation | no m.takedown recommendation handling |
| MSC4203 | 🟡 | 10/20 | H | Sending to-device events to appservices | to_device field wired in transaction body but always empty |
| MSC4202 | 🟡 | 40/40 | M | Reporting User Profiles | client report endpoint exists; federation forwarding absent |
| MSC4201 | ⭕️ | 0/10 | H | Profiles as Rooms v2 | only generic /profile/{user} exists; no roomID profile lookup |
| MSC4198 | ⭕️ | 0/0 | H | Usage of OIDC login_hint | login_hint not handled at OIDC auth |
| MSC4197 | 🟢 | 100/100 | H | Copy-Paste Hints | event content field; passthrough |
| MSC4196 | ⭕️ | 0/0 | H | MatrixRTC voice and video calling application `m.call` | m.call MatrixRTC slots; no m.rtc.member or m.call.intent handling |
| MSC4195 | 🟡 | 40/40 | M | MatrixRTC Transport using LiveKit Backend | livekit advertised in /rtc/transports; JWT and delayed events out of scope |
| MSC4194 | ⭕️ | 0/0 | H | Batch redaction of events by sender within a room (including soft failed events) | POST /rooms/{}/redact/user/{} not wired |
| MSC4193 | 🟢 | 100/100 | H | Spoilers on Media | event content field; passthrough; nothing for HS to do |
| MSC4192 | ⚫ | ?/? | H | Comparison of proposals for ignoring invites | comparison/research document, not a feature |
| MSC4188 | ⭕️ | 0/0 | H | Handling HTTP 410 Gone Status in Matrix Server Discovery | 410 Gone not specially handled in well-known resolver |
| MSC4186 | 🟢 | 90/90 | H | Simplified Sliding Sync | sync v5 implementation routed at simplified_msc3575 path |
| MSC4185 | ⭕️ | 0/0 | H | Event Visibility API | no can_user_see_event endpoint |
| MSC4184 | ⭕️ | 0/0 | H | Dynamic Notification Suppression | no m.push_rules_executed field on events |
| MSC4179 | ⚫ | ?/? | H | Moderation event hiding | client-side rendering hint |
| MSC4177 | ⭕️ | 0/0 | H | Add upload location hints proposal | no m.upload.locations or location query param |
| MSC4176 | ⭕️ | 0/0 | H | Translatable Errors | no localized error messages map |
| MSC4174 | ⭕️ | 0/0 | H | Web push | no webpush pusher kind or VAPID |
| MSC4173 | ⭕️ | 0/0 | H | test pusher | no /pushers/push test endpoint |
| MSC4171 | ⭕️ | 0/0 | H | Service members | no service members handling in heroes |
| MSC4168 | 🟡 | 60/60 | H | Update `m.space.*` state on room upgrade | src/api/client/room/upgrade.rs:447; copies m.space.parent always plus m.space... |
| MSC4167 | ⭕️ | 0/0 | H | Copy bans on room upgrade | bans not copied during room upgrade |
| MSC4166 | 🟢 | 100/100 | H | Specify `/turnServer` response when no TURN servers are available | turnServer returns 404 M_NOT_FOUND when no TURN URIs configured |
| MSC4165 | 🟢 | 100/100 | H | Remove own power level on deactivation | power level entry removed for self on deactivation |
| MSC4164 | 🟢 | 100/100 | H | Leave all rooms on deactivation | deactivation leaves all joined/invited/knocked rooms |
| MSC4162 | 🟡 | 30/30 | M | One-Time Key Reset Endpoint | no /keys/reset; claim ordering is implicit via key prefix iter |
| MSC4161 | ⚫ | ?/? | H | Crypto terminology for non-technical users | crypto terminology guidance for clients |
| MSC4158 | 🟢 | 80/100 | M | MatrixRTC focus information in .well-known | rtc_foci exposed in .well-known/matrix/client |
| MSC4157 | ⚫ | ?/? | H | Delayed Events (widget-api) | widget-api only; not a homeserver concern |
| MSC4155 | ⭕️ | 0/0 | H | Invite filtering | no m.invite_permission_config handling |
| MSC4154 | 🟢 | 100/100 | H | Request max body size | max_request_size default 24MB, M_TOO_LARGE returns 413 |
| MSC4152 | ⭕️ | 0/0 | H | Room labeling and filtering | room labels and /rooms/{roomId}/labels not implemented |
| MSC4150 | ⚫ | ?/? | H | m.allow recommendation for moderation policy lists | m.allow recommendation for policy lists is client-side |
| MSC4149 | 🟡 | 80/80 | M | Update CSP Directives for Media Repository | global CSP aligns with MSC; missing font-src and script-src 'none' |
| MSC4148 | ⭕️ | 0/0 | H | Permitting HTTP(S) URLs for SSO IdP icons | SSO IdP icon limited to mxc URIs in config; HTTP(S) not allowed |
| MSC4146 | ⚫ | ?/? | H | Shared Message Drafts | shared message drafts via m.drafts rooms is client-side |
| MSC4145 | ⭕️ | 0/0 | H | Simple verified accounts | m.verified profile field and endpoint not implemented |
| MSC4144 | ⚫ | ?/? | H | Per-message profiles | m.per_message_profile is client-only event content |
| MSC4143 | 🟢 | 90/90 | M | MatrixRTC | GET rtc/transports routed; only HS-side requirement of the MSC |
| MSC4141 | ⭕️ | 0/0 | H | Time based notification filtering | time_and_day push rule condition not supported |
| MSC4140 | ⭕️ | 0/0 | H | Cancellable delayed events | delayed events endpoints not implemented despite Ruma types |
| MSC4139 | ⚫ | ?/? | H | Bot buttons &amp; conversations | m.prompts mixin is client-only event content |
| MSC4136 | ⭕️ | 0/0 | H | Shared retry hints between servers | retry_hints in /send_join response not implemented |
| MSC4131 | ⚫ | ?/? | H | Handling `m.room.encryption` events | client-side guidance on handling m.room.encryption events |
| MSC4128 | 🟢 | 100/100 | H | Error on invalid auth where it is optional | invalid token returns error even on optional auth endpoints |
| MSC4127 | ⭕️ | 0/0 | H | Removal of query string auth | removal of query string auth not implemented; still accepted |
| MSC4125 | 🟢 | 90/100 | H | Specify servers to join via for federated invites | federation invite via field used both inbound and outbound |
| MSC4121 | 🟢 | 100/100 | H | `m.role.moderator` `/.well-known/matrix/support` role. | m.role.moderator served via Ruma ContactRole alias and config |
| MSC4120 | ⭕️ | 0/0 | H | Allow `HEAD` on `/download` | HEAD on /download not wired; routes mounted via Ruma metadata GET only |
| MSC4119 | ⚫ | ?/? | H | Voluntary content flagging | client-only m.room.context flagging mixin; server is content-agnostic |
| MSC4117 | ⭕️ | 0/0 | H | Reinstating Events (Reversible Redactions) | m.room.reinstate (reversible redactions) not implemented |
| MSC4114 | ⚫ | ?/? | H | Matrix as a password manager | client-only password manager via rooms; no server-side requirements |
| MSC4110 | ⭕️ | 0/0 | H | Fewer Features | m.room.event_features state event has no special server handling |
| MSC4109 | ⭕️ | 0/0 | H | Appservices &amp; soft-failed events | appservice v2/transactions endpoint with soft-failed events absent |
| MSC4108 | 🟡 | 20/20 | M | Mechanism to allow OAuth 2.0 API sign in and E2EE set up via QR code | auth_metadata route present; rendezvous and device grant absent |
| MSC4107 | ⭕️ | 0/0 | H | Feature-focused versioning | features key on /versions not added |
| MSC4106 | ⭕️ | 0/0 | H | Join as Muted | join-as-muted default_membership not implemented |
| MSC4104 | ⭕️ | 0/0 | H | Auth Lock: Soft-failure-be-gone! | m.auth_lock event and auth-rule not implemented |
| MSC4103 | ⭕️ | 0/0 | M | Make threaded read receipts opt-in in /sync | threaded_read_receipts sync filter not implemented |
| MSC4102 | ⭕️ | 0/0 | M | Clarifying precedence in threaded and unthreaded read receipts in EDUs | unthreaded-takes-precedence aggregation rule not enforced |
| MSC4101 | ⭕️ | 0/0 | H | Hashes for unencrypted media | hashes field on unencrypted media info not consumed by server |
| MSC4100 | ⭕️ | 0/0 | H | Scoped signing keys | scoped signing keys / X-Matrix-Scoped not implemented |
| MSC4097 | ⭕️ | 0/0 | H | Interactions between media redirection and authentication | media redirect symmetric encryption not implemented |
| MSC4096 | ⭕️ | 0/0 | H | Proposal to make forceTurn option configurable server-side | forceTurn not advertised in well-known |
| MSC4095 | 🟡 | 40/40 | M | Bundled URL previews | Ruma type-defs enabled; server is content-agnostic for events |
| MSC4094 | ⭕️ | 0/0 | H | Sync Server and Client Times with endpoint | GET /_matrix/client/v3/get_server_now endpoint missing |
| MSC4092 | ⚫ | ?/? | H | Enforce tests around sensitive parts of the specification | process MSC about test enforcement; no protocol changes |
| MSC4089 | ⭕️ | 0/0 | H | Delivery Receipts | m.delivery receipts not implemented |
| MSC4086 | ⭕️ | 0/0 | H | Event media reference counting | event-media reference counting not implemented |
| MSC4084 | ⭕️ | 0/0 | H | Improving security of MSC2244 | v4 send endpoint with UIA for redactions not implemented |
| MSC4083 | ⭕️ | 0/0 | H | Delta-compressed E2EE file transfers | delta-compressed media transfers not implemented |
| MSC4081 | ⭕️ | 0/0 | H | Eagerly sharing fallback keys with federated servers | eager fallback key sharing not implemented |
| MSC4080 | ⭕️ | 0/0 | H | Cryptographic Identities (Client-Owned Identities) | cryptographic identities/send_pdus endpoint not implemented |
| MSC4079 | ⭕️ | 0/0 | H | Server-Defined Client Landing Pages | landing_page in well-known not implemented |
| MSC4078 | ⭕️ | 0/0 | H | Registering pushers against push notification services should forward back fa... | upstream_errcode/upstream_error not surfaced from /pushers/set |
| MSC4076 | 🟢 | 90/100 | H | Let E2EE clients calculate app badge counts themselves (disable_badge_count) | disable_badge_count honored when sending push notifications |
| MSC4075 | ⭕️ | 0/0 | H | MatrixRTC Notification Event (call ringing) | m.rtc.notification push rule and event handling absent |
| MSC4074 | ⭕️ | 0/0 | H | Server side annotation aggregation | server-side annotation aggregation not implemented |
| MSC4073 | ⚫ | ?/? | H | Shepherd teams | process MSC about SCT shepherd teams; not protocol |
| MSC4072 | ⭕️ | 0/0 | H | Handling devices with no one-time keys in `/keys/claim` | Missing/exhausted devices are filtered out, not returned as empty objects. |
| MSC4071 | ⭕️ | 0/0 | H | Pagination Token Headers | No X-Matrix-Pagination-* header handling. |
| MSC4069 | ⭕️ | 0/0 | H | Inhibit profile propagation | No ?propagate query parameter on profile endpoints. |
| MSC4062 | ⚫ | 0/0 | M | Add a push rule tweak to disable email notification | Tuwunel has no email pusher; tweak only affects email pushers. |
| MSC4060 | ⭕️ | 0/0 | H | Accept room rules before speaking | No m.room.rules state event or acceptance gating. |
| MSC4059 | ⭕️ | 0/0 | H | Mutable event content | No mutable-event EDU or hashes-omitted detection. |
| MSC4058 | ⭕️ | 0/0 | H | Additive Events | No m.additive EDU or unsigned.m.additive metadata pipeline. |
| MSC4057 | ⭕️ | 0/0 | H | Static Room Aliases | No .well-known/matrix/rooms lookup before federation directory. |
| MSC4056 | ⭕️ | 0/0 | H | Role-Based Access Control (mk II) | No m.role / m.role_map RBAC support. |
| MSC4053 | ⭕️ | 0/0 | M | Extensible Events - Mentions mixin | No mixin push rules with room_version_supports condition. |
| MSC4051 | 🟢 | 80/80 | M | Using the create event as the room ID | V12 RoomVersionRules.room_create_event_id_as_room_id dispatched. |
| MSC4050 | ⚫ | 0/0 | H | MXID verification | Pure client/third-party signaling via custom event types. |
| MSC4049 | ⭕️ | 0/0 | H | Sending events as a server or room | No room version permitting non-user-ID senders. |
| MSC4048 | ⭕️ | 0/0 | H | Authenticated key backup | No m.backup.v2.curve25519-aes-sha2 algorithm or backup_mac handling. |
| MSC4047 | ⭕️ | 0/0 | H | Send Keys | No m.room.send_key state event or send-key auth path. |
| MSC4046 | ⭕️ | 0/0 | H | Make &amp; send PDU endpoints | None of the four make_pdu/send_pdu endpoints implemented. |
| MSC4045 | ⭕️ | 0/0 | H | Deprecating the use of IP addresses in server names | No room version banning IP-literal server names. |
| MSC4044 | ⭕️ | 0/0 | H | Enforcing user ID grammar in rooms | No room version enforcing strict user ID grammar. |
| MSC4043 | ⭕️ | 0/0 | H | Presence Override API | No /presence/{userId}/override endpoint. |
| MSC4042 | ⭕️ | 0/0 | H | Disabled Presence State | No 'disabled' presence state. |
| MSC4039 | ⚫ | 0/0 | H | Access the Content repository with the Widget API | Widget API extension; entirely client-to-widget scope. |
| MSC4038 | ⭕️ | 0/0 | H | Key backup for MLS | No MLS or m.dmls_backup.v1.aes-hmac-sha2 backup algorithm support. |
| MSC4037 | 🟡 | ?/40 | L | Thread root is not in the thread | Receipts allowed for thread roots; spec wording is mostly client-facing. |
| MSC4036 | ⚫ | 0/0 | H | Room organization by promoting threads | Pure client UI behavior toggled by m.promote_threads state event. |
| MSC4034 | ⭕️ | 0/0 | H | Media limits | No /usage endpoint and no m.storage.* fields in /config. |
| MSC4033 | ⭕️ | 0/0 | H | Explicit ordering of events for receipts | No order field on events or receipts. |
| MSC4032 | ⚫ | 0/0 | H | Asset Collections | Asset Collections defines client-side data structures for 3D worlds; no serve... |
| MSC4031 | ⭕️ | 0/0 | H | Pre-generating invites and room invite codes | pre-generated invites and m.room.invite state event not implemented |
| MSC4029 | 🟡 | 40/50 | M | Fixing `X-Matrix` request authentication | X-Matrix verification covers basics; canonicalization rules not fully specified |
| MSC4028 | ⭕️ | 0/0 | H | Push all encrypted events except for muted rooms | .m.rule.encrypted_event server-default override rule absent |
| MSC4027 | ⚫ | ?/? | H | Custom Images in Reactions | custom image reactions, m.annotation key semantics |
| MSC4023 | ⭕️ | 0/0 | H | Thread ID for second-order relation | unsigned.thread_id not added to events |
| MSC4021 | ⭕️ | 0/0 | H | Archive client controls | m.room.archive_controls not relayed in /publicRooms |
| MSC4020 | ⭕️ | 0/0 | H | Room model configuration | m.room.create model object flagging not supported |
| MSC4019 | ⭕️ | 0/0 | H | Encrypted event relationships | m.room.relationship_encryption flag not handled by server |
| MSC4016 | ⚫ | ?/? | H | Streaming and resumable E2EE file transfer with random access | streaming E2EE file transfer needs new media transport |
| MSC4014 | ⭕️ | 0/0 | H | Pseudonymous Identities | pseudonymous identities (sender_key, mxid_mapping) not implemented |
| MSC4013 | ⚫ | ?/? | H | Poll history cache | client convention using existing relations API |
| MSC4011 | ⭕️ | 0/0 | H | Thumbnail media negotiation | thumbnail Accept header negotiation not implemented |
| MSC4006 | ⚫ | ?/? | H | Answered Elsewhere for VoIP | VoIP m.call.hangup reason value, client concern |
| MSC4005 | ⭕️ | 0/0 | M | Explicit read receipts for sent events | Server does not auto-generate read receipt on send |
| MSC4004 | ⚫ | ?/? | H | unified view of identity service | identity service API, not homeserver |
| MSC4003 | ⚫ | ?/? | H | Semantic table attributes | HTML table sanitization is client concern |
| MSC4002 | ⚫ | ?/? | H | Walkie talkie | Walkie-talkie real-time voice, vague client-driven proposal |
| MSC4001 | 🟡 | 0/40 | H | Return start of room state at context endpoint | context returns state at LAST event, MSC asks for state at FIRST |
| MSC4000 | ⭕️ | 0/0 | H | Forwards fill (`/backfill` forwards) | forwards_fill federation endpoint not implemented |
| MSC3999 | ⭕️ | 0/0 | H | Add causal parameter to `/timestamp_to_event` | timestamp_to_event causal event_id parameter not supported |
| MSC3998 | ⭕️ | 0/0 | H | Add timestamp massaging to `/join` and `/knock` | join/knock ts query param not honored |
| MSC3997 | ⭕️ | 0/0 | H | Add timestamp massaging to `/createRoom` | createRoom ts query param not honored (always timestamp: None) |
| MSC3996 | ⭕️ | 0/0 | H | Encrypted mentions-only rooms | m.has_mentions cleartext flag and is_encrypted_mention rule not present |
| MSC3995 | ⭕️ | 0/0 | H | Linearized Matrix | Linearized Matrix hub/participant architecture not implemented |
| MSC3994 | ⭕️ | 0/0 | H | Display why an event caused a notification | rule_kind/rule_id not added to /notifications |
| MSC3993 | ⭕️ | 0/0 | H | Room takeover | room takeover variants not implemented |
| MSC3991 | ⭕️ | 0/0 | H | Power level up! Taking the room to new heights | raise own power level above max not allowed |
| MSC3985 | ⭕️ | 0/0 | H | Break-out rooms | m.breakout state event not handled |
| MSC3984 | ⭕️ | 0/0 | H | Sending key queries to appservices | key query proxy to appservice not implemented |
| MSC3983 | ⭕️ | 0/0 | H | Sending One-Time Key (OTK) claims to appservices | OTK claim proxy to appservice not implemented |
| MSC3982 | ⭕️ | 0/0 | H | Limit maximum number of events sent to an AS | no 100-event cap on appservice transactions |
| MSC3979 | ⚫ | ?/? | H | Revised feature profiles | client feature profiles, not a homeserver concern |
| MSC3977 | ⚫ | ?/? | H | Introduction | IETF MIMI framework draft, not a Matrix MSC |
| MSC3973 | ⚫ | ?/? | H | Search users in the user directory with the Widget API | widget API extension; client/embedder feature |
| MSC3971 | ⭕️ | 0/0 | H | Sharing image packs | image pack sharing/links not implemented |
| MSC3964 | ⭕️ | 0/0 | H | Notifications for room tags | room_tag push condition not implemented |
| MSC3963 | ⭕️ | 0/0 | H | Oblivious Matrix over HTTPS | Oblivious MoH endpoints absent |
| MSC3961 | 🟢 | 90/100 | H | Sliding Sync Extension: Typing Notifications | sliding sync typing extension implemented |
| MSC3960 | 🟢 | 90/100 | H | Sliding Sync Extension: Receipts | sliding sync receipts extension implemented |
| MSC3959 | 🟢 | 90/100 | H | Sliding Sync Extension: Account Data | sliding sync account_data extension implemented |
| MSC3956 | ⚫ | ?/? | H | Extensible Events - Encrypted Events | client-side extensible encrypted event format |
| MSC3955 | ⭕️ | 0/0 | H | Extensible Events - Automated event mixin (notices) | m.automated mixin for extensible events not implemented |
| MSC3954 | ⭕️ | 0/0 | H | Extensible Events - Text Emotes | Extensible m.emote event type not specifically handled. |
| MSC3949 | ⚫ | 0/0 | H | Power Level Tags | Power-level tag state event is client UX; no server enforcement. |
| MSC3948 | ⚫ | 0/0 | H | Repository room for Thirdroom | ThirdRoom 3D-asset repository room type; no homeserver semantics. |
| MSC3947 | ⭕️ | 0/0 | H | Allow Clients to Request Searching the User Directory Constrained to Only Hom... | exclude_sources parameter on user_directory/search not implemented. |
| MSC3946 | ⭕️ | 0/0 | H | Dynamic room predecessor | m.room.predecessor state event not handled. |
| MSC3944 | ⭕️ | 0/0 | H | Dropping stale send-to-device messages | Stale-to-device cancellation/dedup logic not implemented. |
| MSC3935 | ⚫ | 0/0 | H | Cute Events against social distancing | Client-side cute event msgtype; no server behavior. |
| MSC3934 | ⭕️ | 0/0 | H | Bulk push rules change endpoint | PUT /pushrules_bulk/.../actions and /enabled endpoints not implemented. |
| MSC3933 | ⭕️ | 0/0 | H | Core push rules for Extensible Events | Extensible-event default underride push rules not added. |
| MSC3932 | ⭕️ | 0/0 | H | Extensible events room version push rule feature flag | Extensible-event room version push rule gating not enabled. |
| MSC3931 | ⭕️ | 0/0 | H | Push rule condition for room version features | room_version_supports push condition not enabled in tuwunel. |
| MSC3927 | ⭕️ | 0/0 | H | Extensible Events - Audio | Extensible m.audio event type not specifically dispatched. |
| MSC3926 | ⭕️ | 0/0 | H | Disable server-default notifications for bot users by default | enable_predefined_push_rules registration body field not implemented. |
| MSC3922 | ⭕️ | 0/0 | H | Removing SRV records from homeserver discovery | SRV record discovery still active; would need code removal. |
| MSC3919 | ⚫ | 0/0 | H | Matrix Message Format (IETF/MIMI) | IETF informational draft on Matrix message format; not a server feature. |
| MSC3918 | ⚫ | 0/0 | H | Matrix Message Transport (IETF/MIMI) | IETF informational draft about Matrix as MIMI transport; not a server feature. |
| MSC3917 | ⭕️ | 0/0 | H | Cryptographically Constrained Room Membership | Cryptographic membership (RRK / RSK / signed memberships) not implemented. |
| MSC3915 | ⭕️ | 0/0 | H | Owner power level | PL150 owner role / creator-defaults-to-150 not implemented. |
| MSC3914 | ⭕️ | 0/0 | H | Matrix native group call push rule | .m.rule.room.call push rule + call_started condition not implemented. |
| MSC3912 | ⭕️ | 0/0 | H | Redaction of related events | with_rel_types / with_relations on /redact not implemented. |
| MSC3911 | ⭕️ | 0/0 | H | Linking media to events | attach_media query, /media/copy, restrictions block in federation media not p... |
| MSC3910 | ⚫ | 0/0 | H | Content tokens for media | [→ MSC3916] |
| MSC3909 | ⭕️ | 0/0 | H | Membership based mutes | Membership-based mutes via new mute/leave-mute states; not implemented. |
| MSC3908 | ⚫ | ?/? | H | Expiring Policy List Recommendations | expiring policy field interpreted by clients/bots |
| MSC3907 | ⚫ | ?/? | H | Mute Policy Recommendation | mute policy recommendation enforced by clients/bots |
| MSC3906 | ⚫ | 0/0 | H | Protocol to use an existing Matrix client session to complete login and setup... | [→ MSC4108] |
| MSC3903 | ⚫ | ?/? | H | X25519 Elliptic-curve Diffie-Hellman ephemeral for establishing secure channe... | [→ MSC4108] client-to-client X25519 ECDH; no server role |
| MSC3902 | 🟡 | 30/40 | M | Faster remote room joins over federation (overview) | sends omit_members but immediately fetches full state |
| MSC3901 | ⭕️ | 0/0 | M | Deleting State | meta-MSC of sub-proposals; obsolete-state cleanup not implemented |
| MSC3898 | ⚫ | ?/? | H | Native Matrix VoIP signalling for cascaded SFUs | VoIP SFU signalling is opaque events between clients |
| MSC3896 | ⭕️ | 0/0 | H | Appservice media | appservice media namespace not implemented |
| MSC3895 | ⭕️ | 0/0 | H | Federation API Behaviour of Partial-State Resident Servers | M_UNABLE_DUE_TO_PARTIAL_STATE error code not implemented |
| MSC3892 | ⚫ | ?/? | H | Custom Emotes with Encryption | custom emotes are pure client/state-event feature |
| MSC3890 | 🟡 | 0/? | M | Remotely silence local notifications | complement: 0p/2f |
| MSC3888 | ⚫ | ?/? | M | Voice Broadcast | voice broadcast is opaque events, no server change required |
| MSC3886 | ⚫ | 0/0 | H | Simple client rendezvous capability | [→ MSC4108] |
| MSC3885 | 🟢 | 100/100 | H | Sliding Sync Extension: To-Device | to_device extension uses its own opaque since token in v5 sync |
| MSC3884 | 🟢 | 90/100 | H | Sliding Sync Extension: E2EE | sliding sync e2ee extension implemented |
| MSC3883 | ⭕️ | 0/0 | H | Fundamental state changes | draft proposal, no concrete API; would require new room version |
| MSC3881 | ⭕️ | 0/0 | H | Remotely toggling push notifications for another client | pusher enabled and device_id fields not exposed |
| MSC3880 | ⚫ | ?/? | H | dummy replies for Olm | client-side Olm dummy event behavior |
| MSC3879 | ⚫ | ?/? | H | Trusted key forwards | E2EE key forwarding flag is client-side |
| MSC3874 | 🟡 | 0/? | M | MSC3874 Loading Messages excluding Threads | complement: 0p/1f |
| MSC3872 | ⭕️ | 0/0 | M | Order of rooms in Spaces | manual room ordering in spaces; vague proposal, no API defined |
| MSC3871 | 🟡 | 50/? | H | Gappy timeline | complement: 3p/3f |
| MSC3870 | ⭕️ | 0/0 | H | Async media upload extension: upload to URL | upload_url field and /complete endpoint not implemented |
| MSC3869 | ⚫ | ?/? | H | Read event relations with the Widget API | Widget API extension; homeservers do not implement widget API |
| MSC3868 | ⚫ | ?/? | M | Room Contribution | custom state event for room contribution, no server requirements |
| MSC3866 | ⭕️ | 0/0 | H | `M_USER_AWAITING_APPROVAL` error code | M_USER_AWAITING_APPROVAL error code not implemented |
| MSC3865 | 🟢 | 100/100 | H | User-given attributes for users | client-side; uses generic account_data endpoints already implemented |
| MSC3864 | 🟢 | 100/100 | H | User-given attributes for rooms | client-side; uses generic account_data endpoints already implemented |
| MSC3862 | ⭕️ | 0/0 | H | event_match (almost) anything | event_match only matches strings; non-string primitives not converted |
| MSC3857 | ⭕️ | 0/0 | H | Welcome messages/screening | no m.room.welcome state event handling |
| MSC3852 | ⭕️ | 0/0 | H | Expose user agent information on `Device` | last_seen_user_agent not exposed on Device |
| MSC3851 | ⭕️ | 0/0 | H | Allow custom room presets when creating a room | only standard RoomPreset variants accepted; no custom string presets |
| MSC3849 | ⭕️ | 0/0 | H | Observations and Reinforcement | no observation/reinforcement event handling |
| MSC3848 | ⭕️ | 0/0 | H | Introduce errcodes for specific event sending failures. | no M_INSUFFICIENT_POWER/M_NOT_JOINED/M_ALREADY_JOINED errcodes emitted |
| MSC3847 | ⭕️ | 0/0 | H | Ignoring invites with policy rooms | no policy room handling for m.policies account data |
| MSC3846 | ⚫ | 0/0 | H | Allowing widgets to access TURN servers | widget TURN access; client-widget API only |
| MSC3845 | ⭕️ | 0/0 | H | Draft: Expanding policy rooms to reputation | no m.opinion recommendation handling |
| MSC3843 | ⭕️ | 0/0 | H | Reporting content over federation | federation /rooms/{}/report/{} endpoint not implemented |
| MSC3842 | ⚫ | 0/0 | H | Power levels on message (extensible) events | proposal body is TBD; nothing to implement |
| MSC3840 | ⭕️ | 0/0 | M | Ignore invites | client-side ignored invites account data; no server behavior required |
| MSC3839 | ⚫ | 0/0 | H | primary-identity-as-key | speculative login system replacement; not actionable as a proposal |
| MSC3837 | ⭕️ | 0/0 | H | Cascading profile tags for push rules | no profile_tags array; only single profile_tag handled |
| MSC3834 | ⭕️ | 0/0 | H | Opportunistic user key pinning (TOFU) | TOFU signing key is client-side; no server hooks |
| MSC3825 | ⭕️ | 0/0 | M | Obvious relation fallback location | is_falling_back location handled by Ruma types passively |
| MSC3819 | ⚫ | 0/0 | H | Allowing widgets to send/receive to-device messages | widget to-device is client-widget API only |
| MSC3817 | ⚫ | 0/0 | H | Allow widgets to create rooms | widget API only, no server-side surface |
| MSC3815 | ⚫ | 0/0 | H | 3D Worlds | 3D worlds is client-side room type and state events; no server behavior |
| MSC3814 | 🟢 | 80/90 | H | Dehydrated devices with SSSS | dehydrated devices SSSS routes wired with put/get/delete and events pagination |
| MSC3813 | ⚫ | 0/0 | H | Obfuscated events | obfuscated events; client-side dummy traffic |
| MSC3812 | ⚫ | 0/0 | H | Hint buttons in messages | hint buttons in messages; client UI |
| MSC3803 | ⚫ | 0/0 | H | Matrix Widget API v2 | Widget API v2 issue placeholder |
| MSC3796 | ⚫ | 0/0 | M | Auth/linking for content repo (and enforcing GDPR erasure) | [→ MSC3916] |
| MSC3784 | ⚫ | 0/0 | H | Using room type of `m.policy` for policy rooms | m.policy room-type identifier; informational only |
| MSC3780 | ⚫ | 0/0 | H | Knocking on `action=join` | matrix-uri client UX fallback for knock |
| MSC3779 | ⭕️ | 0/0 | H | "Owned" state events | owned state events require new room version |
| MSC3775 | ⚫ | 0/0 | H | Markup Locations for Audiovisual Media | event content schema for media markup |
| MSC3772 | ⭕️ | 0/0 | H | Push rule for mutually related events | relation_match push condition not implemented |
| MSC3768 | ⚫ | 0/0 | H | Push rule action for in-app notifications | [→ MSC2625] |
| MSC3767 | ⭕️ | 0/0 | H | Time based notification filtering | time_and_day push condition not present |
| MSC3761 | ⭕️ | 0/0 | H | State event change control | m.event.acl ACL events for state not implemented |
| MSC3760 | ⭕️ | 0/0 | H | State sub-keys | state_subkey requires new room version; not present |
| MSC3759 | ⭕️ | 0/0 | H | Leave event metadata for deactivated users | deactivation leaves omit m.deactivated metadata |
| MSC3757 | 🟡 | 0/? | M | Restricting who can overwrite a state event. | [→ MSC4354] complement: 0p/1f |
| MSC3755 | ⚫ | 0/0 | H | Member pronouns | pronouns are client member-content fields |
| MSC3752 | ⚫ | 0/0 | H | Markup locations for text | event content schema for text markup locations |
| MSC3751 | ⚫ | 0/0 | H | Allowing widgets to read account data | Widget API permission, not a homeserver concern |
| MSC3744 | ⭕️ | 0/0 | H | Support for flexible authentication | no flexible-auth /register or /account/authenticator endpoints |
| MSC3741 | ⭕️ | 0/0 | H | Revealing the useful login flows to clients after a soft logout | login does not return per-user flows for soft-logout tokens |
| MSC3735 | ⚫ | 0/0 | M | Add device information to m.room_key.withheld message | client-side to-device field; server relays unchanged |
| MSC3726 | ⭕️ | 0/0 | H | Safer Password-based Authentication with BS-SPEKE | open MSC; no BS-SPEKE login/register/password flows |
| MSC3725 | ⚫ | 0/0 | H | Content warnings | client-side content warning event content; no server changes |
| MSC3723 | ⭕️ | 0/0 | H | Federation `/versions` | open MSC; no /_matrix/federation/versions endpoint |
| MSC3720 | ⭕️ | 0/0 | H | Account status endpoint | branch MSC; no /account_status endpoints (CS or federation) |
| MSC3713 | ⭕️ | 0/0 | H | Alleviating ACL exhaustion with ACL Slots | open MSC; no ACL slot state-key handling |
| MSC3682 | ⭕️ | 0/0 | H | Sending Account Data to Application Services | AS transactions do not include account_data field |
| MSC3673 | ⭕️ | 0/0 | H | Encrypting ephemeral data units | branch MSC; no encrypted EDU envelope support |
| MSC3672 | ⭕️ | 0/0 | H | Sharing ephemeral streams of location data | branch MSC; no m.beacon EDU support or location streaming |
| MSC3664 | ⭕️ | 0/0 | H | Pushrules for relations | no related_event_match push rule condition implemented |
| MSC3662 | ⚫ | 0/0 | H | Allow Widgets to share user MxIds to the client | widget-to-client API; no server involvement |
| MSC3647 | ⭕️ | 0/0 | H | Bring Your Own Bridge - Decentralising Bridges | WIP bridge negotiation; no spec-level details, no server impl |
| MSC3644 | ⚫ | 0/0 | H | Extensible Events: Edits and replies | client-side extensible event format; no server-side dispatch |
| MSC3639 | ⚫ | 0/0 | H | Matrix for the social media use case | client-side social media room/event conventions; no server changes |
| MSC3635 | ⚫ | 0/0 | H | Early Media for VoIP | client-side VoIP signalling; no server changes required |
| MSC3618 | ⭕️ | 0/0 | M | Simplify federation `/send` response | branch MSC; tuwunel returns full pdus map per current spec |
| MSC3613 | ⭕️ | 0/0 | H | Combinatorial join rules | branch MSC; no combinatorial join_rules array logic in tuwunel |
| MSC3593 | ⭕️ | 0/0 | H | Safety Controls through a generic Administration API | none of the proposed /admin/* endpoints exist; tuwunel uses admin room |
| MSC3592 | ⚫ | 0/0 | H | Markup locations for PDF documents | client-side PDF markup event types; no server implementation required |
| MSC3585 | 🟢 | 100/100 | H | Allow the base event to be omitted from `/federation/v1/event_auth` response | event_auth handler omits the requested event itself per MSC |
| MSC3575 | 🟢 | ?/? | M | Sliding Sync (aka Sync v3) | [→ MSC4186] src/api/client/sync/v5.rs:62 |
| MSC3574 | ⭕️ | 0/0 | H | Marking up resources | no m.markup.resource or annotation handling |
| MSC3572 | ⭕️ | 0/0 | M | Relation aggregation cleanup | no relations rename; m.relations only |
| MSC3571 | ⭕️ | 0/0 | H | Aggregation pagination | no /aggregations endpoint; no aggregation pagination |
| MSC3570 | ⭕️ | 0/0 | M | Relation history visibility changes | no special history visibility for relations; new room version needed |
| MSC3554 | ⭕️ | 0/0 | H | Extensible Events - Translatable Messages | no lang field handling; ruma feature not enabled |
| MSC3553 | ⭕️ | 0/0 | H | Extensible Events - Videos | unstable-msc3553 not enabled in ruma features |
| MSC3552 | ⭕️ | 0/0 | H | Extensible Events - Images and Stickers | unstable-msc3552 not enabled in ruma features |
| MSC3551 | ⭕️ | 0/0 | H | Extensible Events - Files | unstable-msc3551 not enabled; no extensible m.file event |
| MSC3547 | ⭕️ | 0/0 | H | Allow appservice bot user to read any rooms the appservice is part of | appservice still must masquerade or be a member |
| MSC3531 | ⚫ | ?/? | H | Letting moderators hide messages pending moderation | client-only m.visibility event; server explicitly unchanged |
| MSC3523 | ⭕️ | 0/0 | H | Timeboxed/ranged relations endpoint | no from_target/to_target query params on /relations |
| MSC3510 | ⚫ | 0/0 | H | Let users with the same power level kick/ban/demote each other. | [→ MSC3915] |
| MSC3489 | 🟡 | 50/50 | M | m.beacon: Sharing streams of location data with history | unstable-msc3489 ruma feature on; no specific beacon logic |
| MSC3488 | 🟡 | 40/40 | M | m.location: Extending events with location data | location event types pass through; no m.tile_server in well-known |
| MSC3480 | 🟡 | 50/50 | M | Make device names private | allow_device_name_federation config gates device name exposure |
| MSC3469 | 🟡 | ?/50 | L | Mandate HTTP Range on Content Repository Endpoints | depends on object_store / hyper response writer for ranges |
| MSC3468 | ⭕️ | 0/0 | H | MXC to Hashes | no MXC-to-hash endpoints; no /clone or /hash routes |
| MSC3417 | 🟢 | 100/100 | H | Call room room type | creation_content type=m.call passes through createRoom |
| MSC3414 | ⭕️ | 0/0 | H | Encrypted state events | no encrypted state event handling or encrypted_state in publicRooms |
| MSC3401 | ⭕️ | 0/10 | H | Native Group VoIP signalling | only default PL for m.call/m.call.member; no to-device signaling |
| MSC3395 | ⭕️ | 0/0 | H | Synthetic Appservice Events | no synthetic appservice events emitted on register/login/logout |
| MSC3394 | ⭕️ | 0/0 | H | New auth rule that only allows someone to post a message in relation to anoth... | no auth rule restricting top-level vs threaded messages |
| MSC3389 | ⭕️ | 0/0 | H | Redaction changes for events with a relation | no m.relates_to preservation in redactions |
| MSC3386 | ⭕️ | 0/0 | H | Unified Join Rules | no unified allow_join/allow_knock; no new room version |
| MSC3385 | 🟡 | 30/40 | M | Bulk small improvements to room upgrades | upgrade copies fixed list of state, not all m.* state nor account_data |
| MSC3382 | ⚫ | ?/? | H | Inline message Attachments | PR-style amendment to MSC2881, not a standalone proposal |
| MSC3368 | ⭕️ | 0/0 | H | Message Content Tags | no message-content tag awareness |
| MSC3361 | ⭕️ | 0/0 | H | Opportunistic Direct Push | no direct pusher kind or notifications in sync |
| MSC3360 | ⭕️ | 0/0 | H | Server Status | no /server/status endpoint or m.server.status event |
| MSC3359 | ⭕️ | 0/0 | H | Delayed Push | no jitter pusher field; not advertised in versions |
| MSC3356 | ⭕️ | 0/0 | H | Add additional OpenID user info fields | openid userinfo returns only sub |
| MSC3338 | ⭕️ | 0/0 | H | Adding iframe specifics to preview json | url preview has no iframe/oEmbed support |
| MSC3325 | ⭕️ | 0/0 | H | Upgrading invite-only rooms | upgrade does not switch invite-only rooms to restricted |
| MSC3309 | ⭕️ | 0/0 | H | Room Counters | no m.room.counter event handling |
| MSC3306 | ⭕️ | 0/0 | H | How to count unread messages | notification_count uses push-rule Notify actions, not MSC3306 algo |
| MSC3277 | ⭕️ | 0/0 | H | Scheduled messages | no scheduled-message at= query param support |
| MSC3269 | ⭕️ | 0/0 | H | An error code for busy servers | no M_SERVER_BUSY error code |
| MSC3262 | ⭕️ | 0/0 | H | aPAKE authentication | SRP6a aPAKE login/registration not implemented |
| MSC3246 | ⚫ | ?/? | H | Audio waveforms (extensible events) | client message-content field; no server role |
| MSC3245 | ⚫ | ?/? | H | Voice messages (using extensible events) | client message type; ruma feature enabled but server has no role |
| MSC3230 | ⚫ | ?/? | H | Spaces top level order | m.space_order is account_data; uses generic API |
| MSC3219 | ⭕️ | 0/0 | H | Space Flair | space flair events and member flag not implemented |
| MSC3217 | ⭕️ | 0/0 | H | Clientside hints for a soft kick | m.softkick hint on member event not implemented |
| MSC3216 | ⭕️ | 0/0 | H | Synchronized access control for Spaces | space-level synchronized PL replication absent |
| MSC3215 | ⭕️ | 0/0 | H | Aristotle - Moderation in all things | decentralized moderation room scheme not implemented |
| MSC3214 | 🟢 | 90/100 | M | Allow overriding `m.room.power_levels` using `initial_state` | initial_state PL effectively replaces default via later append |
| MSC3202 | 🟡 | 30/40 | H | Encrypted Appservices | device_id masquerading present; AS txn extensions missing |
| MSC3192 | ⭕️ | 0/0 | H | Batch state endpoint | batch_state endpoint not implemented |
| MSC3189 | ⭕️ | 0/0 | H | Per-room/per-space profiles | per-room/space scoped profile API not implemented |
| MSC3184 | ⚫ | ?/? | H | Challenges Messages | client-only challenge message types |
| MSC3174 | ⭕️ | 0/0 | H | An error code for spam rejections | M_ANTISPAM_REJECTION error code not used |
| MSC3160 | ⚫ | ?/? | H | Attach timezone metadata to time information in messages | client-only HTML &lt;time&gt; markup in messages |
| MSC3144 | ⭕️ | 0/0 | H | Allow Widgets By Default in Private Rooms | private_chat preset does not lower widgets PL |
| MSC3131 | ⚫ | ?/? | H | Verifying with QR codes v2 | client-only QR verification v2 method names |
| MSC3105 | ⭕️ | 0/0 | H | Previewing user-interactive flows | OPTIONS preflight for UIA flows not implemented |
| MSC3089 | ⭕️ | 0/0 | H | File trees | client-only data trees on m.space; no server change required |
| MSC3088 | ⭕️ | 0/0 | H | Room subtyping | client-only m.room.purpose state event; no server change required |
| MSC3086 | ⚫ | 0/0 | H | Asserted Identity for VoIP Calls | client VoIP event content; server transparent |
| MSC3079 | ⭕️ | 0/0 | H | Low Bandwidth Client-Server API | branch; no CoAP/CBOR/DTLS support |
| MSC3062 | ⚫ | 0/0 | H | Bot verification | client-only verification method |
| MSC3061 | ⚫ | 0/0 | H | Sharing room keys for past messages | client-only; sender-flagged room key property |
| MSC3060 | ⭕️ | 0/0 | H | Room labels | branch; m.room.labels not surfaced in publicRooms |
| MSC3051 | ⭕️ | 0/0 | M | A scalable relation format | open; m.relations array not handled |
| MSC3038 | ⭕️ | 0/0 | H | Typed Typing Notifications | branch; no events field on typing |
| MSC3032 | 🟡 | 40/40 | M | Thoughts on updating presence | effective presence; busy supported, profile-as-rooms absent |
| MSC3026 | 🟢 | 100/100 | H | `busy` presence state | PresenceState::Busy and msc3026.busy_presence flag |
| MSC3020 | ⭕️ | 0/0 | M | Support for private federation networks | branch; same proposal as MSC3018, not implemented |
| MSC3018 | ⭕️ | 0/0 | M | Support for private federation networks | branch; no m.networks capability or network query |
| MSC3015 | ⚫ | 0/0 | H | Room state personal overrides | client-only; account data convention |
| MSC3014 | ⭕️ | 0/0 | H | HTTP Pushers for the full event with extra rooms information | open; no full_event_with_rooms pusher format |
| MSC3012 | ⭕️ | 0/0 | H | Post-registration terms of service API | branched; no /terms endpoint or m.terms account data |
| MSC3009 | ⚫ | 0/0 | H | Websocket transport for client &lt;--&gt; widget communications | client to widget transport; not server-side |
| MSC3008 | ⚫ | 0/0 | H | Scoped access for widgets | widget client/UA concern; obsoleted by OIDC scopes |
| MSC2997 | ⚫ | 0/0 | H | Add t-shirt | joke proposal; t-shirt design |
| MSC2974 | ⚫ | ?/? | H | Widgets: Re-exchange capabilities | widget-side request_capabilities; client-only |
| MSC2970 | 🟡 | 60/80 | M | Remove pusher path requirement | path/scheme constraints relaxed; lacks fragment/userinfo/8000-char checks |
| MSC2962 | ⭕️ | 0/0 | H | Managing power levels via Spaces | no auto_users or m.room.power_level_mappings handling |
| MSC2961 | 🟡 | 20/40 | M | External Signatures | endpoint accepts arbitrary signature keys; object form discarded |
| MSC2949 | ⚫ | ?/? | H | Proposal to clarify "Requires auth" and "Rate-limited" in the spec | spec-text clarification; no homeserver behavior |
| MSC2943 | ⭕️ | 0/0 | H | Return an event ID for membership endpoints | membership endpoint responses lack event_id |
| MSC2938 | ⭕️ | 0/10 | H | Report content to moderators | target field and room_moderators routing not implemented |
| MSC2931 | ⚫ | ?/? | H | Widget navigate permission | widget navigate capability; client-only |
| MSC2923 | ⭕️ | 0/0 | M | Matrix to Matrix connections | speculative idea-stage; no concrete API |
| MSC2895 | ⭕️ | 0/0 | H | Improving the way membership lists are queried | no /rooms endpoint nor ?membership query on /members |
| MSC2883 | ⭕️ | 0/0 | H | [WIP] Matrix-flavoured MLS | WIP MLS; no DMLS support |
| MSC2882 | ⭕️ | 0/0 | M | [WIP] Tempered Transitive Trust | WIP; new public_user_signing key, m.device.signature EDU not implemented |
| MSC2881 | ⚫ | ?/? | H | Message Attachments | new event content schema (m.attachment relation); generic event passthrough |
| MSC2873 | ⚫ | ?/? | H | Identifying clients and user settings in widgets | widget URL template variables and theme_change; client-only |
| MSC2872 | ⚫ | ?/? | H | Move the widget `title` to the root | widget definition field reorder; client-only |
| MSC2871 | ⚫ | ?/? | H | Sending approved capabilities back to the widget | widget-only feature; homeserver not involved |
| MSC2855 | ⭕️ | 0/0 | M | Server-Initiated Client Clear-Cache &amp; Reload | no clear-cache signal mechanism |
| MSC2848 | ⭕️ | 0/10 | H | Globally unique event IDs | only legacy GET /event/:eventId; new room-scoped path absent |
| MSC2846 | ⭕️ | 0/0 | H | Decentralizing media through CIDs | open; CID-based MXC URLs not implemented |
| MSC2845 | 🟡 | 0/20 | M | Thirdparty Lookup API for Telephone Numbers | src/api/client/thirdparty.rs returns empty protocols TODO |
| MSC2836 | 🟡 | 0/10 | H | Threading | advertises org.matrix.msc2836 in /versions but no event_relationships |
| MSC2828 | ⭕️ | 0/0 | M | Proposal to restrict allowed user IDs over federation | no extended_user_id_char auth rule restriction |
| MSC2821 | ⭕️ | 0/0 | H | Test Pusher | POST /pushers/push test endpoint not implemented |
| MSC2815 | 🟢 | 90/100 | M | Proposal to allow room moderators to view redacted event content | include_unredacted_content honored; admin or redact PL gates access |
| MSC2813 | ⚫ | ?/? | H | Handling invalid Widget API requests | client/widget error handling rules |
| MSC2812 | ⭕️ | 0/0 | H | Role-based power structures | role-based power proposal still draft; no m.role events |
| MSC2802 | ⭕️ | 0/0 | H | Full Room Abstraction | open meta proposal to redesign spec; not implementable as-is |
| MSC2790 | ⚫ | ?/? | H | Widgets - Prompting for user input within the client | client-side widget modal API |
| MSC2787 | ⭕️ | 0/0 | H | Portable Identities | no UPK/UDK/attestation infrastructure |
| MSC2785 | ⭕️ | 0/0 | H | Event notification attributes and actions | no notification_attribute_data or notifications_profile endpoints |
| MSC2782 | 🟡 | 60/80 | M | Pushers with the full event content | src/service/pusher/send.rs sends full event when format != event_id_only |
| MSC2775 | ⚫ | 0/10 | M | Lazy loading room membership over federation | [→ MSC3706/MSC3902] |
| MSC2772 | ⭕️ | 0/0 | M | Notifications for Jitsi Calls | no .m.jitsi default underride push rules |
| MSC2762 | ⚫ | ?/? | H | Allowing widgets to send/receive events | client-side widget API; homeserver not involved |
| MSC2757 | ⭕️ | 0/0 | H | Sign Events | No event_signing key type; no client signature plumbing |
| MSC2755 | ⭕️ | 0/0 | H | Lazy load rooms | No room_limit_by_complexity filter handling |
| MSC2753 | ⭕️ | 0/0 | H | Peeking via Sync (Take 2) | No /peek or /unpeek; no peek section in sync |
| MSC2749 | ⭕️ | 0/0 | H | Per-user E2EE on/off setting | No m.encryption capability; no force/preference logic |
| MSC2747 | ⚫ | 0/0 | H | Transferring VoIP Calls | Client-only m.call.replaces event semantics |
| MSC2730 | ⭕️ | 0/0 | H | Verifiable forwarded events | No /forward/{targetRoomId}; no signature validation |
| MSC2723 | ⚫ | 0/0 | H | Forwarded message metadata | Client-side m.forwarded content field only |
| MSC2716 | ⭕️ | 0/0 | H | Incrementally importing history into existing rooms | No /batch_send; no m.room.insertion/batch/marker handling |
| MSC2706 | ⭕️ | 0/0 | H | IPFS as a media repository | No IPFS support; no m.ipfs capability |
| MSC2704 | 🟢 | 100/100 | M | Handling duplicate media on `/upload` + clarifying the origin of an MXC URI | Fresh MXC per upload; no dedup |
| MSC2703 | 🟢 | 100/100 | H | Media ID grammar | 32-char alphanumeric media IDs; opaque |
| MSC2700 | 🟡 | 50/50 | M | Thumbnail requirements for the media repo | image crate handles png/jpeg/gif; no svg/video |
| MSC2695 | 🟡 | 40/40 | H | Get event by ID over federation | Federation /event exists; no client /events/{eventId} revival |
| MSC2673 | ⭕️ | 0/0 | H | Notification Levels | No notification_levels concept; push rules used |
| MSC2654 | ⭕️ | 0/0 | H | Unread counts | No unread_count in sync; no msc2654 markers |
| MSC2644 | ⚫ | 0/0 | H | `matrix.to` URI syntax v2 | matrix.to URI syntax; client-only |
| MSC2638 | ⭕️ | 0/0 | H | Ability for clients to request homeservers to resync device lists | No /devices/refresh endpoint; no msc2638 marker in src |
| MSC2625 | ⭕️ | 0/0 | M | Add `mark_unread` push rule action | No mark_unread action; sync exposes only highlight/notification counts |
| MSC2618 | ⚫ | ?/? | H | Helping others with mandatory implementation guides | Spec process MSC; no homeserver behavior |
| MSC2596 | ⭕️ | 0/0 | M | Proposal to always allow rescinding invites | Vendor room version net.maunium.msc2596 not registered; no rescind exception ... |
| MSC2545 | ⚫ | ?/? | M | Image Packs (Emoticons &amp; Stickers) | Client emote/sticker pack rendering; server stores account_data and state events |
| MSC2529 | ⚫ | ?/? | M | Use existing m.room.message/m.text events as captions for images | [→ MSC2530] Client-only relation/caption rendering; superseded by MSC2530 |
| MSC2513 | ⭕️ | 0/10 | M | Allow clients to specify content for membership events | Membership endpoints accept reason only; no content body param |
| MSC2499 | 🟡 | 30/40 | M | Fixes for Well-known URIs | src/service/resolver/well_known.rs follows redirects; 12288B cap; uses /versions |
| MSC2487 | ⭕️ | 0/0 | M | Filtering for Appservices | No filter field on appservice registration |
| MSC2477 | ⭕️ | 0/0 | M | User-defined ephemeral events in rooms | No PUT /rooms/{roomId}/ephemeral/{type}/{txnId} route |
| MSC2474 | ⚫ | ?/? | M | Add key backup version to SSSS account data | Client-side SSSS field; server stores account_data opaquely |
| MSC2448 | 🟢 | 90/100 | H | Using BlurHash as a Placeholder for Matrix Media | blurhash on profile, federation query, media upload, member events |
| MSC2444 | 🟡 | 30/30 | H | Proposal for implementing peeking over federation (peek API) | world_readable allowed on some federation reads; no /peek subscription API |
| MSC2438 | ⭕️ | 0/10 | H | Local and Federated User Erasure Requests | deactivate present but no erase param, no fed/AS erase endpoints |
| MSC2437 | 🟢 | 100/100 | M | Store tagged events in Room Account Data | m.tagged_events stored via existing room account_data routes |
| MSC2427 | ⚫ | ?/? | M | Proposal for JSON-based message formatting | Client-only message formatting alternative to HTML |
| MSC2425 | ⚫ | ?/? | H | Remove Authentication on /submitToken Identity Service API | Identity Server endpoint; not a homeserver concern |
| MSC2413 | ⚫ | ?/? | H | Remove client_secret | 3PID-only proposal; Tuwunel does not support 3PID |
| MSC2398 | ⚫ | ?/? | M | proposal to allow mxc:// in the "a" tag within messages | Client HTML rendering policy for &lt;a href=mxc:&gt; |
| MSC2391 | ⭕️ | 0/0 | H | Federation point-queries. | No federation point-query state endpoint |
| MSC2388 | ⚫ | ?/? | M | Toward the EDU-to-PDU transition: Read Receipts. | Receipts as PDU; superseded direction, Tuwunel uses EDU |
| MSC2385 | ⚫ | ?/? | M | Disable URL Previews, alternative method | Client-only url_previews array on m.room.message |
| MSC2380 | ⭕️ | 0/0 | H | Matrix Media Information API | No /media/r0/info/{origin}/{media_id} endpoint |
| MSC2379 | ⭕️ | 0/0 | H | MSC 2379: Add /versions endpoint to Appservice API. | No /_matrix/app/versions probe code |
| MSC2375 | ⭕️ | 0/0 | M | Appservice Invite States | Appservice transactions send raw PDU JSON without invite_room_state injection |
| MSC2370 | ⭕️ | 0/0 | H | Resolve URL API | No /resolve_url endpoint in source |
| MSC2359 | ⚫ | ?/? | M | E2E Encrypted SFU VoIP conferencing via Matrix | [→ MSC3401] Architectural sketch for client+SFU; no homeserver requirements |
| MSC2356 | ⭕️ | 0/0 | H | Bulk /joined_members endpoint | No POST /joined_members bulk endpoint in src/api |
| MSC2354 | ⚫ | ?/? | M | Device to device streaming file transfers | Client-only WebRTC signaling over event types; server transports opaquely |
| MSC2346 | ⚫ | ?/? | H | MSC 2346: Bridge information state event | m.bridge state event; bridge/client concern |
| MSC2326 | ⭕️ | 0/0 | H | Label based filtering | No labels/not_labels EventFilter support; no m.label handling |
| MSC2316 | ⭕️ | 0/0 | H | Federation queries to aid with database recovery | No /_matrix/federation/v1/query/members route |
| MSC2315 | ⚫ | ?/? | H | Allow users to select "none" as an integration manager | Client account_data m.integrations toggle |
| MSC2314 | ⭕️ | 0/40 | H | Backfilling Current State | src/api/server/state.rs:14 requires event_id; no current-state branch |
| MSC2306 | 🟢 | 100/100 | M | Removing MSISDN password resets | msisdn pw reset endpoint absent; ThreepidDenied on msisdn |
| MSC2301 | ⭕️ | 0/0 | H | Proposal for an /info endpoint on the CS API | No /info merger of /versions; no branding fields exposed |
| MSC2300 | ⭕️ | 0/0 | H | Proposal for a /ping endpoint on the CS API | No GET /_matrix/client/r0/ping route |
| MSC2299 | ⚫ | ?/? | H | Proposal to add m.textfile msgtype | Client-only msgtype m.textfile |
| MSC2291 | ⚫ | ?/? | H | Configuration to Control Crawling | Bot-only advisory state event; no homeserver behavior |
| MSC2278 | ⭕️ | 0/10 | M | Proposal for deleting content for expired and redacted messages | No DELETE /media client API; only admin-only delete helper |
| MSC2271 | ⭕️ | 0/0 | H | Proposal for TOTP 2FA | No TOTP endpoints, no m.login.totp UIA stage |
| MSC2270 | ⚫ | ?/? | M | Proposal for ignoring invites | Client account_data scheme; server stores account data transparently |
| MSC2261 | 🟢 | 100/100 | H | Allow `m.room.aliases` events to be redacted by room admins | Subsumed by MSC2432/v6 redaction rules |
| MSC2260 | 🟢 | 100/100 | H | Update the auth rules for `m.room.aliases` events | Subsumed by MSC2432/v6 auth rules; aliases sender-domain check enforced |
| MSC2233 | ⭕️ | 0/0 | H | Unauthenticated Capabilities API | no /capabilities/server unauthenticated endpoint |
| MSC2232 | ⚫ | 0/0 | H | Expose Homeserver Email Configuration in Registration Parameters | proposal text is the empty MSC template |
| MSC2228 | ⭕️ | 0/0 | H | Proposal for self-destructing messages | self_destruct fields not honored |
| MSC2214 | ⭕️ | 0/0 | H | Joining upgraded private rooms | m.room.previous_member event not implemented |
| MSC2213 | ⭕️ | 0/0 | H | Rejoinability of private rooms | rejoin_rule field not implemented |
| MSC2212 | ⭕️ | 0/0 | H | Third party user power levels | third_party_users not present in PL handling or auth rules |
| MSC2211 | ⚫ | 0/0 | H | Identity Servers Storing Threepid Hashes at Rest | identity server storage details; not HS |
| MSC2199 | ⭕️ | 0/0 | H | Canonical DMs (server-side middle ground edition) | no m.kind in sync summary; uses legacy m.direct account data |
| MSC2192 | ⚫ | 0/0 | H | Inline widgets | client extensible event m.embed; no server logic |
| MSC2190 | 🟢 | 80/80 | M | Allow appservice bots to use /sync | appservice token defaults to sender_localpart user |
| MSC2162 | ⚫ | 0/0 | M | Signaling Errors at Bridges | client/bridge event types; no homeserver enforcement |
| MSC2153 | 🟢 | 100/100 | H | Add a default push rule to ignore m.reaction events | Ruleset::server_default() includes .m.rule.reaction via Ruma |
| MSC2127 | ⭕️ | 0/0 | H | Proposal for a federation capabilities API | federation /capabilities and per-room capabilities not present |
| MSC2108 | ⭕️ | 0/0 | H | Sync over Server Sent Events | no /sync/sse or text/event-stream paths |
| MSC2102 | ⭕️ | 0/0 | M | Enforce Canonical JSON on the wire for the S2S API | no canonical-JSON wire enforcement on inbound S2S |
| MSC2061 | 🟢 | 100/100 | H | make the trailing slash on `GET /_matrix/key/v2/server/` optional | src/api/router.rs:246 routes both /key/v2/server and /server/{key_id} |
| MSC2000 | ⭕️ | 0/0 | H | MSC 2000: Proposal for server-side password policies | branch; no /password_policy endpoint or password validation |
| MSC1974 | ⭕️ | 0/0 | H | Crypto Puzzle Challenge | open; hashcash-style proof-of-work never adopted |
| MSC1973 | ⭕️ | 0/0 | H | Hash Key User ID | open; speculative scheme never adopted |
| MSC1959 | ⚫ | 0/0 | H | Sticker picker API | branch; sticker picker API on integration manager, not homeserver |
| MSC1956 | ⚫ | 0/0 | H | Integrations API | branch; integrations API is integration-manager scope, not homeserver |
| MSC1953 | 🟢 | 100/100 | H | Remove prev_content from the essential keys list | ruma redact() does not retain prev_content |
| MSC1951 | ⚫ | 0/0 | M | Custom emoji and sticker packs in Matrix | branch; client/integration manager concept; uses generic rooms |
| MSC1943 | 🟢 | 100/100 | H | Set v3 to be the default room version | default room version V11 (&gt;= v3) |
| MSC1921 | ⭕️ | 0/0 | M | Cancellation of 3pid validation tokens | 3pid cancelToken endpoints not implemented; 3pid stack stubbed |
| MSC1920 | ⚫ | 0/0 | M | Alternative texts for stickers | branch; client-side rendering field on m.sticker; no server logic |
| MSC1902 | ⚫ | ?/0 | H | Splitting the media repo into a client-side and server-side component | [→ MSC3916] |
| MSC1862 | 🟡 | 50/50 | M | Presence flag for capabilities API | presence on/off enforced; m.presence not in /capabilities response |
| MSC1849 | 🟡 | 20/40 | M | Proposal for aggregations via relations | [→ MSC2674/MSC2675/MSC2676] modern /relations API present; MSC1849 specifics ... |
| MSC1818 | 🟢 | 100/100 | H | Remove references to presence lists | presence list endpoints absent (compliant by removal) |
| MSC1797 | ⭕️ | 0/0 | H | Proposal for more granular profile error codes | branch; M_USER_NOT_FOUND/M_PROFILE_* error codes not used |
| MSC1796 | ⭕️ | 0/0 | M | Proposal for improving notifications for E2E encrypted rooms | branch; m.mentions on encrypted events not honored server-side |
| MSC1781 | ⚫ | ?/? | H | Proposal for associations for DIDs and DID names | identity-server endpoints for DID validation; not a homeserver concern |
| MSC1780 | ⭕️ | 0/0 | H | Add DIDs and DID names as admin accounts to HS | open; m.did medium not supported in 3pid endpoints |
| MSC1777 | ⭕️ | 0/0 | H | Proposal for implementing peeking over federation (server pseudousers) | branch; server pseudouser peeking not implemented |
| MSC1776 | ⭕️ | 0/0 | H | Proposal for implementing peeking via /sync in the CS API | branch; POST /sync with peek not implemented |
| MSC1769 | ⭕️ | 0/0 | H | Proposal for extensible profiles as rooms | branch; profile-as-rooms not implemented |
| MSC1768 | ⭕️ | 0/0 | H | Proposal to authenticate with public keys | open; m.login.proof.* not implemented |
| MSC1763 | ⭕️ | ?/0 | H | Proposal for specifying configurable per-room message retention periods. | no m.room.retention support; /retention/configuration endpoint absent |
| MSC1762 | ⚫ | ?/? | H | Support user-owned identifiers as new 3PID type | identity-server feature (m.did 3PID type); not a homeserver concern |
| MSC1740 | ⭕️ | ?/0 | M | Using the Accept header to select an encoding | no Accept-based content negotiation; only application/json supported |
| MSC1731 | ⭕️ | 0/0 | M | Mechanism for redirecting to an alternative server during SSO login | branch; homeserver query param on sso loginToken redirect not added |
| MSC1716 | ⭕️ | ?/0 | H | Open on device API | client-only m.openondevice event type; nothing server-side to implement |
| MSC1714 | ⭕️ | 0/0 | H | using the TLS private key to sign federation-signing keys | branch/abandoned 2018; no rsa key id, no TLS-cross-signing in src/api/server/... |
| MSC1700 | 🟢 | 80/80 | M | Improving .well-known discovery of homeservers | well-known client+server discovery served from config |
| MSC1687 | ⭕️ | ?/0 | H | Proposal for storing an encrypted recovery key on the server to aid recovery ... | no PBKDF passphrase backup logic; auth_data passes through opaquely |
| MSC1607 | 🟡 | 20/30 | M | Proposal for room alias grammar | alias parsing delegated to Ruma RoomAliasId; no NFKC/punycode/blacklist logic |
| MSC1597 | 🟡 | 20/30 | M | Grammars for identifiers in the Matrix protocol | identifier validation delegated to Ruma; proposal is exploratory |
| MSC1228 | ⭕️ | ?/0 | H | Removing MXIDs from events | removing mxids never merged; no user_room_key or pseudo IDs in src |

## Closed

Sorted by MSC number, highest first.

| MSC | Status | Correct/Impl | Conf | Title | Note |
|---|---|---|---|---|---|
| MSC4444 | ⚫ | 0/0 | H | Malicious PDUs | April Fools joke MSC, status closed |
| MSC4421 | ⚫ | 0/0 | H | Standardize the spec on US English | spec house-style proposal (en-US); no protocol surface. |
| MSC4415 | ⚫ | 0/0 | H | Make `/_matrix/client/v3/admin/whois/{userId}` only available to admins | /_matrix/client/v3/admin/whois not implemented at all in Tuwunel. |
| MSC4317 | ⭕️ | 0/0 | H | Signed profile data | signed profile data; no `m.signed` profile field handling |
| MSC4316 | ⭕️ | 0/0 | H | External cross-signing signatures with X.509 certificates and (semi-)automate... | X.509 cross-signing; no `external` signature support |
| MSC4301 | ⚫ | ?/? | H | Event capability negotiation between clients | client-to-client capability negotiation |
| MSC4294 | ⭕️ | 0/0 | H | Ignore and mass ignore invites | no ignored_inviters list, no auto invite cleanup |
| MSC4281 | ⚫ | ?/? | H | Mitigating Membership Mistakes, or "Invisible" Cryptography | closed April 1 joke MSC; client-only encryption mode |
| MSC4214 | ⭕️ | 0/0 | H | Embedding Widgets in Messages | closed MSC; m.widget event/capability not implemented |
| MSC4124 | ⭕️ | 0/0 | H | Simple Server Authorization | m.server.knock/participation auth events not implemented |
| MSC4123 | ⭕️ | 0/0 | H | Allow `knock` -&gt; `join` transition | new room version with knock to join transition not implemented |
| MSC4113 | ⭕️ | 0/0 | H | Image hashes in Policy Lists | m.policy.media_hash unknown to server (closed MSC) |
| MSC4098 | ⭕️ | 0/0 | H | Use the SCIM protocol for provisioning | SCIM user provisioning endpoints absent (closed MSC) |
| MSC4052 | ⚫ | 0/0 | H | Hiding read receipts UI in certain rooms | Pure client-side hint via m.hide_ui state event. |
| MSC4018 | ⭕️ | 0/0 | H | Reliable call membership | Reliable call membership endpoints (PUT/DELETE) not present |
| MSC4015 | ⚫ | ?/? | H | Voluntary Bot indicators | voluntary bot flag, profile and member event content |
| MSC3978 | ⭕️ | 0/0 | H | Deprecate room tagging | room tagging not deprecated; still implemented |
| MSC3975 | ⭕️ | 0/0 | H | rel_type for Replies | m.reply rel_type not handled |
| MSC3972 | ⚫ | ?/? | H | Lexicographical strings as an ordering mechanism | client-side ordering algorithm |
| MSC3969 | ⭕️ | 0/0 | H | Size limits | m.room.size_limits state event not enforced |
| MSC3968 | ⭕️ | 0/0 | H | Poorer features | m.room.event_features state event not enforced |
| MSC3945 | 🟡 | 50/50 | M | Private device names | Federation hides device names by default; CSAPI /keys/query still leaks them ... |
| MSC3887 | ⭕️ | 0/0 | M | List matching push rules | closed MSC; list-matching in event_match not implemented |
| MSC3859 | ⭕️ | 0/0 | H | Add well known media domain proposal | no m.media_server in well-known responses |
| MSC3790 | ⚫ | 0/0 | H | Register Clients | client launcher registry; client-only |
| MSC3782 | ⭕️ | 0/0 | H | Matrix public key login spec | m.login.publickey login type not implemented |
| MSC3754 | ⭕️ | 0/0 | H | Removing profile information | [→ MSC4133?] DELETE profile endpoints not exposed |
| MSC3746 | ⚫ | 0/0 | H | Render image data in reactions | [→ MSC4027] image reactions are client-only event content |
| MSC3659 | ⭕️ | 0/0 | H | Invite Rules | closed MSC; no invite_rules account data dispatch |
| MSC3588 | ⚫ | 0/0 | H | WIP: MSC3588: Encrypted Stories As Rooms | client-only feature; explicitly says no server changes required |
| MSC3517 | ⚫ | 0/0 | H | "Mention" Pushrule | [→ MSC3952] |
| MSC3464 | ⭕️ | 0/0 | H | Allow Users to Post on Behalf of Other Users | no m.on_behalf_of or m.allows_on_behalf_of handling |
| MSC3429 | ⭕️ | 0/0 | H | Individual room preview API | no /rooms/{id}/preview endpoint |
| MSC3391 | 🟢 | 100/100 | H | API to delete account data | src/api/client/account_data.rs:126; both DELETE routes via Ruma&lt;R&gt;; tombstone... |
| MSC3302 | ⚫ | ?/? | H | Stories via To-Device-Messaging | client uses generic to-device which is supported |
| MSC3286 | ⭕️ | 0/0 | H | Media spoilers | server passes events opaquely; no spoiler-aware code |
| MSC3282 | ⚫ | 0/0 | H | Expose enable_set_displayname in capabilities response | [→ MSC3283] |
| MSC3279 | ⚫ | 0/0 | H | Expose enable_set_displayname in capabilities response | [→ MSC3283] |
| MSC3270 | ⚫ | ?/? | H | Symmetric megolm backup | server stores backup auth_data/session_data opaquely |
| MSC3265 | ⚫ | ?/? | H | Login and SSSS with a Single Password | client-only construction; explicitly no server-side changes |
| MSC3255 | ⚫ | ?/? | H | Use SRV record for homeservers discovery by clients | client-side discovery via SRV; closed proposal |
| MSC3244 | ⭕️ | 0/10 | H | Room version capabilities | capabilities lacks room_capabilities knock/restricted info |
| MSC3137 | 🟢 | 100/100 | H | Define space room type, subset of MSC1772 | type:m.space in m.room.create accepted; used in directory and spaces |
| MSC3125 | ⭕️ | 0/0 | H | Limits API — Part 5: per-Instance limits | per-instance limits admin API absent |
| MSC3124 | ⚫ | ?/? | H | Handling spoilers in plain-text message fallback | client-only spoiler fallback handling |
| MSC3074 | ⚫ | 0/0 | H | Proposal for URIs conforming to RFC 3986 syntax. | client URI scheme; not a server feature |
| MSC3073 | ⭕️ | 0/0 | H | Role based access control | closed; rbac/m.role not implemented |
| MSC3068 | ⚫ | 0/0 | H | Compliance tiers | informational compliance terminology only |
| MSC3067 | ⚫ | 0/0 | H | Prevent/remove legacy groups from being in the spec | meta MSC; spec-process decision to drop legacy groups |
| MSC3053 | ⭕️ | 0/0 | H | Limits API — Part 2: per-Room limits | closed; no admin/limits endpoints or m.limits.* events |
| MSC3013 | ⭕️ | 0/0 | H | Encrypted Push | closed; no encrypted-push algorithm support |
| MSC3007 | ⭕️ | 0/0 | H | Forced insertion and room blocking by self-banning | closed; no insert_member power or /insert endpoint |
| MSC3006 | ⭕️ | 0/0 | H | Bot Interactions | closed; bot-interaction event types not implemented |
| MSC3005 | ⭕️ | 0/0 | H | Streaming Federation Events | closed; no streaming federation transport |
| MSC2957 | ⭕️ | 0/0 | H | Cryptographically Concealed Credentials | PAKE-style login flow; closed; not implemented |
| MSC2912 | ⭕️ | 0/0 | H | Setting cross-signing keys during registration | no device_signing field accepted by /register |
| MSC2876 | ⚫ | ?/? | H | Allowing widgets to read events in a room | widget read_events action; client-only |
| MSC2839 | ⭕️ | 0/0 | M | Dynamic User-Interactive Authentication | closed; UIA flows are static in Tuwunel |
| MSC2835 | ⭕️ | 0/10 | M | Add UIA to the /login endpoint | closed; /login does not consume UIA auth dict |
| MSC2810 | ⚫ | ?/? | M | Consistent globs specification | closed glob spec doc; ACLs/push rules already use existing globs |
| MSC2779 | ⚫ | ?/? | H | Clarify that event IDs are globally unique | spec clarification issue; closed; no server behavior change |
| MSC2773 | ⭕️ | 0/0 | M | Room kinds | closed; no m.kind summary or m.room.kind handling |
| MSC2771 | ⚫ | ?/? | H | Bookmarks | client-side bookmarks via account_data; closed |
| MSC2697 | ⚫ | 0/0 | M | Device dehydration | [→ MSC3814] Superseded by MSC3814 dehydration v2; closed |
| MSC2631 | 🟢 | 80/80 | M | Add `default_payload` to PusherData | ruma HttpPusherData flattens custom data; default_payload accepted via passth... |
| MSC2589 | ⚫ | ?/? | M | Improve replies | Client reply rendering; closed MSC; server ignores reply_body fields |
| MSC2579 | ⚫ | ?/? | L | Improved tagging support | Client tag-ordering account_data; server stores opaquely |
| MSC2516 | ⚫ | ?/? | M | Add a new message type for voice messages | Client-only msgtype; server does no msgtype-specific handling |
| MSC2475 | ⚫ | ?/? | L | API versioning | Spec process meta-MSC about API version naming; closed |
| MSC2463 | ⭕️ | 0/0 | M | Exclusion of MXIDs in push rules content matching | closed MSC; no MXID exclusion in push rule content matching |
| MSC2461 | ⚫ | 0/0 | M | Proposal for Authenticated Content Repository API | [→ MSC3916] |
| MSC2416 | 🟢 | 90/100 | H | Add m.login.jwt authentication type | m.login.jwt fully wired in session module |
| MSC2390 | ⚫ | ?/? | M | On the EDU-to-PDU transition. | Process MSC; closed; recommends no further EDU use |
| MSC2389 | ⚫ | ?/? | M | Toward the EDU-to-PDU transition: Typing. | Typing as PDU; closed proposal, Tuwunel uses EDU |
| MSC2376 | ⚫ | ?/? | M | Disable URL Previews | Client-only HTML attribute hint; server has no role |
| MSC2063 | ⚫ | 0/0 | M | Add "server information" public API proposal | closed; no real proposal text (template file only) |
| MSC1998 | ⭕️ | 0/0 | H | Two-Factor Authentication Providers | closed; TOTP/recovery 2FA never adopted by spec |
| MSC1958 | ⚫ | ?/? | H | Widget architecture changes | client widget account_data shape; servers do not interpret widget content |
| MSC1935 | ⚫ | 0/0 | M | Key validity enforcement | [→ MSC2076] closed; superseded by MSC2076 |
| MSC1888 | 🟢 | 90/100 | H | Proposal to send EDUs to appservices | [→ MSC2409] appservice receive_ephemeral with EDU push; src/service/sending/s... |
| MSC1840 | ⚫ | 0/0 | H | Typed rooms | closed; superseded by m.room.create type field used by MSC1772 |
| MSC1722 | ⚫ | ?/? | H | Support for displaying math(s) in messages | client-side rendering of MathML in formatted_body; servers do not interpret |
| MSC1703 | ⚫ | ?/? | H | encrypting recovery keys for online megolm backups | amendment PR to MSC1687; closed without merge |
| MSC1680 | ⚫ | ?/? | H | cross-signing of devices to simplify key verification | empty Google-doc stub; cross-signing specified in MSC1756 |
| MSC1497 | 🟢 | 100/100 | H | Advertising support of experimental features in the CS API | unstable_features map present in /_matrix/client/versions |
| MSC1425 | 🟢 | 100/100 | H | Room Versioning | room versioning fully present; STABLE_ROOM_VERSIONS in core/config |
| MSC1318 | ⚫ | ?/? | H | Proposal for Open Governance of Matrix.org | [→ MSC1779] governance proposal; not a homeserver feature |
| MSC1310 | ⚫ | ?/? | H | Proposal for a media information API | empty Google-doc stub; media info API never specified |
| MSC1267 | ⚫ | ?/? | H | Interactive key verification using short authentication strings | stub Google doc; SAS verification specified later (MSC2241+); client-only fea... |
| MSC1227 | 🟢 | 80/90 | H | Proposal for lazy-loading room members to improve initial sync speed and clie... | lazy_load_members supported via filter; service in rooms/lazy_loading |
| MSC1225 | ⚫ | ?/? | H | Extensible event types &amp; fallback in Matrix | empty Google-doc stub; extensible events specified later in MSC1767 |
| MSC1215 | ⚫ | ?/? | H | Groups as Rooms | [→ MSC1772] empty Google-doc stub; groups feature dropped in favor of Spaces |
| MSC1194 | ⚫ | ?/? | H | A way for HSes to remove bindings from ISes (aka unbind) | identity-server unbind feature; one-line proposal, abandoned |
| MSC971 | ⚫ | ?/? | H | Add groups stuff to spec | [→ MSC1772] groups stuff superseded by Spaces (MSC1772); proposal is doc link... |
| MSC688 | ⚫ | ?/? | H | Room Summaries (was: Calculate room names server-side) | stub Google doc; room summary work moved to heroes/MSC688 in spec |
| MSC455 | ⚫ | ?/? | H | Do we want to specify a matrix:// URI scheme for rooms? (SPEC-5) | [→ MSC2312] stub Google doc; matrix:// URI scheme superseded by matrix: URI (... |
| MSC441 | ⚫ | ?/? | H | Support for Reactions / Aggregations | [→ MSC2675/MSC2676] stub-only Google doc; superseded by MSC2675/MSC2676 react... |

## Unknown

Sorted by MSC number, highest first.

| MSC | Status | Correct/Impl | Conf | Title | Note |
|---|---|---|---|---|---|
| MSC4932 | ⚫ | ?/? | H |  | No proposal exists; MSC number is a typo or refers to deleted issue. |
| MSC3196 | ⚫ | ?/? | H |  | unknown MSC; no proposal text exists |
| MSC2225 | ⚫ | 0/0 | H |  | unknown MSC; no proposal text exists |
| MSC1301 | ⚫ | ?/? | H |  | unknown MSC number; no proposal exists |
| MSC1286 | ⚫ | ?/? | H |  | unknown MSC number; no proposal exists |
| MSC1236 | ⚫ | ?/? | H |  | unknown MSC number; no proposal exists |
| MSC1229 | ⚫ | ?/? | H |  | unknown MSC number; no proposal exists |
| MSC701 | ⚫ | ?/? | H |  | unknown MSC number; no proposal or PR exists |

