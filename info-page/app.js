/**
 * Nexora Technical Information Page - Interactive Logic
 */

// 12 Standardized Benchmark Scenarios matching BENCHMARKS.md exactly
const BENCHMARK_SCENARIOS = [
  {
    id: "SC-01",
    name: "Empty Browser (Start Page)",
    category: "general",
    hostRam: 6.5,
    webviewRam: 48.0,
    totalRam: 54.5,
    chromeRam: 185.0,
    edgeRam: 160.0,
    nexoraCpu: "0.18%",
    chromeCpu: "0.80%",
    technicalFactor: "Zero background news feeds, weather widgets, or cloud telemetry"
  },
  {
    id: "SC-02",
    name: "1 Static Tab (Wikipedia)",
    category: "general",
    hostRam: 9.2,
    webviewRam: 70.7,
    totalRam: 79.9,
    chromeRam: 230.0,
    edgeRam: 210.0,
    nexoraCpu: "0.18%",
    chromeCpu: "0.50%",
    technicalFactor: "In-memory Bloom filter stops tracking pixels and analytical scripts"
  },
  {
    id: "SC-03",
    name: "1 YouTube Tab (1080p60 AVC)",
    category: "media",
    hostRam: 14.1,
    webviewRam: 249.1,
    totalRam: 263.2,
    chromeRam: 420.0,
    edgeRam: 390.0,
    nexoraCpu: "3.34%",
    chromeCpu: "7.20%",
    technicalFactor: "Direct3D 11 & NVDEC zero-copy GPU video bitstream decoding"
  },
  {
    id: "SC-04",
    name: "1 YouTube Tab (4K30 VP9)",
    category: "media",
    hostRam: 16.8,
    webviewRam: 406.2,
    totalRam: 423.0,
    chromeRam: 680.0,
    edgeRam: 630.0,
    nexoraCpu: "13.20%",
    chromeCpu: "24.50%",
    technicalFactor: "Hardware bitstream decoding and managed frame buffer allocation"
  },
  {
    id: "SC-05",
    name: "5 Open Tabs (News, Wiki, Docs)",
    category: "tabs",
    hostRam: 13.5,
    webviewRam: 287.3,
    totalRam: 300.8,
    chromeRam: 780.0,
    edgeRam: 690.0,
    nexoraCpu: "0.70%",
    chromeCpu: "3.40%",
    technicalFactor: "Background timer throttling and multi-controller resource balancing"
  },
  {
    id: "SC-06",
    name: "10 Open Tabs (Active / Mixed)",
    category: "tabs",
    hostRam: 18.2,
    webviewRam: 527.0,
    totalRam: 545.2,
    chromeRam: 1450.0,
    edgeRam: 1280.0,
    nexoraCpu: "1.32%",
    chromeCpu: "6.80%",
    technicalFactor: "Multi-controller process management with zero renderer crosstalk"
  },
  {
    id: "SC-07",
    name: "10 Tabs (Smart Suspension Active)",
    category: "tabs",
    hostRam: 14.6,
    webviewRam: 72.0,
    totalRam: 86.6,
    chromeRam: 1100.0,
    edgeRam: 850.0,
    nexoraCpu: "0.26%",
    chromeCpu: "3.50%",
    technicalFactor: "Inactive controllers unmapped from physical RAM (~1.5 KB serialized in Rust)"
  },
  {
    id: "SC-08",
    name: "Streaming with DRM (Netflix 1080p)",
    category: "media",
    hostRam: 15.2,
    webviewRam: 304.4,
    totalRam: 319.6,
    chromeRam: 520.0,
    edgeRam: 460.0,
    nexoraCpu: "3.96%",
    chromeCpu: "8.50%",
    technicalFactor: "Certified Widevine CDM hardware pipeline with zero-copy texture sharing"
  },
  {
    id: "SC-09",
    name: "3-Hour Extended Browsing Session",
    category: "general",
    hostRam: 17.4,
    webviewRam: 433.8,
    totalRam: 451.2,
    chromeRam: 1800.0,
    edgeRam: 1400.0,
    nexoraCpu: "0.97%",
    chromeCpu: "5.60%",
    technicalFactor: "Periodic working set trims and SQLite WAL checkpointing prevent memory bloat"
  },
  {
    id: "SC-10",
    name: "Cold Startup Latency",
    category: "latency",
    hostRam: 6.5,
    webviewRam: 48.0,
    totalRam: 54.5,
    chromeRam: 185.0,
    edgeRam: 160.0,
    nexoraCpu: "690 ms",
    chromeCpu: "1,400 ms",
    technicalFactor: "Native Win32 compiled Rust binary initialization with no Electron shell"
  },
  {
    id: "SC-11",
    name: "Warm Session Restore (10 Tabs)",
    category: "latency",
    hostRam: 14.6,
    webviewRam: 72.0,
    totalRam: 86.6,
    chromeRam: 1100.0,
    edgeRam: 850.0,
    nexoraCpu: "368 ms",
    chromeCpu: "2,100 ms",
    technicalFactor: "Lazy Tab Pipeline: foreground tab only loaded on launch; background tabs idle"
  },
  {
    id: "SC-12",
    name: "Resident Tab Switch Latency",
    category: "latency",
    hostRam: 18.2,
    webviewRam: 527.0,
    totalRam: 545.2,
    chromeRam: 1450.0,
    edgeRam: 1280.0,
    nexoraCpu: "14 ms",
    chromeCpu: "450 ms",
    technicalFactor: "Native SetIsVisible Win32 visibility toggle with zero web reloading"
  }
];

