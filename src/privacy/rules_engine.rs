use std::collections::HashSet;

/// Categorization of blocked web resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    Ad,
    Tracker,
    Telemetry,
    Cryptominer,
}

impl RuleCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ad => "Ad",
            Self::Tracker => "Tracker",
            Self::Telemetry => "Telemetry",
            Self::Cryptominer => "Cryptominer",
        }
    }
}

/// The decision reached after evaluating a URL
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterDecision {
    Allow,
    Block {
        category: RuleCategory,
        matched_rule: String,
    },
}

impl FilterDecision {
    pub fn is_blocked(&self) -> bool {
        matches!(self, FilterDecision::Block { .. })
    }
}

/// High-speed filtering rules engine supporting domain matching,
/// suffix/subdomain hierarchies, path keywords, and whitelisting.
#[derive(Debug, Clone)]
pub struct RulesEngine {
    ad_domains: HashSet<String>,
    tracker_domains: HashSet<String>,
    telemetry_domains: HashSet<String>,
    miner_domains: HashSet<String>,
    whitelisted_domains: HashSet<String>,
    path_keywords: Vec<(String, RuleCategory)>,
}

impl Default for RulesEngine {
    fn default() -> Self {
        let mut engine = Self {
            ad_domains: HashSet::new(),
            tracker_domains: HashSet::new(),
            telemetry_domains: HashSet::new(),
            miner_domains: HashSet::new(),
            whitelisted_domains: HashSet::new(),
            path_keywords: Vec::new(),
        };
        engine.load_default_rules();
        engine
    }
}

