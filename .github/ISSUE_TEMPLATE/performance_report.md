---
name: Performance Issue / Benchmark Anomaly
about: Report high memory usage, CPU spikes, or video frame drops
title: "[PERF]: "
labels: ["performance", "benchmark"]
assignees: ""
---

### Performance Issue Description
Describe the performance regression, excessive memory usage, high CPU load, or dropped video frames.

### Streaming or Website URL
URL where the anomaly occurs (e.g. YouTube 4K stream, Netflix, complex web application).

### Observed Metrics
- **RAM Usage (Working Set)**: [e.g. 850 MB]
- **CPU Utilization %**: [e.g. 35% on i7-3770]
- **GPU Decode Engine Activity %**: [From Task Manager -> GPU -> Video Decode]
- **Dropped Video Frames**: [If streaming video, from YouTube "Stats for nerds"]

### Target Hardware
- **CPU**:
- **RAM**:
- **GPU & Driver Version**:
- **Display Resolution & Refresh Rate**: [e.g. 1920x1080 @ 60Hz]

### Benchmark Harness Output
If you ran `tests/benchmark.ps1`, attach the output summary or JSON report here.

