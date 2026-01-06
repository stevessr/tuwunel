# Reverse Proxy Setup - Traefik

[<= Back to Generic Deployment Guide](generic.md#setting-up-the-reverse-proxy)

## Installation

_This section is incomplete. Please consider contributing to it._

## Configuration

_This section is incomplete. Please consider contributing to it._

> [!IMPORTANT]
>
> [Encoded Character Filtering](https://doc.traefik.io/traefik/security/request-path/#encoded-character-filtering)
> options must be set to `true`.

## Verification

After starting Traefik, verify it's working by checking:

```bash
curl https://your.server.name/_tuwunel/server_version
curl https://your.server.name:8448/_tuwunel/server_version
```

---

[=> Continue with "You're Done"](generic.md#you-are-done)
