# Tuwunel 1.4.8

December 21, 2025

All federating deployments must upgrade to this patch for mitigations to severe vulnerabilities in Matrix protocol implementation logic. This is an off-schedule coordinated security release. Full release notes will be included with the next scheduled release.

### Security Fixes

- Requests to the [Federation Invite API](https://spec.matrix.org/v1.17/server-server-api/#put_matrixfederationv2inviteroomideventid) lacked sufficient validation on all input fields. An attacker can use this route to process other kinds of events: upon acceptance, they are signed by the victim's server as specified by the Matrix protocol. The attacker can therefore forge events on behalf of the victim's authority to gain control of a room. This vulnerability was present in all versions and derivatives of Conduit.
