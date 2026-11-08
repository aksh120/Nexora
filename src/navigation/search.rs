pub struct SearchEngine;

impl SearchEngine {
    pub fn build_url(engine_name: &str, query: &str) -> String {
        let encoded = Self::percent_encode(query.trim());
        match engine_name.to_lowercase().as_str() {
            "bing" => format!("https://www.bing.com/search?q={}", encoded),
            "duckduckgo" | "ddg" => format!("https://duckduckgo.com/?q={}", encoded),
            "brave" => format!("https://search.brave.com/search?q={}", encoded),
            "ecosia" => format!("https://www.ecosia.org/search?q={}", encoded),
            "youtube" => format!("https://www.youtube.com/results?search_query={}", encoded),
            _ => format!("https://www.google.com/search?q={}", encoded), // Default to Google
        }
    }

    pub fn percent_encode(input: &str) -> String {
        let mut result = String::with_capacity(input.len() * 3);
        for byte in input.bytes() {
            match byte {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                b' ' => {
                    result.push('+');
                }
                _ => {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_url_generation() {
        assert_eq!(
            SearchEngine::build_url("Google", "rust webview2"),
            "https://www.google.com/search?q=rust+webview2"
        );
        assert_eq!(
            SearchEngine::build_url("DuckDuckGo", "nexora browser"),
            "https://duckduckgo.com/?q=nexora+browser"
        );
        assert_eq!(
            SearchEngine::build_url("Bing", "high performance"),
            "https://www.bing.com/search?q=high+performance"
        );
        assert_eq!(
            SearchEngine::build_url("Brave", "privacy search"),
            "https://search.brave.com/search?q=privacy+search"
        );
        assert_eq!(
            SearchEngine::build_url("YouTube", "4k 60fps nature"),
            "https://www.youtube.com/results?search_query=4k+60fps+nature"
        );
    }

    #[test]
    fn test_percent_encode() {
        assert_eq!(SearchEngine::percent_encode("hello world!"), "hello+world%21");
        assert_eq!(SearchEngine::percent_encode("c++ & rust"), "c%2B%2B+%26+rust");
    }
}

