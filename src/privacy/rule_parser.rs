//! AdBlock Plus & uBlock Origin Rule Parser
//!
//! Parses standard adblocking filter list syntax:
//! - Domain blocks: `||example.com^`
//! - Whitelist exemptions: `@@||safe-site.com^`
//! - Substring / path blocks: `/ads/banner/*`
//! - Comments and metadata ignored: `! Comment`, `[Adblock Plus 2.0]`

use crate::privacy::rules_engine::RuleCategory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedRule {
    BlockDomain(String, RuleCategory),
    Whitelist(String),
    BlockPath(String, RuleCategory),
}

pub struct RuleParser;

impl RuleParser {
    pub fn parse_line(line: &str) -> Option<ParsedRule> {
        let trimmed = line.trim();

        // 1. Ignore blank lines and comments
        if trimmed.is_empty() || trimmed.starts_with('!') || trimmed.starts_with('#') || trimmed.starts_with('[') {
            return None;
        }

        // 2. Whitelist exception rules: `@@||domain^` or `@@https://...`
        if let Some(rest) = trimmed.strip_prefix("@@") {
            let candidate = if let Some(domain_part) = rest.strip_prefix("||") {
                domain_part
            } else {
                rest.trim_start_matches("http://").trim_start_matches("https://")
            };
            let without_options = candidate.split('$').next().unwrap_or(candidate);
            let without_path = without_options.split('/').next().unwrap_or(without_options);
            let clean = without_path.trim_end_matches('^').trim().to_lowercase();
            if !clean.is_empty() {
                return Some(ParsedRule::Whitelist(clean));
            }
            return None;
        }

        // 3. Domain block rules: `||domain^`
        if let Some(rest) = trimmed.strip_prefix("||") {
            let without_options = rest.split('$').next().unwrap_or(rest);
            let without_path = without_options.split('/').next().unwrap_or(without_options);
            let domain_part = without_path.trim_end_matches('^').trim().to_lowercase();

            if !domain_part.is_empty() {
                let cat = if domain_part.contains("track") || domain_part.contains("analytics") {
                    RuleCategory::Tracker
                } else if domain_part.contains("telemetry") {
                    RuleCategory::Telemetry
                } else if domain_part.contains("mine") || domain_part.contains("crypto") {
                    RuleCategory::Cryptominer
                } else {
                    RuleCategory::Ad
                };
                return Some(ParsedRule::BlockDomain(domain_part, cat));
            }
        }

        // 4. Path / Substring block rules
        if trimmed.starts_with('/') || trimmed.starts_with('*') || trimmed.contains("banner") || trimmed.contains("ad_") || trimmed.contains("pixel") {
            let clean = trimmed.trim_matches('*').to_lowercase();
            if clean.len() >= 3 {
                let cat = if clean.contains("track") || clean.contains("pixel") {
                    RuleCategory::Tracker
                } else {
                    RuleCategory::Ad
                };
                return Some(ParsedRule::BlockPath(clean, cat));
            }
        }

        None
    }

    pub fn parse_rules(text: &str) -> Vec<ParsedRule> {
        text.lines().filter_map(Self::parse_line).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain_block() {
        assert_eq!(
            RuleParser::parse_line("||doubleclick.net^"),
            Some(ParsedRule::BlockDomain("doubleclick.net".into(), RuleCategory::Ad))
        );
        assert_eq!(
            RuleParser::parse_line("||analytics.google.com^$third-party"),
            Some(ParsedRule::BlockDomain("analytics.google.com".into(), RuleCategory::Tracker))
        );
    }

    #[test]
    fn test_parse_whitelist() {
        assert_eq!(
            RuleParser::parse_line("@@||youtube.com^"),
            Some(ParsedRule::Whitelist("youtube.com".into()))
        );
    }

    #[test]
    fn test_parse_path_rules() {
        assert_eq!(
            RuleParser::parse_line("/ads/banner/*"),
            Some(ParsedRule::BlockPath("/ads/banner/".into(), RuleCategory::Ad))
        );
    }

    #[test]
    fn test_ignore_comments() {
        assert_eq!(RuleParser::parse_line("! Title: EasyList"), None);
        assert_eq!(RuleParser::parse_line("# Comment"), None);
        assert_eq!(RuleParser::parse_line("[Adblock Plus 2.0]"), None);
    }
}

