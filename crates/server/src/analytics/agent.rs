//! The device class and the browser family of a reader, from the User-Agent.

use cuhlumn_api::analytics::{Browser, Device};

/// The classification rules for User-Agent texts.
#[derive(Clone, Copy, Debug)]
pub struct Agent;

/// The device class and the browser family of a reader.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Client {
    pub device: Device,
    pub browser: Browser,
}

/// The User-Agent parts of crawlers, scripts, and automated browsers.
const BOTS: &[&str] = &[
    "bot",
    "crawl",
    "spider",
    "headless",
    "curl",
    "wget",
    "python",
    "go-http-client",
    "java/",
    "okhttp",
];

/// The browser rules in order. The first rule with a matching part wins.
const BROWSERS: &[(Browser, &[&str])] = &[
    (Browser::Edge, &["edg/"]),
    (Browser::Opera, &["opr/"]),
    (Browser::Samsung, &["samsungbrowser"]),
    (Browser::Firefox, &["firefox/", "fxios"]),
    (Browser::Chrome, &["chrome/", "crios"]),
    (Browser::Safari, &["safari/"]),
];

impl Agent {
    /// Returns the client of a User-Agent, or `None` when the User-Agent is empty or names a bot.
    #[must_use]
    pub fn classify(user_agent: &str) -> Option<Client> {
        let text = user_agent.trim().to_ascii_lowercase();
        if text.is_empty() || BOTS.iter().any(|part| text.contains(part)) {
            return None;
        }
        Some(Client {
            device: device(&text),
            browser: browser(&text),
        })
    }
}

/// Returns the device class of a lowercase User-Agent.
fn device(text: &str) -> Device {
    let android = text.contains("android");
    if text.contains("ipad") || (android && !text.contains("mobile")) {
        return Device::Tablet;
    }
    if text.contains("mobi") || text.contains("iphone") || android {
        return Device::Mobile;
    }
    Device::Desktop
}

/// Returns the browser family of a lowercase User-Agent.
fn browser(text: &str) -> Browser {
    BROWSERS
        .iter()
        .find(|(_, parts)| parts.iter().any(|part| text.contains(part)))
        .map_or(Browser::Other, |(browser, _)| *browser)
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::analytics::{Browser, Device};

    use crate::analytics::agent::{Agent, Client};

    #[test]
    fn classifies_browsers_and_devices() {
        let cases = [
            (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
                Device::Desktop,
                Browser::Chrome,
            ),
            (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.0.0",
                Device::Desktop,
                Browser::Edge,
            ),
            (
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 OPR/115.0.0.0",
                Device::Desktop,
                Browser::Opera,
            ),
            (
                "Mozilla/5.0 (X11; Linux x86_64; rv:132.0) Gecko/20100101 Firefox/132.0",
                Device::Desktop,
                Browser::Firefox,
            ),
            (
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Safari/605.1.15",
                Device::Desktop,
                Browser::Safari,
            ),
            (
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1",
                Device::Mobile,
                Browser::Safari,
            ),
            (
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/130.0.0.0 Mobile/15E148 Safari/604.1",
                Device::Mobile,
                Browser::Chrome,
            ),
            (
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/132.0 Mobile/15E148 Safari/605.1.15",
                Device::Mobile,
                Browser::Firefox,
            ),
            (
                "Mozilla/5.0 (Linux; Android 14; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/26.0 Chrome/122.0.0.0 Mobile Safari/537.36",
                Device::Mobile,
                Browser::Samsung,
            ),
            (
                "Mozilla/5.0 (Linux; Android 14; SM-X710) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
                Device::Tablet,
                Browser::Chrome,
            ),
            (
                "Mozilla/5.0 (iPad; CPU OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1",
                Device::Tablet,
                Browser::Safari,
            ),
            ("Lynx/2.9.0 libwww-FM/2.14", Device::Desktop, Browser::Other),
        ];
        for (user_agent, device, browser) in cases {
            assert_eq!(
                Agent::classify(user_agent),
                Some(Client { device, browser }),
                "{user_agent}"
            );
        }
    }

    #[test]
    fn rejects_bots_and_empty_agents() {
        let bots = [
            "",
            "   ",
            "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
            "Mozilla/5.0 AppleWebKit/537.36 (KHTML, like Gecko; compatible; bingbot/2.0)",
            "Mozilla/5.0 (Windows NT 10.0) AppleWebKit/537.36 HeadlessChrome/130.0.0.0 Safari/537.36",
            "ia_archiver crawler",
            "Spider-Harbor/1.0",
            "curl/8.10.1",
            "Wget/1.24.5",
            "python-requests/2.32.3",
            "Go-http-client/2.0",
            "Java/21.0.5",
            "okhttp/4.12.0",
        ];
        for user_agent in bots {
            assert_eq!(Agent::classify(user_agent), None, "{user_agent}");
        }
    }
}
