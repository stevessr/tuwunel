# Tuwunel 1.4.7

December 3, 2025

Tuwunel is now deployed at scale serving the citizens of Switzerland in production. Some optimizations were requested to reduce operating costs from projected growth over product lifespan: this release delivers with markedly reduced CPU usage and improves responsiveness. However, complications during routine dependency upgrades consumed valuable time planned for features and issues which could not be completed for this release.

### New Features

- Upgrade Support for Room Version 12 is now available. Though this room version has been supported for the entire 1.4 series, all Tuwunel servers have been protected by Hydra Backports on all room versions. As such, other work was able to be prioritized for the preceding releases.

### Enhancements

- Recursive relations have been enabled. This is an optimization which allows the server to gather more data using fewer client requests, for example, of a thread with many reactions and replies. The implementation is now optimal and utilizes the full capabilities of Tuwunel's asynchronous database.

- Several miscellaneous but significant optimizations took place at the direction of memory profiling. This reduced load on the allocator for database queries and JSON serialization of complex objects. Heroes calculations and the joined room hot-path on sync v3 were further optimized to reduce the database query load itself.

- Jemalloc has been repackaged with platform-specific optimizations enhancing the build. The upgrade to the dev branch of libjemalloc itself was considered as too much variability for the same release, it is planned for an upcoming release.

- Thanks to element-hq/synapse#18970 by @dasha-uwu, we have very slightly turned down the amount of randomness when selecting join-servers, More retries also occur within a single request if necessary. Thanks to @gogo199432 and @lifeofguenter for reporting problems in (#128) and (#205) respectively. More opportunities are still available to make large room joins robust.

### Bug Fixes

- Special thanks to @yefimg for fixing LDAP logout in (#231) from a report kindly made by @orhtej2 in (#97); thank you for your patience waiting for domain expertise to assist here.

- Thanks to @Radiant-Xyz the example configurations have been updated to remove `allow_check_for_updates`. This fixes any warnings for the item no longer existing. (#221)

- Thanks again to @Radiant-Xyz reporting in (#219) the `/whoami` endpoint is now returns spec-compliant errors for Mautrix bridges (fe12daead9). Thanks also to @bobobo1618 for confirming the fix is working.

- Relations responses were sometimes incorrect in the forwards direction. This was fixed by (5147b541) bringing those responses into full compliance. Note the prior release had also fixed compliance issues but in the backwards direction.

- Server selection for backfill struggled sometimes for version 12 rooms. These rooms might fail to load history after join. Additional servers are now found using `creators` and `additional_creators` instead.

- Room leave compliance has been fixed for an edge-case where a room becomes empty except for a locally invited user which does not have its leave event sent down `/sync`.

- Thanks to @grinapo for a report which lead to the discovery of events acquired over backfill not being checked for whether they already exist.

### Upcoming

- As stated in the summary, several planned items could not be cut into this release. These include SSO/OIDC support (#7), Element Call setup assistance and documentation (#217)(#215), User-level Admin Room and Media deletion (#192), and any other assigned issue. These items are on the short-list for the next cycle and mean a lot to us; to all participants: your issues are not being ignored and we hear you.
