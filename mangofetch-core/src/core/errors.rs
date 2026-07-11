/// Strongly-typed download error classification.
#[derive(Debug, Clone)]
pub enum DownloadError {
    AuthRequired,
    RateLimited,
    Restricted,
    FileMissing,
    NotFound,
    FfmpegNeeded,
    YtdlpNeeded,
    YtdlpOutdated,
    Unknown(String),
}

impl DownloadError {
    /// Get the error code for UI/logging.
    pub fn code(&self) -> &'static str {
        match self {
            DownloadError::AuthRequired => "auth_required",
            DownloadError::RateLimited => "rate_limited",
            DownloadError::Restricted => "restricted",
            DownloadError::FileMissing => "file_missing",
            DownloadError::NotFound => "not_found",
            DownloadError::FfmpegNeeded => "ffmpeg_needed",
            DownloadError::YtdlpNeeded => "ytdlp_needed",
            DownloadError::YtdlpOutdated => "ytdlp_outdated",
            DownloadError::Unknown(_) => "unknown",
        }
    }

    /// Get the user-facing hint message.
    pub fn hint(&self) -> &'static str {
        match self {
            DownloadError::AuthRequired => "This content requires login. Install the browser extension and visit the site while logged in.",
            DownloadError::RateLimited => "Too many requests. Try again in a few minutes.",
            DownloadError::Restricted => "This content is private or age-restricted.",
            DownloadError::FileMissing => "Downloaded file could not be located in the output folder.",
            DownloadError::NotFound => "Content not found or has been deleted.",
            DownloadError::FfmpegNeeded => "FFmpeg is required for this download. Install it from Settings.",
            DownloadError::YtdlpNeeded => "yt-dlp is required. Install it from Settings.",
            DownloadError::YtdlpOutdated => "yt-dlp needs updating. Restart the app to auto-update.",
            DownloadError::Unknown(_) => "",
        }
    }

    /// Classify error from a message string.
    pub fn from_message(error: &str) -> Self {
        let lower = error.to_lowercase();

        // Auth errors (check for specific auth indicators, not generic "cookie" mentions)
        if (lower.contains("authentication") && lower.contains("fail"))
            || lower.contains("login")
            || lower.contains("sign in")
            || lower.contains("403")
        {
            return DownloadError::AuthRequired;
        }

        if lower.contains("captcha")
            || lower.contains("blocking")
            || lower.contains("rate limit")
            || lower.contains("429")
            || lower.contains("too many")
        {
            return DownloadError::RateLimited;
        }

        if lower.contains("private") || lower.contains("restricted") || lower.contains("age") {
            return DownloadError::Restricted;
        }

        if lower.contains("downloaded file") && lower.contains("not found") {
            return DownloadError::FileMissing;
        }

        if lower.contains("not found")
            || lower.contains("404")
            || lower.contains("unavailable")
            || lower.contains("deleted")
        {
            return DownloadError::NotFound;
        }

        if lower.contains("ffmpeg") || lower.contains("mux") || lower.contains("merge") {
            return DownloadError::FfmpegNeeded;
        }

        if lower.contains("yt-dlp") || lower.contains("ytdlp") || lower.contains("no downloader") {
            return DownloadError::YtdlpNeeded;
        }

        if lower.contains("nsig") || lower.contains("signature") || lower.contains("cipher") {
            return DownloadError::YtdlpOutdated;
        }

        DownloadError::Unknown(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_required() {
        let err = DownloadError::from_message("Please login to continue");
        assert_eq!(err.code(), "auth_required");
        let err = DownloadError::from_message("HTTP Error 403: Forbidden");
        assert_eq!(err.code(), "auth_required");
        let err = DownloadError::from_message("authentication failed");
        assert_eq!(err.code(), "auth_required");
    }

    #[test]
    fn test_rate_limited() {
        let err = DownloadError::from_message("Too many requests");
        assert_eq!(err.code(), "rate_limited");
        let err = DownloadError::from_message("HTTP Error 429");
        assert_eq!(err.code(), "rate_limited");
        let err = DownloadError::from_message("captcha required");
        assert_eq!(err.code(), "rate_limited");
    }

    #[test]
    fn test_restricted() {
        let err = DownloadError::from_message("This video is private");
        assert_eq!(err.code(), "restricted");
        let err = DownloadError::from_message("age-restricted content");
        assert_eq!(err.code(), "restricted");
    }

    #[test]
    fn test_file_missing() {
        let err = DownloadError::from_message("downloaded file not found on disk");
        assert_eq!(err.code(), "file_missing");
    }

    #[test]
    fn test_not_found() {
        let err = DownloadError::from_message("404 Not Found");
        assert_eq!(err.code(), "not_found");
        let err = DownloadError::from_message("The video has been deleted");
        assert_eq!(err.code(), "not_found");
        let err = DownloadError::from_message("content unavailable");
        assert_eq!(err.code(), "not_found");
    }

    #[test]
    fn test_ffmpeg_needed() {
        let err = DownloadError::from_message("ffmpeg is required");
        assert_eq!(err.code(), "ffmpeg_needed");
        let err = DownloadError::from_message("error while muxing");
        assert_eq!(err.code(), "ffmpeg_needed");
    }

    #[test]
    fn test_ytdlp_needed() {
        let err = DownloadError::from_message("yt-dlp is missing");
        assert_eq!(err.code(), "ytdlp_needed");
        let err = DownloadError::from_message("no downloader found");
        assert_eq!(err.code(), "ytdlp_needed");
    }

    #[test]
    fn test_ytdlp_outdated() {
        let err = DownloadError::from_message("Unable to extract nsig");
        assert_eq!(err.code(), "ytdlp_outdated");
        let err = DownloadError::from_message("signature decryption failed");
        assert_eq!(err.code(), "ytdlp_outdated");
    }

    #[test]
    fn test_unknown() {
        let err = DownloadError::from_message("some random error");
        assert_eq!(err.code(), "unknown");
        match err {
            DownloadError::Unknown(msg) => assert_eq!(msg, "some random error"),
            _ => panic!("Expected Unknown variant"),
        }
    }

    #[test]
    fn test_case_insensitivity() {
        let err = DownloadError::from_message("LOGIN REQUIRED");
        assert_eq!(err.code(), "auth_required");
    }

    #[test]
    fn test_no_false_positive_cookie() {
        // "cookies from browser" should not match as auth error
        let err = DownloadError::from_message("Using cookies from browser");
        assert_eq!(err.code(), "unknown");
    }
}
