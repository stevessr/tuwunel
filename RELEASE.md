# Tuwunel 1.4.4

October 15, 2025

All deployments serving ElementX, SchildiChat Next, or any client using sliding-sync must upgrade from Tuwunel 1.4.3 to this patch.

### Bug Fixes

- The sliding-sync updates in Tuwunel 1.4.3 failed to protect against the lack of idempotency in the current protocol. When sync requests are made, the server updates state which affects future sync requests. If the client interrupts or discards the result of a request, the connection will stray out of sync; messages can be missed. This fix inserts a guard to reset the server-side state upon such an expectation failure. Thank you @canarysnort01 for reporting this in (#190) as well as others who were inconvenienced by this issue. This fix is being released on an emergency basis. Future revisions will improve its efficiency. Some users may notice regressive behavior with unread-markers not disappearing instantly. Please open additional issue tickets so we can finally get this right. Thank you all for your patience and kindness through this difficult time.

### Corrections

- Release notes for 1.4.3 missed citing @boarfish55 for their participation in (#175).

All release notes are intentionally written by hand to personally thank everyone for their participation. Please let us know if anything was incorrect or omitted in these notes.
