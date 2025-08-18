# Jetson Orin Nano Super Integration (Notes)

- Declocked, DVFS‑limited profiles; aggressive power gating of GPU/CPU clusters.
- TensorRT/ONNX Runtime INT8 engines; batch=1, low‑latency kernels.
- IPC with OBC via UART/SPI; watchdog supervised; optional reboot on thermal trip.
- Thermal: radiator strap + thermistor feedback; duty‑cycle heavy workloads.
