# Security Policy

Vibe Island is a local developer tool that observes AI coding-tool sessions and desktop state. Treat logs, screenshots, terminal output, and local agent state as potentially sensitive.

## Supported Versions

Security fixes target the latest `main` branch and the most recent published release.

## Reporting a Vulnerability

Please report vulnerabilities privately when possible:

- Use GitHub's **Report a vulnerability** flow for this repository if available.
- Otherwise open a minimal issue that avoids live secrets, private hostnames, tokens, or exploit payloads.
- For coordination, contact `v0idch3cksum` on Discord.

## Sensitive Data Guidelines

- Do not attach raw `.omc/`, `.serena/`, terminal cache, or session-log files without reviewing and redacting them first.
- Redact API keys, session cookies, SSH hosts, client names, and private filesystem paths.
- If a token or credential may have been captured by local state, rotate it before sharing diagnostics.