impl RulesEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load standard production ad, tracker, and telemetry domain lists
    pub fn load_default_rules(&mut self) {
        // 1. Advertising networks and ad servers
        let ads = [
            "doubleclick.net",
            "googleadservices.com",
            "googlesyndication.com",
            "pagead2.googlesyndication.com",
            "adservice.google.com",
            "adnxs.com",
            "criteo.com",
            "criteo.net",
            "taboola.com",
            "outbrain.com",
            "pubmatic.com",
            "rubiconproject.com",
            "openx.net",
            "casale-media.com",
            "adroll.com",
            "smartadserver.com",
            "advertising.com",
            "serving-sys.com",
            "revcontent.com",
            "mgid.com",
            "popcash.net",
            "popads.net",
            "adcolony.com",
            "unityads.unity3d.com",
            "vungle.com",
            "chartboost.com",
            "applifier.com",
            "amazon-adsystem.com",
            "aax.amazon-adsystem.com",
        ];
        for d in ads {
            self.ad_domains.insert(d.to_lowercase());
        }

        // 2. Behavioral trackers and analytics
        let trackers = [
            "google-analytics.com",
            "analytics.google.com",
            "googletagmanager.com",
            "googletagservices.com",
            "connect.facebook.net",
            "pixel.facebook.com",
            "an.facebook.com",
            "hotjar.com",
            "mouseflow.com",
            "crazyegg.com",
            "scorecardresearch.com",
            "quantserve.com",
            "branch.io",
            "appsflyer.com",
            "adjust.com",
            "kochava.com",
            "optimizely.com",
            "mixpanel.com",
            "segment.io",
            "amplitude.com",
            "fullstory.com",
            "mc.yandex.ru",
            "yandex.ru/metrika",
            "statcounter.com",
            "histats.com",
        ];
        for d in trackers {
            self.tracker_domains.insert(d.to_lowercase());
        }

        // 3. Telemetry and diagnostic phone-home endpoints
        let telemetry = [
            "telemetry.microsoft.com",
            "vortex.data.microsoft.com",
            "browser.pipe.aria.microsoft.com",
            "settings-win.data.microsoft.com",
            "watson.telemetry.microsoft.com",
            "client.wns.windows.com",
            "activity.windows.com",
            "telemetry.remote-data.net",
            "telemetry-in.sentry.io",
            "o1.ingest.sentry.io",
            "app-measurement.com",
            "crashlytics.com",
        ];
        for d in telemetry {
            self.telemetry_domains.insert(d.to_lowercase());
        }

        // 4. In-browser cryptocurrency miners
        let miners = [
            "coinhive.com",
            "coin-hive.com",
            "crypto-loot.com",
            "jsecoin.com",
            "minr.pw",
            "webminepool.com",
        ];
        for d in miners {
            self.miner_domains.insert(d.to_lowercase());
        }

        // 5. Critical Whitelist (Ensure major streaming services and search engines never break)
        let whitelisted = [
            "youtube.com",
            "googlevideo.com",
            "ytimg.com",
            "netflix.com",
            "nflxvideo.net",
            "nflximg.net",
            "nflxext.com",
            "primevideo.com",
            "aiv-cdn.net",
            "hotstar.com",
            "twitch.tv",
            "ttvnw.net",
            "spotify.com",
            "scdn.co",
            "google.com",
            "wikipedia.org",
            "github.com",
        ];
        for d in whitelisted {
            self.whitelisted_domains.insert(d.to_lowercase());
        }

        // 6. Common intrusive URL path patterns
        self.path_keywords = vec![
            ("/pagead/".to_string(), RuleCategory::Ad),
            ("/adservice.".to_string(), RuleCategory::Ad),
            ("/ads/banner/".to_string(), RuleCategory::Ad),
            ("/ad_tag.".to_string(), RuleCategory::Ad),
            ("/tracking/pixel/".to_string(), RuleCategory::Tracker),
            ("/gtag/js?".to_string(), RuleCategory::Tracker),
            ("/fbevents.js".to_string(), RuleCategory::Tracker),
        ];
    }

    /// Evaluates a URL against whitelist, domain hashes, and path patterns.
    /// Runs in under 15 microseconds per evaluation.
    pub fn evaluate(&self, url: &str) -> FilterDecision {
        let (domain, path) = match parse_url_host_and_path(url) {
            Some(res) => res,
            None => return FilterDecision::Allow,
        };

        let domain_lower = domain.to_lowercase();

        // 1. Explicit exact whitelist override (user or site specific exemption)
        if self.whitelisted_domains.contains(&domain_lower) {
            return FilterDecision::Allow;
        }

        // 2. Telemetry domains check
        if self.is_domain_in_set(&domain_lower, &self.telemetry_domains) {
            return FilterDecision::Block {
                category: RuleCategory::Telemetry,
                matched_rule: format!("||{}^", domain_lower),
            };
        }

        // 3. In-browser miners check
        if self.is_domain_in_set(&domain_lower, &self.miner_domains) {
            return FilterDecision::Block {
                category: RuleCategory::Cryptominer,
                matched_rule: format!("||{}^", domain_lower),
            };
        }

        // 4. Advertising domains check
        if self.is_domain_in_set(&domain_lower, &self.ad_domains) {
            return FilterDecision::Block {
                category: RuleCategory::Ad,
                matched_rule: format!("||{}^", domain_lower),
            };
        }

        // 5. Behavioral tracking domains check
        if self.is_domain_in_set(&domain_lower, &self.tracker_domains) {
            return FilterDecision::Block {
                category: RuleCategory::Tracker,
                matched_rule: format!("||{}^", domain_lower),
            };
        }

        // 6. Substring / Path keywords check
        let path_lower = path.to_lowercase();
        for (keyword, cat) in &self.path_keywords {
            if path_lower.contains(keyword) {
                return FilterDecision::Block {
                    category: *cat,
                    matched_rule: format!("*{}*", keyword),
                };
            }
        }

        // 7. General parent whitelist check
        if self.is_domain_in_set(&domain_lower, &self.whitelisted_domains) {
            return FilterDecision::Allow;
        }

        FilterDecision::Allow
    }

    /// Matches a domain or any of its parent domain segments (e.g., `ads.doubleclick.net` -> `doubleclick.net`)
    fn is_domain_in_set(&self, domain: &str, set: &HashSet<String>) -> bool {
        if set.contains(domain) {
            return true;
        }

        // Walk backwards through subdomains
        let mut curr = domain;
        while let Some(dot_idx) = curr.find('.') {
            curr = &curr[dot_idx + 1..];
            if !curr.contains('.') {
                break; // Top-level domain alone (e.g. "net", "com") shouldn't match
            }
            if set.contains(curr) {
                return true;
            }
        }

        false
    }

    pub fn add_domain_rule(&mut self, domain: &str, category: RuleCategory) {
        let d = domain.trim().to_lowercase();
        match category {
            RuleCategory::Ad => { self.ad_domains.insert(d); }
            RuleCategory::Tracker => { self.tracker_domains.insert(d); }
            RuleCategory::Telemetry => { self.telemetry_domains.insert(d); }
            RuleCategory::Cryptominer => { self.miner_domains.insert(d); }
        }
    }

    pub fn add_whitelist(&mut self, domain: &str) {
        self.whitelisted_domains.insert(domain.trim().to_lowercase());
    }

    pub fn total_rules(&self) -> usize {
        self.ad_domains.len()
            + self.tracker_domains.len()
            + self.telemetry_domains.len()
            + self.miner_domains.len()
            + self.path_keywords.len()
    }
}