document.addEventListener("DOMContentLoaded", () => {
  renderBenchmarkTable("all");
  setupBenchmarkFilters();
  setupVisualizer();
  setupInternalPages();
  setupShortcutSearch();
  setupCopyButtons();
  setupMobileNav();

  // Initialize Lucide icons if available
  if (window.lucide && typeof window.lucide.createIcons === "function") {
    window.lucide.createIcons();
  }
});

// Render the benchmark table based on active filter
function renderBenchmarkTable(filter) {
  const tbody = document.getElementById("benchmarkTableBody");
  if (!tbody) return;

  const filtered = filter === "all" 
    ? BENCHMARK_SCENARIOS 
    : BENCHMARK_SCENARIOS.filter(item => item.category === filter);

  tbody.innerHTML = filtered.map(item => {
    const isLatency = item.category === "latency";
    const savings = isLatency 
      ? `-${Math.round((1 - (parseFloat(item.nexoraCpu) / parseFloat(item.chromeCpu))) * 100)}% time`
      : `-${Math.round((1 - (item.totalRam / item.chromeRam)) * 100)}%`;

    const nexoraMetric = isLatency 
      ? `<span style="font-weight:600; color:var(--text-primary);">${item.nexoraCpu}</span>` 
      : `<span style="font-weight:600; color:var(--text-primary);">${item.totalRam.toFixed(1)} MB</span> <span style="font-size:0.75rem; color:var(--text-muted);">(${item.hostRam.toFixed(1)} + ${item.webviewRam.toFixed(1)})</span>`;

    const chromeMetric = isLatency ? item.chromeCpu : `${item.chromeRam.toFixed(1)} MB`;

    return `
      <tr>
        <td class="mono" style="color:var(--text-muted);">${item.id}</td>
        <td class="scenario-name">${item.name}</td>
        <td class="mono">${nexoraMetric}</td>
        <td class="mono" style="color:var(--text-secondary);">${chromeMetric}</td>
        <td class="diff-positive">${savings}</td>
        <td style="font-size:0.8rem; color:var(--text-muted);">${item.technicalFactor}</td>
      </tr>
    `;
  }).join("");
}

// Setup benchmark category filter buttons
function setupBenchmarkFilters() {
  const buttons = document.querySelectorAll(".bench-tab-btn");
  buttons.forEach(btn => {
    btn.addEventListener("click", () => {
      buttons.forEach(b => b.classList.remove("active"));
      btn.classList.add("active");
      const filter = btn.getAttribute("data-filter");
      renderBenchmarkTable(filter);
    });
  });
}

// Setup comparative memory visualizer
function setupVisualizer() {
  const select = document.getElementById("scenarioSelect");
  if (!select) return;

  select.innerHTML = BENCHMARK_SCENARIOS
    .filter(d => d.category !== "latency")
    .map(d => `<option value="${d.id}">${d.id}: ${d.name}</option>`)
    .join("");

  select.addEventListener("change", (e) => {
    updateVisualizer(e.target.value);
  });

  // Default to SC-03 (YouTube 1080p60)
  select.value = "SC-03";
  updateVisualizer("SC-03");
}

