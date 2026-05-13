# Security Policy

Torel is pre-release compiler infrastructure. Security-sensitive areas include parsing untrusted source files, package resolution, dependency fetching, generated code, unsafe/FFI boundaries, and future registry services.

## Reporting a Vulnerability

Please report suspected vulnerabilities privately by opening a GitHub security advisory for this repository when available.

If advisory reporting is unavailable, contact the maintainers through the ByteCraft-Co GitHub organization and avoid posting exploit details in public issues.

## Supported Versions

Torel has not shipped a stable release yet. Security fixes currently target `main`.

## Expectations

Reports are most useful when they include:

- affected commit or version
- reproduction steps
- expected impact
- suggested mitigation, if known

Please do not include destructive payloads or data from systems you do not own or have permission to test.
