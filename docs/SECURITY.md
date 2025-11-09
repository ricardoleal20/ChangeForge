# Security Policy

This document describes how to report vulnerabilities responsibly and what to expect during triage and coordinated disclosure.

## Scope

- Vulnerabilities include issues that could allow:
  - Privilege escalation, remote code execution, injection, sensitive data exposure, authentication or authorization bypass.
  - Deserialization attacks, SSRF, CSRF, XSS, SQLi/NoSQLi, path traversal, RCE, and significant DoS.
- Out of scope:
  - Issues caused by insecure user environment configuration.
  - Reports without a proof of concept, duplicates of already reported vulnerabilities, or purely theoretical impacts without a practical vector.
  - Unsupported versions or clearly marked example/demo artifacts.

## Reporting a Vulnerability

Please do not open a public issue to report vulnerabilities.

1. Use the repository’s GitHub Security Advisories to submit a private, coordinated report (Security → Report a vulnerability).
2. When possible, include:
   - A clear description of the problem and its impact.
   - Affected versions and environment details.
   - Reproduction steps and/or a proof of concept.
   - Any temporary mitigations, if applicable.

We will review the report as promptly as possible, keep you informed during the investigation, and coordinate a responsible disclosure.

## Handling Process

1. Receive and triage the report.
2. Reproduce the issue and assess impact and scope.
3. Develop, test, and validate a fix.
4. Coordinate responsible disclosure:
   - Publish patches and/or releases.
   - Provide researcher credit if desired.

## Responsible Disclosure

Please keep vulnerability details confidential until a fix is available and a reasonable upgrade window has passed. We appreciate responsible and collaborative reporting.

## Cryptography and Secrets

Do not include real secrets or sensitive data in reports. If sensitive material is required to reproduce the issue, note it in your advisory report and we will coordinate an appropriate private channel within the Security Advisories workflow.

## Questions

If you have questions about this policy or the reporting process, start a draft Security Advisory with your questions and we will respond in that private channel.


