# Basic Auth Proxy

A proxy server that validates Basic Authentication credentials against an OpenID Connect provider and returns user information in response headers.

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

1. The proxy receives Basic Authentication requests
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