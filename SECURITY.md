# Security Policy

## Supported Versions

Since VantaDB is currently in its initial `v0.1` stabilization phase, previous architectural snapshots are not managed for backported fixes. Only the current trunk is expected to remain stable.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| Pre-v0.1| :x:                |

## Reporting a Vulnerability

If you discover a memory violation or PyO3 serialization vulnerability that allows execution outside boundary mapping protections, please open an Issue with replication steps or reach out to the core maintainers privately. Do not exploit index panics visibly on untrusted vectors in production pending formal stabilization guarantees.