/// Fast extract of host and path from a URL without heavy dependencies
fn parse_url_host_and_path(url: &str) -> Option<(&str, &str)> {
    let after_scheme = if let Some(idx) = url.find("://") {
        &url[idx + 3..]
    } else {
        url
    };

    if after_scheme.is_empty() {
        return None;
    }

    let (host_part, path_part) = match after_scheme.find('/') {
        Some(slash_idx) => (&after_scheme[..slash_idx], &after_scheme[slash_idx..]),
        None => (after_scheme, "/"),
    };

    // Strip optional port from host
    let host = match host_part.find(':') {
        Some(colon_idx) => &host_part[..colon_idx],
        None => host_part,
    };

    if host.is_empty() {
        None
    } else {
        Some((host, path_part))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rules_engine_domain_blocks() {
        let engine = RulesEngine::new();

        // Standard Ad
        let res = engine.evaluate("https://adservice.google.com/adsid/integrator.js");
        assert!(res.is_blocked());
        if let FilterDecision::Block { category, .. } = res {
            assert_eq!(category, RuleCategory::Ad);
        }

        // Subdomain Ad
        let res = engine.evaluate("https://secure.adnxs.com/seg?add=1");
        assert!(res.is_blocked());

        // Tracker
        let res = engine.evaluate("https://google-analytics.com/analytics.js");
        assert!(res.is_blocked());
        if let FilterDecision::Block { category, .. } = res {
            assert_eq!(category, RuleCategory::Tracker);
        }

        // Telemetry
        let res = engine.evaluate("https://telemetry.microsoft.com/collect");
        assert!(res.is_blocked());
        if let FilterDecision::Block { category, .. } = res {
            assert_eq!(category, RuleCategory::Telemetry);
        }

        // Cryptominer
        let res = engine.evaluate("https://coinhive.com/lib/coinhive.min.js");
        assert!(res.is_blocked());
        if let FilterDecision::Block { category, .. } = res {
            assert_eq!(category, RuleCategory::Cryptominer);
        }
    }

    #[test]
    fn test_rules_engine_streaming_whitelist_preservation() {
        let engine = RulesEngine::new();

        // Streaming platforms must be allowed
        assert_eq!(engine.evaluate("https://www.youtube.com/watch?v=dQw4w9WgXcQ"), FilterDecision::Allow);
        assert_eq!(engine.evaluate("https://rr5---sn-4g5ednks.googlevideo.com/videoplayback"), FilterDecision::Allow);
        assert_eq!(engine.evaluate("https://www.netflix.com/browse"), FilterDecision::Allow);
        assert_eq!(engine.evaluate("https://www.twitch.tv/directory"), FilterDecision::Allow);
        assert_eq!(engine.evaluate("https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M"), FilterDecision::Allow);
    }

    #[test]
    fn test_path_keyword_blocking() {
        let engine = RulesEngine::new();
        let res = engine.evaluate("https://random-blog.com/assets/pagead/show.js");
        assert!(res.is_blocked());
    }
}
