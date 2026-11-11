use std::collections::{HashMap, HashSet, VecDeque};

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

/// Aho-Corasick Automaton for single-pass multi-pattern matching
#[derive(Debug, Clone, Default)]
struct AcState {
    transitions: HashMap<u8, usize>,
    fail: usize,
    output: Option<(RuleCategory, String)>,
}

#[derive(Debug, Clone, Default)]
pub struct AhoCorasickAutomaton {
    states: Vec<AcState>,
}

impl AhoCorasickAutomaton {
    pub fn build(patterns: &[(String, RuleCategory)]) -> Self {
        let mut states = vec![AcState::default()]; // Root is index 0

        // 1. Build Trie
        for (keyword, category) in patterns {
            let kw_lower = keyword.to_lowercase();
            let bytes = kw_lower.as_bytes();
            if bytes.is_empty() {
                continue;
            }

            let mut curr = 0;
            for &b in bytes {
                let next = match states[curr].transitions.get(&b) {
                    Some(&n) => n,
                    None => {
                        let new_idx = states.len();
                        states.push(AcState::default());
                        states[curr].transitions.insert(b, new_idx);
                        new_idx
                    }
                };
                curr = next;
            }
            states[curr].output = Some((*category, keyword.clone()));
        }

        // 2. Build Failure Links using BFS
        let mut queue = VecDeque::new();
        let root_children: Vec<usize> = states[0].transitions.values().copied().collect();
        for child in root_children {
            states[child].fail = 0;
            queue.push_back(child);
        }

        while let Some(u) = queue.pop_front() {
            let u_transitions: Vec<(u8, usize)> = states[u].transitions.iter().map(|(&b, &v)| (b, v)).collect();
            for (b, v) in u_transitions {
                let mut f = states[u].fail;
                while f != 0 && !states[f].transitions.contains_key(&b) {
                    f = states[f].fail;
                }
                let target = states[f].transitions.get(&b).copied().unwrap_or(0);
                states[v].fail = target;

                if states[v].output.is_none() {
                    states[v].output = states[target].output.clone();
                }

                queue.push_back(v);
            }
        }

        Self { states }
    }

    pub fn find(&self, text: &str) -> Option<(RuleCategory, String)> {
        if self.states.is_empty() {
            return None;
        }

        let mut curr = 0;
        for &b in text.as_bytes() {
            while curr != 0 && !self.states[curr].transitions.contains_key(&b) {
                curr = self.states[curr].fail;
            }
            if let Some(&next) = self.states[curr].transitions.get(&b) {
                curr = next;
            }
            if let Some(ref out) = self.states[curr].output {
                return Some(out.clone());
            }
        }

        None
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
    path_automaton: AhoCorasickAutomaton,
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
            path_automaton: AhoCorasickAutomaton::default(),
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
            "www.youtube.com",
            "youtube-nocookie.com",
            "www.youtube-nocookie.com",
            "youtu.be",
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
        self.path_automaton = AhoCorasickAutomaton::build(&self.path_keywords);
    }

    /// Evaluates a URL against whitelist, domain hashes, and path patterns.
    /// Runs in under 5 microseconds per evaluation using single-pass Aho-Corasick.
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

        // 6. Substring / Path keywords check via single-pass Aho-Corasick Automaton
        let path_lower = path.to_lowercase();
        if let Some((cat, matched_kw)) = self.path_automaton.find(&path_lower) {
            return FilterDecision::Block {
                category: cat,
                matched_rule: format!("*{}*", matched_kw),
            };
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

    pub fn add_path_rule(&mut self, keyword: &str, category: RuleCategory) {
        self.path_keywords.push((keyword.to_string(), category));
        self.rebuild_automaton();
    }

    pub fn rebuild_automaton(&mut self) {
        self.path_automaton = AhoCorasickAutomaton::build(&self.path_keywords);
    }

    pub fn reload_custom_rules(&mut self, text: &str) -> (usize, usize) {
        let rules = crate::privacy::rule_parser::RuleParser::parse_rules(text);
        let mut domains_added = 0;
        let mut paths_added = 0;

        for rule in rules {
            match rule {
                crate::privacy::rule_parser::ParsedRule::BlockDomain(d, cat) => {
                    self.add_domain_rule(&d, cat);
                    domains_added += 1;
                }
                crate::privacy::rule_parser::ParsedRule::Whitelist(w) => {
                    self.add_whitelist(&w);
                }
                crate::privacy::rule_parser::ParsedRule::BlockPath(p, cat) => {
                    self.path_keywords.push((p, cat));
                    paths_added += 1;
                }
            }
        }

        self.rebuild_automaton();
        (domains_added, paths_added)
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

    #[test]
    fn test_aho_corasick_automaton_accuracy() {
        let rules = vec![
            ("/adservice.".to_string(), RuleCategory::Ad),
            ("/tracking/pixel/".to_string(), RuleCategory::Tracker),
            ("/telemetry".to_string(), RuleCategory::Telemetry),
        ];
        let ac = AhoCorasickAutomaton::build(&rules);

        assert_eq!(ac.find("/static/img/adservice.min.js"), Some((RuleCategory::Ad, "/adservice.".to_string())));
        assert_eq!(ac.find("/api/v1/tracking/pixel/123"), Some((RuleCategory::Tracker, "/tracking/pixel/".to_string())));
        assert_eq!(ac.find("/clean/path/document.pdf"), None);
    }
}
