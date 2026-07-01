---
title: OpenTelemetry
type: glossary-entry
status: active
tags: [observability, metrics, tracing]
aliases: [OpenTelemetry, OTel, opentelemetry]
description: "A collection of APIs, SDKs, and tools used to instrument, generate, collect, and export telemetry data."
links: "[[README.md]]"
---

# OpenTelemetry (OTel)

OpenTelemetry is an open-source observability framework for generating and managing telemetry data (traces, metrics, and logs).

## Integration in VantaDB

VantaDB implements OpenTelemetry to provide full visibility into its inner workings. When operating as a server or a complex embedded instance, VantaDB exports telemetry data that allows operators to profile [[latency]], throughput, and vector recall.

It acts as a backend-agnostic layer that bridges the internal `tracing` crate events to systems like Jaeger, Prometheus, or Datadog.

## See Also
- [[BENCHMARKS.md]]