function updateVisualizer(scenarioId) {
  const item = BENCHMARK_SCENARIOS.find(d => d.id === scenarioId) || BENCHMARK_SCENARIOS[2];
  const maxVal = Math.max(item.chromeRam, item.totalRam, item.edgeRam) * 1.05;

  const hostPct = (item.hostRam / maxVal) * 100;
  const webviewPct = (item.webviewRam / maxVal) * 100;
  const chromePct = (item.chromeRam / maxVal) * 100;
  const edgePct = (item.edgeRam / maxVal) * 100;

  // Nexora split bars
  const hostBar = document.getElementById("barNexoraHost");
  const webviewBar = document.getElementById("barNexoraWebview");
  const nexoraNum = document.getElementById("numNexora");
  if (hostBar) hostBar.style.width = `${hostPct}%`;
  if (webviewBar) webviewBar.style.width = `${webviewPct}%`;
  if (nexoraNum) nexoraNum.textContent = `${item.totalRam.toFixed(1)} MB`;

  // Chrome bar
  const chromeBar = document.getElementById("barChrome");
  const chromeNum = document.getElementById("numChrome");
  if (chromeBar) chromeBar.style.width = `${chromePct}%`;
  if (chromeNum) chromeNum.textContent = `${item.chromeRam.toFixed(1)} MB`;

  // Edge bar
  const edgeBar = document.getElementById("barEdge");
  const edgeNum = document.getElementById("numEdge");
  if (edgeBar) edgeBar.style.width = `${edgePct}%`;
  if (edgeNum) edgeNum.textContent = `${item.edgeRam.toFixed(1)} MB`;

  // Savings summary text
  const savingsPct = Math.round((1 - (item.totalRam / item.chromeRam)) * 100);
  const savingsEl = document.getElementById("visualizerSavings");
  if (savingsEl) {
    savingsEl.innerHTML = `Nexora consumes <strong>${savingsPct}% less memory</strong> than Google Chrome in this workload.`;
  }
}

// Internal pages tab switcher
function setupInternalPages() {
  const tabs = document.querySelectorAll(".internal-page-tab");
  const panes = document.querySelectorAll(".page-display-pane");

  tabs.forEach(tab => {
    tab.addEventListener("click", () => {
      const pageId = tab.getAttribute("data-page");
      tabs.forEach(t => t.classList.remove("active"));
      panes.forEach(p => p.classList.remove("active"));

      tab.classList.add("active");
      const target = document.getElementById(`pane-${pageId}`);
      if (target) target.classList.add("active");
    });
  });
}

// Shortcuts search filter
function setupShortcutSearch() {
  const searchInput = document.getElementById("shortcutSearch");
  const rows = document.querySelectorAll(".shortcuts-table tr");

  if (!searchInput) return;

  searchInput.addEventListener("input", (e) => {
    const q = e.target.value.toLowerCase().trim();
    rows.forEach(row => {
      const text = row.textContent.toLowerCase();
      row.style.display = text.includes(q) ? "" : "none";
    });
  });
}

// Copy-to-clipboard for terminal code blocks
function setupCopyButtons() {
  const buttons = document.querySelectorAll(".terminal-copy");
  buttons.forEach(btn => {
    btn.addEventListener("click", () => {
      const block = btn.closest(".terminal-block");
      if (!block) return;
      const code = block.querySelector("code");
      if (!code) return;

      navigator.clipboard.writeText(code.innerText.trim()).then(() => {
        const originalHtml = btn.innerHTML;
        btn.innerHTML = `<i data-lucide="check" class="lucide" style="width:14px; height:14px; stroke:var(--emerald);"></i> Copied!`;
        if (window.lucide && typeof window.lucide.createIcons === "function") {
          window.lucide.createIcons();
        }
        setTimeout(() => {
          btn.innerHTML = originalHtml;
          if (window.lucide && typeof window.lucide.createIcons === "function") {
            window.lucide.createIcons();
          }
        }, 2000);
      });
    });
  });
}

// Mobile navigation drawer toggle
function setupMobileNav() {
  const toggle = document.querySelector(".mobile-toggle");
  const nav = document.querySelector(".nav-menu");

  if (!toggle || !nav) return;

  toggle.addEventListener("click", () => {
    nav.classList.toggle("open");
  });

  // Close when clicking a nav link
  nav.querySelectorAll("a").forEach(link => {
    link.addEventListener("click", () => {
      nav.classList.remove("open");
    });
  });
}
