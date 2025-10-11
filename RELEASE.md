# Tuwunel 1.4.3

October 10, 2025

### Featured

**Sync Tokens have been eliminated** now with stateless sync. Users should notice a reduction in database size after running this version. Long-time users, including from conduwuit and even Conduit will benefit the most. New users may not notice reductions, but nevertheless will be preventing database growth going forward. With the prior addition of room deletions courtesy of @dasha-uwu, only enhanced media retention remains between Tuwunel and sustainable cost-of-ownership.

**Sliding Sync has been fixed (Element X)** after a third pass was made to install an entirely new architecture based on the latest evolution of the highly active MSC4186. For background, the first work done earlier this year was for optimization without logical changes to what was inherited from conduwuit. The existing implementation worked by all appearances, but didn't meet specific production quality demands. The second pass made last month to rapidly prepare this passé implementation for production use against highly developed modern clients did not turn out well. More tests passed; fewer things worked. This time the core logic had been rewritten. These three iterations have now modernized the entire module to keep up with the final stages of the specification's development. It is still not perfect, so your input and issue reports are greatly appreciated as always.

### Enhancements

- Configuration options passed after arguments `--option` or `-O` now accept dots to address the TOML tables found in the config file. Thanks to the suggestion by @lucat1 while trying to configure `ldap.bind_password_file` from the command line (#162). This was separately uncovered by @andrewerf trying to configure the `tls` section (#167).

- Configuring `log_to_stderr` now provides an explicit way to redirect logging and tracing to stderr. This is often requested for use with systemd.

- The `!admin query raw` commands no longer require the redundant `raw-` prefix on every command name.

- Thanks to @SophiaH67 for pointing out that our new verbose-logging docker images aren't very useful without being pushed to registries, then taking the initiative to host it themselves until we corrected that.

- After a report by @munkinasack in (#186), @dasha-uwu determined we could solve a lot of recurring network issues by clearing the well-known cache entry for a destination that becomes unreachable.

- Thanks to @dasha-uwu for general improvements, refactoring and fixes for the room alias and presence subsystems.

- Thanks to a report by @ohitsdylan a cryptic error message from the DNS resolver has now been caught and reworded to indicate a missing or empty `/etc/resolv.conf` (#179).

- Inspired by @boarfish offering due confusion about our many build variants in (#175), some documentation about `x86_64-v1` `-v2` and `-v3` has been improved. Special thanks to @Hashbrown777 for providing a quick one-liner shell script which made its way to the documentation.

- Various performance improvements took place in s2s request handlers, and the ubiquitous matrix event `Pdu` structure.

- Nix builds have been added to CI.

### Bug Fixes

- Thanks to @harharlinks for reporting the Code of Conduct link on our github-pages was 404. Also thanks to @Tronde for reporting additional broken links in (#165). This helped discover pages had stopped deploying at some point and needed fixing.

- Thanks to @agx for contributing a fix for the systemd units on multiple platforms which contained unknown or deprecated keys (#168). And thanks again for adding missing documentation to the well-known sections of the example configuration which many users will greatly appreciate (#173).

- @mitch9911:matrix.org reported the `device_signing/upload` endpoint was omitted when adding JWT as a UIAA flow to other API's. This was subsequently patched (#169).

- The report by @orhtej2 of an invalid `?via` parameter sent by FluffyChat when joining a version 12 room was greatly appreciated, even though there was nothing more we could do on our end (#171). Thanks to @dasha-uwu for triaging and investigating this issue to conclusion.

- Thanks to a follow-up by @alaviss in (#176) the DNS-passthru feature was found to still be enforcing a large minimum-TTL for its DNS cache. This was subsequently corrected.

- Thank you @canarysnort01 for the apropos and rather surgical off-by-one fix to sliding-sync range selection in (#188). Unfortunately this entire unit had to be rewritten for the latest iteration of sliding-sync, but the fix carried value in any case to improve the rewrite.

### Notices

It has come to our attention courtesy of @andrewerf that the Arch packages are built with `--no-default-features`. This may be a problem for an ideal experience. The backstory is that conduwuit underwent a "feature skew" over its lifetime which still remains today: our default-features are basically minimal requirements, while `--all-features` should be default features. Let us first take a moment to reiterate our gratitude to AUR package maintainers @drrossum and @kimiblock who have supported this project from the first hours of its existence. No action is required on their part as the plan now is to remove several optional features by the next release to make `--no-default-features` viable. We still recommend default features in general unless this conflicts with AUR policies or philosophies.

