use crate::navigation::search::SearchEngine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationTarget {
    Url(String),
    Search {
        engine: String,
        query: String,
        url: String,
    },
    Internal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityBadge {
    Secure,
    Insecure,
    Internal,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionType {
    TopSite,
    History,
    Bookmark,
    SearchQuery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmniboxSuggestion {
    pub title: String,
    pub url: String,
    pub suggestion_type: SuggestionType,
}

pub struct Omnibox {
    top_sites: Vec<(&'static str, &'static str)>,
}

impl Default for Omnibox {
    fn default() -> Self {
        Self::new()
    }
}

impl Omnibox {
    pub fn new() -> Self {
        Self {
            top_sites: vec![
                ("YouTube", "https://www.youtube.com"),
                ("Netflix", "https://www.netflix.com"),
                ("Amazon Prime Video", "https://www.primevideo.com"),
                ("JioHotstar", "https://www.hotstar.com"),
                ("Twitch", "https://www.twitch.tv"),
                ("Spotify", "https://open.spotify.com"),
                ("Sony LIV", "https://www.sonyliv.com"),
                ("ZEE5", "https://www.zee5.com"),
                ("GitHub", "https://github.com"),
                ("Reddit", "https://www.reddit.com"),
                ("Wikipedia", "https://www.wikipedia.org"),
            ],
        }
    }

    pub fn parse_input(raw: &str, default_engine: &str) -> NavigationTarget {
        let input = raw.trim();
        if input.is_empty() {
            return NavigationTarget::Url("about:blank".to_string());
        }

        // 1. Internal custom browser URLs
        if input.starts_with("nexora://") || input.starts_with("about:") || input.starts_with("edge://") {
            return NavigationTarget::Internal(input.to_string());
        }

        // 2. Direct scheme URL (http://, https://, file://)
        if input.starts_with("http://") || input.starts_with("https://") || input.starts_with("file://") {
            return NavigationTarget::Url(input.to_string());
        }

        // 3. Localhost and loopback IPs
        if input == "localhost"
            || input.starts_with("localhost:")
            || input.starts_with("127.0.0.1")
            || input.starts_with("[::1]")
        {
            return NavigationTarget::Url(format!("http://{}", input));
        }

        // 4. Check if it's a domain name (contains a dot, no whitespace, valid TLD/structure)
        if !input.contains(' ') && input.contains('.') {
            let parts: Vec<&str> = input.split('/').next().unwrap_or(input).split('.').collect();
            if parts.len() >= 2 {
                let last = parts.last().unwrap_or(&"");
                // Check if last part looks like a TLD or port
                let is_likely_tld = (2..=12).contains(&last.len())
                    && last.chars().all(|c| c.is_alphabetic() || c.is_ascii_digit());
                let has_port = input.contains(':');

                if is_likely_tld || has_port {
                    return NavigationTarget::Url(format!("https://{}", input));
                }
            }
        }

        // 5. Fallback: Search Engine Query
        let search_url = SearchEngine::build_url(default_engine, input);
        NavigationTarget::Search {
            engine: default_engine.to_string(),
            query: input.to_string(),
            url: search_url,
        }
    }

    pub fn get_security_badge(url: &str) -> SecurityBadge {
        if url.starts_with("https://") {
            SecurityBadge::Secure
        } else if url.starts_with("http://") {
            if url.starts_with("http://localhost") || url.starts_with("http://127.0.0.1") {
                SecurityBadge::Local
            } else {
                SecurityBadge::Insecure
            }
        } else if url.starts_with("nexora://") || url.starts_with("about:") {
            SecurityBadge::Internal
        } else if url.starts_with("file://") {
            SecurityBadge::Local
        } else {
            SecurityBadge::Insecure
        }
    }

    pub fn get_suggestions(&self, input: &str, default_engine: &str) -> Vec<OmniboxSuggestion> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let lower = trimmed.to_lowercase();
        let mut suggestions = Vec::new();

        // 1. Search Query suggestion always first or second
        let search_url = SearchEngine::build_url(default_engine, trimmed);
        suggestions.push(OmniboxSuggestion {
            title: format!("Search {} for \"{}\"", default_engine, trimmed),
            url: search_url,
            suggestion_type: SuggestionType::SearchQuery,
        });

        // 2. Pre-seeded popular streaming & developer top sites
        for (name, url) in &self.top_sites {
            let domain = url.trim_start_matches("https://").trim_start_matches("www.");
            if domain.contains(&lower) || name.to_lowercase().contains(&lower) {
                suggestions.push(OmniboxSuggestion {
                    title: name.to_string(),
                    url: url.to_string(),
                    suggestion_type: SuggestionType::TopSite,
                });
            }
        }

        // Limit to top 6 suggestions for sleek compact display
        suggestions.truncate(6);
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input_direct_urls() {
        assert_eq!(
            Omnibox::parse_input("https://youtube.com", "Google"),
            NavigationTarget::Url("https://youtube.com".to_string())
        );
        assert_eq!(
            Omnibox::parse_input("http://example.com/test", "Google"),
            NavigationTarget::Url("http://example.com/test".to_string())
        );
        assert_eq!(
            Omnibox::parse_input("github.com", "Google"),
            NavigationTarget::Url("https://github.com".to_string())
        );
        assert_eq!(
            Omnibox::parse_input("sub.domain.co.uk/stream?id=123", "Google"),
            NavigationTarget::Url("https://sub.domain.co.uk/stream?id=123".to_string())
        );
        assert_eq!(
            Omnibox::parse_input("localhost:3000", "Google"),
            NavigationTarget::Url("http://localhost:3000".to_string())
        );
        assert_eq!(
            Omnibox::parse_input("127.0.0.1:8080/app", "Google"),
            NavigationTarget::Url("http://127.0.0.1:8080/app".to_string())
        );
    }

    #[test]
    fn test_parse_input_search_queries() {
        match Omnibox::parse_input("fast rust browser", "Google") {
            NavigationTarget::Search { engine, query, url } => {
                assert_eq!(engine, "Google");
                assert_eq!(query, "fast rust browser");
                assert_eq!(url, "https://www.google.com/search?q=fast+rust+browser");
            }
            other => panic!("Expected Search target, got: {:?}", other),
        }

        match Omnibox::parse_input("what is playready drm?", "DuckDuckGo") {
            NavigationTarget::Search { engine, query, url } => {
                assert_eq!(engine, "DuckDuckGo");
                assert_eq!(query, "what is playready drm?");
                assert_eq!(url, "https://duckduckgo.com/?q=what+is+playready+drm%3F");
            }
            other => panic!("Expected Search target, got: {:?}", other),
        }
    }

    #[test]
    fn test_parse_internal_urls() {
        assert_eq!(
            Omnibox::parse_input("nexora://settings", "Google"),
            NavigationTarget::Internal("nexora://settings".to_string())
        );
        assert_eq!(
            Omnibox::parse_input("about:blank", "Google"),
            NavigationTarget::Internal("about:blank".to_string())
        );
    }

    #[test]
    fn test_security_badge() {
        assert_eq!(Omnibox::get_security_badge("https://google.com"), SecurityBadge::Secure);
        assert_eq!(Omnibox::get_security_badge("http://insecure.site"), SecurityBadge::Insecure);
        assert_eq!(Omnibox::get_security_badge("http://localhost:8000"), SecurityBadge::Local);
        assert_eq!(Omnibox::get_security_badge("file:///C:/path/file.html"), SecurityBadge::Local);
        assert_eq!(Omnibox::get_security_badge("nexora://diagnostics"), SecurityBadge::Internal);
    }

    #[test]
    fn test_suggestions() {
        let omnibox = Omnibox::new();
        let suggestions = omnibox.get_suggestions("you", "Google");
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::SearchQuery);
        // YouTube should match "you"
        let has_youtube = suggestions.iter().any(|s| s.title == "YouTube");
        assert!(has_youtube, "YouTube should be suggested for 'you'");
    }
}
