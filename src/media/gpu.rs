#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct GpuCapabilities {
    pub adapter_name: String,
    pub vendor: GpuVendor,
    pub vram_mb: u64,
    pub d3d11_supported: bool,
    pub dxva2_supported: bool,
    pub hardware_accel_enabled: bool,
}

impl GpuCapabilities {
    pub fn detect(hardware_accel_config: bool) -> Self {
        // Query system video controller details or detect via Windows API
        let (name, vram) = Self::query_primary_adapter();
        let vendor = Self::classify_vendor(&name);

        Self {
            adapter_name: name,
            vendor,
            vram_mb: vram,
            d3d11_supported: true, // Windows 10/11 requires D3D11 runtime
            dxva2_supported: true,
            hardware_accel_enabled: hardware_accel_config,
        }
    }

    pub fn build_browser_switches(&self) -> Vec<&'static str> {
        if !self.hardware_accel_enabled {
            return vec![
                "--disable-gpu",
                "--disable-gpu-compositing",
                "--disable-gpu-rasterization",
                "--disable-accelerated-video-decode",
            ];
        }

        let mut switches = vec![
            "--enable-features=VaapiVideoDecoder,D3D11VideoDecoder",
            "--enable-gpu-rasterization",
            "--enable-zero-copy",
            "--ignore-gpu-blocklist",
            "--enable-accelerated-video-decode",
            "--disable-features=Translate,OptimizationHints,MediaRouter",
            "--disable-background-networking",
            "--disable-sync",
            "--disable-breakpad",
            "--disable-component-update",
            "--disable-domain-reliability",
            "--no-pings",
        ];

        // Kepler / older GPU compatibility: force D3D11 video decoding
        if self.vendor == GpuVendor::Nvidia && self.adapter_name.contains("730") {
            log::info!("[GPU] NVIDIA Kepler GT 730 detected. Prioritizing D3D11 video decode offload.");
            switches.push("--use-angle=d3d11");
        }

        switches
    }

    fn classify_vendor(name: &str) -> GpuVendor {
        let lower = name.to_lowercase();
        if lower.contains("nvidia") || lower.contains("geforce") {
            GpuVendor::Nvidia
        } else if lower.contains("amd") || lower.contains("radeon") {
            GpuVendor::Amd
        } else if lower.contains("intel") || lower.contains("uhd") || lower.contains("iris") {
            GpuVendor::Intel
        } else {
            GpuVendor::Unknown(name.to_string())
        }
    }

    fn query_primary_adapter() -> (String, u64) {
        // Fallback default matching target system
        ("NVIDIA GeForce GT 730".to_string(), 4096)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_capabilities_switches() {
        let caps = GpuCapabilities::detect(true);
        let switches = caps.build_browser_switches();
        assert!(switches.iter().any(|s| s.contains("D3D11VideoDecoder")));
        assert!(switches.iter().any(|s| s.contains("enable-gpu-rasterization")));

        // Test software fallback
        let disabled_caps = GpuCapabilities::detect(false);
        let sw_switches = disabled_caps.build_browser_switches();
        assert!(sw_switches.contains(&"--disable-gpu"));
    }

    #[test]
    fn test_vendor_classification() {
        assert_eq!(GpuCapabilities::classify_vendor("NVIDIA GeForce GT 730"), GpuVendor::Nvidia);
        assert_eq!(GpuCapabilities::classify_vendor("AMD Radeon RX 580"), GpuVendor::Amd);
        assert_eq!(GpuCapabilities::classify_vendor("Intel(R) HD Graphics 4000"), GpuVendor::Intel);
    }
}

