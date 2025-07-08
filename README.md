# Basic Auth Proxy

A proxy server that validates Basic Authentication credentials against an OpenID Connect provider and returns user information in response headers. This proxy is specifically designed for use with **Caddy's forward auth proxy**, especially for its [**WebDAV** module](https://github.com/mholt/caddy-webdav). Other reverse proxies
(Nginx, Traefik) would probably work as well.

## Why This Proxy?

### WebDAV Authentication Challenges

WebDAV clients typically only support Basic Authentication, but many modern applications use OAuth2/OpenID Connect for authentication. This creates a gap where:

1. **WebDAV clients** (like macOS Finder, Windows Explorer, mobile apps) can only send Basic Auth credentials
2. **Modern applications** use OAuth2/OpenID Connect tokens
3. **Caddy's forward proxy** needs to validate these Basic Auth credentials against an OIDC provider

### The Solution

This proxy bridges that gap by:
1. Receiving Basic Authentication requests from WebDAV clients
2. Validating the username/password against your OpenID Connect provider
3. Returning user information in headers that Caddy can use for authorization decisions
4. Enabling seamless WebDAV access with modern authentication

Is is inspired by and is meant to complement [OAuth2 Proxy](https://oauth2-proxy.github.io/).

### Use Case: Caddy Forward Auth + WebDAV

```yaml
# Caddy configuration example
webdav.example.com {
        webdav {
                root /srv/
        }

        forward_auth * basic-auth-proxy:8080 {
                copy_headers Authorization
                uri /
        }
}
```

The proxy validates credentials and returns headers like:
- `X-Auth-Request-User` - User ID
- `X-Auth-Request-Email` - User's email
- `X-Auth-Request-Groups` - User's groups

Caddy can then use these headers for additional authorization logic.

## Environment Variables

- `LISTEN_ADDR` - Address to listen on (default: "0.0.0.0:8080")
- `OIDC_ISSUER` - OpenID Connect issuer URL (required)
- `OIDC_CLIENT_ID` - OpenID Connect client ID (required)
- `OIDC_CLIENT_SECRET` - OpenID Connect client secret (required)
- `GROUPS_CLAIM` - Name of the claim containing user groups (optional)
- `ADDITIONAL_SCOPES` - Additional scopes to request, comma-separated (optional, defaults to empty list)

## Example Usage

### Basic configuration
```yaml
version: '3.8'
services:
  basic-auth-proxy:
    build: .
    ports:
      - "8080:8080"
    environment:
      - OIDC_ISSUER=https://your-oidc-provider.com
      - OIDC_CLIENT_ID=your-client-id
      - OIDC_CLIENT_SECRET=your-client-secret
```

### With groups claim and additional scopes
```yaml
version: '3.8'
services:
  basic-auth-proxy:
    build: .
    ports:
      - "8080:8080"
    environment:
      - OIDC_ISSUER=https://your-oidc-provider.com
      - OIDC_CLIENT_ID=your-client-id
      - OIDC_CLIENT_SECRET=your-client-secret
      - GROUPS_CLAIM=groups
      - ADDITIONAL_SCOPES=offline_access,read:users
```

## How it works

1. The proxy receives Basic Authentication requests from WebDAV clients
2. It validates the username/password against the OpenID Connect provider using the Resource Owner Password flow
3. If authentication succeeds, it returns a 200 OK response with user information in headers
4. If authentication fails, it returns a 401 Unauthorized response

## Response Headers

On successful authentication, the proxy returns the following headers with user information:

- `X-Auth-Request-User` - User ID (subject)
- `X-Auth-Request-Email` - User's email address (if available)
- `X-Auth-Request-Preferred-Username` - User's preferred username (if available)
- `X-Auth-Request-Groups` - Comma-separated list of user groups (if groups claim is configured)

## Scopes

The proxy will request the following scopes by default:
- `openid`
- `email`
- `profile`

Additional scopes can be specified via the `ADDITIONAL_SCOPES` environment variable as a comma-separated list.

## Groups Claim

If `GROUPS_CLAIM` is set, the proxy will extract user groups from the specified claim in the ID token or user info response. The groups will be included in the `X-Auth-Request-Groups` header.

## Development

To run locally during development:

```bash
cargo run
```

Make sure to set the required environment variables before running.