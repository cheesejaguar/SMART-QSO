## OPTIMIZE_JETSON.md — Power Optimization Guide for Jetson Orin Nano ("Super")

This guide consolidates practical levers to minimize average and peak power draw on NVIDIA Jetson Orin Nano–class modules (including higher‑bin “Super” variants). It prioritizes changes that are robust in headless deployments and measurable with standard tools.

### 0) Measure first (baseline and regressions)
- Tegrastats: `sudo tegrastats --interval 1000` (watch `POM_5V_IN`, `CPU`, `GPU`, `EMC`)
- jtop: `sudo -E jtop`
- External power meter or in-line shunt for absolute Wh/J per task
- Log before/after; change one variable at a time

### 1) System power profiles (nvpmodel) and clocks
- Query/set profiles: `sudo nvpmodel -q` / `sudo nvpmodel -m <mode>`
- Use conservative mode in cruise; create a custom JSON if needed
- Avoid `jetson_clocks` in low-power ops (pins max clocks)

### 2) CPU DVFS, core parking, governors
- Offline big cores during cruise; re-enable for bursts:
  - `echo 0 | sudo tee /sys/devices/system/cpu/cpu5/online` (repeat for cpu4..)
- Governors: `echo powersave | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor`
- Cap max freq: `echo 1200000 | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_max_freq`

### 3) GPU/accelerator usage
- Prefer TensorRT INT8/FP8 with layer fusion; quantize offline
- Duty-cycle GPU (batch work, then idle); let DVFS downclock
- If DLA present on your SKU, prefer it for steady inference

### 4) Memory/EMC frequency
- Cap EMC when possible (paths vary by L4T): `/sys/kernel/debug/clk/emc/*` or devfreq nodes
- Reduce memory traffic: fused kernels, smaller tensors, zero-copy paths

### 5) Headless display stack
- Disable GUI: `sudo systemctl set-default multi-user.target`; disable display managers
- Headless EGL only; no X/Wayland; mask vendor GUI helpers

### 6) I/O and peripherals
- Wi‑Fi/BT: `rfkill block all` when unused; else enable powersave (`iw set power_save on`)
- Ethernet: disable WOL; bring interface down when idle
- USB autosuspend: `sudo powertop --auto-tune` and udev rules
- Disable unused buses (I2C/SPI/UART/CSI) in DT or at runtime
- Power‑gate sensors via regulator GPIO when idle

### 7) Video/vision
- Use NVENC/NVDEC; lower resolution/fps/bitrate; avoid constant preview
- Batch inference (every N frames); early exit when idle

### 8) Filesystem/logging/services
- Mount `noatime,nodiratime`; tmpfs for ephemeral logs; trim journald
- Disable unneeded services (cups, avahi, bluetooth, ModemManager, …)
- Keep container base minimal; avoid background cron

### 9) Thermal & fan policy
- Lower fan PWM during cruise while respecting thermal headroom
- Improve heatsink to reduce fan duty

### 10) Duty cycling & scheduling
- Gate heavy tasks by SOC/sunlit; cgroups to cap CPU quotas
- Prefer sleeps/waits over polling; batch I/O and telemetry

### 11) Radios & network
- Reduce link rate/channel width; disable 11ac/ax when not needed
- Use UDP for telemetry; coalesce packets; avoid chatty keepalives
- Schedule TX windows; pre-connect to avoid long scans

### 12) Board rails & PMIC
- Ensure unused regulators are off in DT; validate 5V/3V3 AUX consumers
- Remove LEDs/expansion power; high-side switches for sensor rails

### 13) Kernel cpuidle/deep idle
- Ensure cpuidle enabled; verify deepest states in cpuidle sysfs
- Avoid high-frequency timers in user space (tickless idle default)

### 14) Tooling & automation
- Powertop autotune: `sudo powertop --auto-tune`
- Tegrastats CSV logging; jtop exports
- Systemd unit to apply on boot:
```ini
# /etc/systemd/system/jetson-lowpower.service
[Unit]
Description=Jetson low-power settings
After=multi-user.target

[Service]
Type=oneshot
ExecStart=/usr/local/sbin/jetson_lowpower.sh
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
```
Script `/usr/local/sbin/jetson_lowpower.sh`:
```bash
#!/usr/bin/env bash
set -euo pipefail
sudo nvpmodel -m 0 || true
for cpu in /sys/devices/system/cpu/cpu[0-9]*; do
  echo powersave | sudo tee $cpu/cpufreq/scaling_governor >/dev/null || true
  echo 1200000 | sudo tee $cpu/cpufreq/scaling_max_freq >/dev/null || true
done
for cpu in 5 4; do
  [ -e /sys/devices/system/cpu/cpu$cpu/online ] && echo 0 | sudo tee /sys/devices/system/cpu/cpu$cpu/online >/dev/null || true
done
if ip link show wlan0 >/dev/null 2>&1; then
  sudo iw dev wlan0 set power_save on || true
fi
sudo systemctl set-default multi-user.target || true
sudo systemctl disable --now gdm lightdm sddm 2>/dev/null || true
sudo powertop --auto-tune 2>/dev/null || true
```

### 15) ML specifics
- INT8 TensorRT, prune/quantize, reduce context/sequence/batch
- Fuse pre/post-processing to minimize memory traffic

### 16) Rollback & verification
- Keep a “performance” service to restore cores/clocks for bursts
- Verify thermals and function after any caps

### Quick checklist
- nvpmodel capped; CPU powersave + core parking + max_freq cap
- GPU/EMC ceilings not overpinned; DVFS active
- Headless mode; radios off or powersave
- Peripherals power‑gated; services trimmed
- Duty-cycled workloads; tegrastats/jtop logs show sustained savings

### References
- NVIDIA L4T / Jetson docs (Orin series)
- tegrastats, jtop, nvpmodel
- TensorRT Best Practices

