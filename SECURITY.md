# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.2.x   | Yes       |
| < 0.2   | No        |

## Reporting a Vulnerability

**Please do NOT report security vulnerabilities through public GitHub issues.**

### Responsible Disclosure Process

1. **Email**: Send a detailed description to **security@kami-project.dev**
2. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Affected versions
   - Potential impact assessment
   - Suggested fix (if any)
3. **Response time**: We aim to acknowledge within **48 hours** and provide
   a fix timeline within **7 business days**.

### What to Expect

- We will confirm receipt of your report.
- We will investigate and assess the severity using CVSS scoring.
- We will develop and test a fix privately.
- We will coordinate disclosure with you before publishing.
- We will credit you in the advisory (unless you prefer anonymity).

### Scope

The following are in scope for security reports:

- **WASM sandbox escapes** (guest accessing host memory or resources)
- **Capability bypass** (tool accessing resources not declared in tool.toml)
- **Path traversal** in filesystem jail
- **Network allow-list bypass**
- **SQL injection** in the SQLite adapter
- **Authentication bypass** in HTTP transport
- **Denial of service** via resource exhaustion

### Out of Scope

- Vulnerabilities in upstream dependencies (report directly to maintainers)
- Issues requiring physical access to the host machine
- Social engineering attacks

## Security Model

For a comprehensive description of KAMI's security architecture, threat model,
and defense-in-depth strategy, see [docs/SECURITY.md](docs/SECURITY.md).

## Security Updates

Security fixes are released as patch versions (e.g., 0.2.1) and announced via:
- GitHub Security Advisories
- CHANGELOG.md entries marked with `[SECURITY]`
