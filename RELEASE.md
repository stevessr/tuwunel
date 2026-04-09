# Tuwunel 1.6.0

April 9, 2026

### New Features & Enhancements

- **Next-Gen Auth OIDC** server enhancing ElementX and SchildiNext has arrived! It all began only a month ago with (#342), a large draft PR by @lytedev assessed by the Tuwunel team to be several months away. What happened next was truly extraordinary. Starting with @chbgdn and followed by @siennathesane, @DonPrus and @shaba an entire project within this project assembled to test and iterate this branch at a rapid clip. The OIDC server now builds on existing infrastructure in Tuwunel previously used for SSO. If you have an Identity Provider configured already for use with SSO then the OIDC server Just Works. Huge thanks to everyone involved. (Implements MSC2964/2965/2966/2967)

- **S3 Storage support** is now available! Starting from (#362) graciously developed by @exodrifter, Tuwunel now introduces multiple media backends with configurable sections. Support currently includes S3 endpoints and local filesystem directories. The existing media directory is now itself a configurable storage provider implied by the section `[global.storage_provider.media.local]`. See the examples under `[global.storage_provider.<ID>.S3]` to configure your own S3 provider. Then list it in `media_storage_providers` to download media from it, and `store_media_on_providers` for uploading media to it. Experimental migration support is available with the `!admin query storage sync` command. SPECIAL UPDATE: Thanks to testing by @utop-top large uploads (~200 MiB) may not work for some S3 providers until additional support is added in 1.6.1. We apologize for this limitation.

- User-Interactive Authentication for SSO accounts (MSC2454) has been made possible thanks to @chbgdn in (#389). Accounts no longer require setting a password to use features protected by UIAA flows. Users wishing to disable password authentication on their account altogether may do so by changing it to a single asterisk '*' character (use the admin room commands if your client refuses this password change).

- User-Interactive Authentication for Next-gen OIDC (MSC4312) was implemented by serial auth-system contributor @chbgdn in (#405). This provides cross-signing/identity reset functionality for ElementX and co.

- Asynchronous media uploads for appservices was implemented thanks to @donjuanplatinum (MSC2246) in (#347).

- Thanks to @dasha-uwu the `appservice_dir` can be configured to a directory containing all your appservice yaml files.

- @donjuanplatinum implemented the server-side for fast-joins (MSC3706) in (#349). Thank you!

- Thanks to @ventureoo we support sockets managed by systemd after (#360) (issue #355).

- @vladexa prevented duplicate reactions from being sent by a client to maintain spec compliance with (#353), thank you!

- Thank you @alametti for adding delegation examples (e.g. example.com to matrix.example.com) to the documentation in (#352).

- Thanks to @Lama-Thematique the admin room user registration notice was improved in (#387).

- Thank you @dasha-uwu for implementing the MSC4143 endpoint.

- Thank you @dasha-uwu for removing the report score per MSC4277.

- Thank you @dasha-uwu for removing v1 send_join/leave as per MSC4376.

- RocksDB compaction details are logged for the curious in verbose logging builds.

- Numerous performance optimizations including JSON deserialization and allocator optimizations.

- Sliding-sync no longer persists subscriptions across requests.

- Configuration option `allowed_remote_server_names_experimental` added as exclusive federation allow-listing. NOTE: the `_experimental` suffix was added to indicate the logic of this feature will change in an upcoming release and the suffix will be removed. We sincerely regret this inconvenience.

### Bug Fixes

- Thank you @jameskimmel for fixing the nginx configuration for http/2 support. (#391)

- @exodrifter fixed various errors and typos in documentation (#343), some reported by @RhenCloud in (#338). Thank you both!

- @vladexa fixed systemd reloading by sending monotonic time after consultation with @rexbron. (#359) Thank you both!

- Thanks to @exodrifter the media delete range commands now have improved verbiage as of (#375).

- @yefimg fixed the UIA password flow not being advertised to LDAP users due to regression (#378). Special thanks for this!

- Thank you @proximalriver for fixing the missing `server` keyword in the nginx example. (#383)

- @chbgdn fixed the m.change_password capability not being set based on `login_with_password`. (#388) Thank you!

- Thank you @centromere for reporting cross-platform build regressions in #357 which were fixed.

- Thank you @Ada-lave for reporting a regression with admin startup commands in #320 which we fixed.

- @0x1af2aec8f957 reported the new systemd-friendly listener system required reuse-address flags to be set (#374). Thank you for reporting!

- Thank you @Batmaev for reporting non-compliant minimum timeout was imposed on sliding-sync in (#402) which was corrected.

- @dasha-uwu fixed admin room upgrade to work as expected. @dfuchss inspired with (#361) among many other informal reports. We appreciate the effort of everyone involved on this!

- @tycrek reported the conduit user is involved in `force-join-all-local-users` commands (#373) which was fixed thanks to @dasha-uwu.

- Thanks to @dasha-uwu bugs and compliance regarding `initial_state` during room creation were addressed.
