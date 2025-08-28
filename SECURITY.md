# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

Please report security vulnerabilities by creating a private security advisory on GitHub or emailing the maintainers.

## Security Features

- **API Key Authentication**: All multi-tenant endpoints require valid API keys
- **User Isolation**: Each user's data and execution context is isolated
- **Quota Limits**: Rate limiting prevents abuse
- **Input Validation**: All inputs are validated against schemas
- **No Direct Execution**: Agent only proposes changes via PRs, never executes directly

## Known Limitations

- API keys are currently stored in memory (not persistent)
- No key rotation mechanism implemented
- Rate limiting is per-user, not global
