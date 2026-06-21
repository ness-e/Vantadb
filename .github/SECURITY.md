# Security Policy

## Supported Versions

VantaDB follows **semantic versioning**. Security patches are provided for the latest minor release only.

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Active |
| < 0.1   | ❌        |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, report via **GitHub Security Advisories**:
https://github.com/ness-e/Vantadb/security/advisories/new

This is a private form visible only to the maintainer. You will receive a response within **3 business days**.

### What to include

- Description of the vulnerability
- Steps to reproduce (proof of concept preferred)
- Affected versions
- Impact assessment
- Suggested mitigation (if any)

## Disclosure Policy

We follow a **90-day coordinated disclosure** timeline:

1. **Day 0–3:** Acknowledgment of receipt
2. **Day 3–10:** Triage and validation
3. **Day 10–90:** Fix development and testing, including coordinated embargo with downstream consumers
4. **Day 90+:** Public disclosure via:
   - GitHub Release notes
   - [RustSec Advisory Database](https://rustsec.org)
   - CVE assignment (when applicable)

Extensions beyond 90 days are considered if:
- The fix is complex and active progress is visible
- Coordinating with ecosystem stakeholders requires additional time
- A partial mitigation can be released at day 90 to break the exploitation chain

## Coordinated Disclosure

For vulnerabilities requiring coordination with downstream consumers, an embargo list may be used. Embargo notifications are sent **3–30 working days** before public disclosure and include:

- CVE identifier
- Description and affected versions
- Patch availability
- Scheduled disclosure date

To be added to the embargo list as a downstream consumer, contact the maintainer via the advisory process above.

## Security Advisories

Published advisories can be found at:
https://github.com/ness-e/Vantadb/security/advisories

## Threat Model

VantaDB is an **embedded database engine** that runs in-process. The primary trust boundary lies between VantaDB and the host application:

- **Network input:** The optional HTTP server (`vanta-cli server`) processes untrusted network data via a bounded axum router with rate limiting and optional authentication
- **File I/O:** Database files (`*.vanta`, `*.wal`, `*.bin`) are local filesystem artifacts. Corruption or malicious replacement of these files can lead to data loss or undefined behavior
- **Python FFI:** The Python SDK uses PyO3 with GIL management and NaN/Inf input validation
- **CLI arguments:** All CLI inputs are parsed via clap with bounded argument length
