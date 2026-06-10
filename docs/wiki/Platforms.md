# 🌍 Platform Support Registry

MangoFetch employs a hybrid resolution strategy: it utilizes **Native Rust Extractors** for maximum speed, CPU efficiency, and rate-limit avoidance on popular platforms, while transparently falling back to the generic **yt-dlp** engine for general website compatibility.

## 🥇 Native High-Performance Extractors

These platforms bypass heavy python execution loops. MangoFetch queries API endpoints directly or parses HTML/JSON configurations using optimized Rust components:

| Platform | Native Features & Scope |
| :--- | :--- |
| **YouTube** | Videos, Shorts, Playlists, Channels, and live-stream formats. |
| **Instagram** | High-speed Reels, Stories, grid posts, and Carousel media extraction. |
| **TikTok** | Direct extraction of video streams, including slideshow photo-video formats. |
| **Twitter / X** | Video and GIF extraction from standard post URLs, including VX wrappers (`vxtwitter.com`). |
| **Reddit** | Sound-muxed video downloads from Reddit galleries and direct native hosts (`v.redd.it`). |
| **Twitch** | Live stream segments and clip extraction. |
| **Pinterest** | Video pins and media boards extraction. |
| **Vimeo** | High-definition embed and direct player streams. |
| **Bilibili** | Multi-quality video and audio stream separation support (`bilibili.com`, `b23.tv`). |
| **Bluesky** | Embedded video player feeds and thread images. |
| **Telegram** | Direct download links from public channel post pages (`t.me`). |

### 🎓 Educational & Course Platform Extractor Support
MangoFetch includes specialized native extraction patterns to securely query video files on popular e-learning and creator platform engines:
*   **Hotmart**
*   **Udemy**
*   **Kiwify**
*   **Gumroad**
*   **Teachable**
*   **Kajabi**
*   **Skool**
*   **Thinkific**
*   **Wondrium / Great Courses**
*   **Rocketseat**

---

## 🥈 Generic Extractor (yt-dlp)

For any domain not listed above, MangoFetch automatically delegates metadata resolution to **yt-dlp**. This supports over **1000+ additional platforms**, including:
- Facebook
- SoundCloud
- DailyMotion
- Streamable
- And hundreds of local news broadcasts and hosting portals.

---

## 🧲 Advanced Protocol Support

MangoFetch supports native protocols beyond standard HTTP video URLs:

### 📡 P2P Share Codes
- **Scheme**: `p2p:<unique-code>`
- Direct peer-to-peer file sharing between two MangoFetch instances. Automatically schedules multi-threaded secure TCP streaming with zero-copy buffer optimization.

### 🔗 Magnet Links & Torrents
- **Scheme**: `magnet:?xt=urn:btih:...` or local `.torrent` files.
- Uses the embedded **`librqbit`** library to handle direct torrent swarm connections, file piece validation, and download assembly natively in Rust.
