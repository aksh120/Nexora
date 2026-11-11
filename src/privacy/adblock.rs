use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use crate::privacy::rules_engine::{FilterDecision, RuleCategory, RulesEngine};

/// Snapshot of blocked resource statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AdBlockStats {
    pub total_inspected: usize,
    pub blocked_ads: usize,
    pub blocked_trackers: usize,
    pub blocked_telemetry: usize,
    pub blocked_miners: usize,
}

impl AdBlockStats {
    pub fn total_blocked(&self) -> usize {
        self.blocked_ads + self.blocked_trackers + self.blocked_telemetry + self.blocked_miners
    }

    pub fn block_rate_percent(&self) -> f64 {
        if self.total_inspected == 0 {
            0.0
        } else {
            (self.total_blocked() as f64 / self.total_inspected as f64) * 100.0
        }
    }
}

/// High-speed native Ad & Tracker Blocker Engine
#[derive(Debug)]
pub struct AdBlockEngine {
    pub enabled: bool,
    pub block_trackers: bool,
    rules: RwLock<RulesEngine>,
    total_inspected: AtomicUsize,
    blocked_ads: AtomicUsize,
    blocked_trackers: AtomicUsize,
    blocked_telemetry: AtomicUsize,
    blocked_miners: AtomicUsize,
}

impl Default for AdBlockEngine {
    fn default() -> Self {
        Self {
            enabled: true,
            block_trackers: true,
            rules: RwLock::new(RulesEngine::new()),
            total_inspected: AtomicUsize::new(0),
            blocked_ads: AtomicUsize::new(0),
            blocked_trackers: AtomicUsize::new(0),
            blocked_telemetry: AtomicUsize::new(0),
            blocked_miners: AtomicUsize::new(0),
        }
    }
}

impl AdBlockEngine {
    pub fn new(enabled: bool, block_trackers: bool) -> Self {
        Self {
            enabled,
            block_trackers,
            rules: RwLock::new(RulesEngine::new()),
            total_inspected: AtomicUsize::new(0),
            blocked_ads: AtomicUsize::new(0),
            blocked_trackers: AtomicUsize::new(0),
            blocked_telemetry: AtomicUsize::new(0),
            blocked_miners: AtomicUsize::new(0),
        }
    }

    /// Evaluates whether an outgoing web resource request should be blocked.
    /// Updates atomic performance statistics.
    pub fn should_block(&self, url: &str) -> (bool, Option<RuleCategory>) {
        if !self.enabled {
            return (false, None);
        }

        self.total_inspected.fetch_add(1, Ordering::Relaxed);

        let decision = if let Ok(guard) = self.rules.read() {
            guard.evaluate(url)
        } else {
            FilterDecision::Allow
        };

        match decision {
            FilterDecision::Allow => (false, None),
            FilterDecision::Block { category, ref matched_rule } => {
                // If trackers are specifically disabled, allow tracker requests
                if category == RuleCategory::Tracker && !self.block_trackers {
                    return (false, None);
                }

                match category {
                    RuleCategory::Ad => {
                        self.blocked_ads.fetch_add(1, Ordering::Relaxed);
                    }
                    RuleCategory::Tracker => {
                        self.blocked_trackers.fetch_add(1, Ordering::Relaxed);
                    }
                    RuleCategory::Telemetry => {
                        self.blocked_telemetry.fetch_add(1, Ordering::Relaxed);
                    }
                    RuleCategory::Cryptominer => {
                        self.blocked_miners.fetch_add(1, Ordering::Relaxed);
                    }
                }

                log::debug!(
                    "[AdBlock] BLOCKED [{}] {} (rule: {})",
                    category.as_str(),
                    url,
                    matched_rule
                );

                (true, Some(category))
            }
        }
    }

    /// Retrieve current blocking statistics
    pub fn get_stats(&self) -> AdBlockStats {
        AdBlockStats {
            total_inspected: self.total_inspected.load(Ordering::Relaxed),
            blocked_ads: self.blocked_ads.load(Ordering::Relaxed),
            blocked_trackers: self.blocked_trackers.load(Ordering::Relaxed),
            blocked_telemetry: self.blocked_telemetry.load(Ordering::Relaxed),
            blocked_miners: self.blocked_miners.load(Ordering::Relaxed),
        }
    }

    pub fn reset_stats(&self) {
        self.total_inspected.store(0, Ordering::Relaxed);
        self.blocked_ads.store(0, Ordering::Relaxed);
        self.blocked_trackers.store(0, Ordering::Relaxed);
        self.blocked_telemetry.store(0, Ordering::Relaxed);
        self.blocked_miners.store(0, Ordering::Relaxed);
    }

    pub fn add_custom_rule(&self, domain: &str, category: RuleCategory) {
        if let Ok(mut guard) = self.rules.write() {
            guard.add_domain_rule(domain, category);
        }
    }

    pub fn add_whitelist_domain(&self, domain: &str) {
        if let Ok(mut guard) = self.rules.write() {
            guard.add_whitelist(domain);
        }
    }

    pub fn reload_custom_rules(&self, text: &str) -> (usize, usize) {
        if let Ok(mut guard) = self.rules.write() {
            guard.reload_custom_rules(text)
        } else {
            (0, 0)
        }
    }

    pub fn total_rules(&self) -> usize {
        if let Ok(guard) = self.rules.read() {
            guard.total_rules()
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adblock_engine_blocking_and_stats() {
        let engine = AdBlockEngine::new(true, true);

        // Allowed request
        let (blocked, cat) = engine.should_block("https://news.ycombinator.com/item?id=12345");
        assert!(!blocked);
        assert_eq!(cat, None);

        // Blocked Ad
        let (blocked, cat) = engine.should_block("https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js");
        assert!(blocked);
        assert_eq!(cat, Some(RuleCategory::Ad));

        // Blocked Tracker
        let (blocked, cat) = engine.should_block("https://google-analytics.com/analytics.js");
        assert!(blocked);
        assert_eq!(cat, Some(RuleCategory::Tracker));

        // Blocked Telemetry
        let (blocked, cat) = engine.should_block("https://telemetry.microsoft.com/v2/track");
        assert!(blocked);
        assert_eq!(cat, Some(RuleCategory::Telemetry));

        let stats = engine.get_stats();
        assert_eq!(stats.total_inspected, 4);
        assert_eq!(stats.blocked_ads, 1);
        assert_eq!(stats.blocked_trackers, 1);
        assert_eq!(stats.blocked_telemetry, 1);
        assert_eq!(stats.total_blocked(), 3);
        assert_eq!(stats.block_rate_percent(), 75.0);
    }

    #[test]
    fn test_adblock_engine_disabled() {
        let engine = AdBlockEngine::new(false, false);
        let (blocked, _) = engine.should_block("https://doubleclick.net/ad.js");
        assert!(!blocked, "When disabled, requests must not be blocked");
    }
}
