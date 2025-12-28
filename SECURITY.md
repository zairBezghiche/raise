# Security Policy

As a sovereign infrastructure for critical engineering ("Digital Commons"), **GenAptitude** takes security seriously. We encourage responsible disclosure of vulnerabilities.

## Supported Versions

Only the latest stable release is currently supported with security updates.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

**Do NOT report security vulnerabilities through public GitHub issues.**

If you believe you have found a security vulnerability in the GenAptitude core (JSON-DB, Blockchain, AI Engine), please report it to our security team.

### How to Report

Please email **security@genaptitude.com** with the subject `[SECURITY] Vulnerability Report`.

Include as much of the following information as possible:

- Type of issue (e.g., buffer overflow, SQL injection, XSS).
- Full paths of source file(s) related to the manifestation of the issue.
- The location of the affected source code (tag/branch/commit or direct URL).
- Any special configuration required to reproduce the issue.
- Step-by-step instructions to reproduce the issue.
- Proof-of-concept or exploit code (if possible).
- Impact of the issue, including how an attacker might exploit the issue.

### Our Response

We will acknowledge receipt of your vulnerability report within **48 hours** and strive to send you regular updates about our progress.

## Sovereignty & Data Privacy

Since GenAptitude is a "Local-First" application:

- We generally classify issues that leak data to external servers as **Critical**.
- Issues requiring physical access to the user's workstation are generally classified as **Medium/Low**, depending on the impact.

Thank you for helping us make sovereign engineering safe.
