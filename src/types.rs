use anyhow::{Context, Result, bail};
use regex::Regex;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Represents a YouTube video with strongly-typed fields
#[derive(Debug, Clone, PartialEq)]
pub struct Video {
    pub id: VideoId,
    pub title: String,
    pub channel: Channel,
}

impl Video {
    /// Create a new Video instance
    pub fn new(id: VideoId, title: impl Into<String>, channel: Channel) -> Self {
        Self {
            id,
            title: title.into(),
            channel,
        }
    }

    /// Parse a video from a title-id string (e.g., "Video Title-abc123xyz")
    /// The ID is expected to be the last 11 characters matching YouTube ID pattern
    pub fn from_title_id_string(s: &str) -> Option<Self> {
        let id_regex = Regex::new(r"[A-Za-z0-9_-]{11}$").unwrap();

        if let Some(caps) = id_regex.captures(s) {
            let id_str = caps[0].to_string();
            let title = s[..s.len() - id_str.len()]
                .trim_end_matches('-')
                .to_string();

            if let Ok(id) = VideoId::from_str(&id_str) {
                return Some(Self {
                    id,
                    title,
                    channel: Channel::default(), // Will be set later
                });
            }
        }

        None
    }

    /// Get the full YouTube URL for this video
    pub fn url(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.id)
    }

    /// Get the filename-safe representation for downloads
    pub fn filename(&self) -> String {
        format!("{}-{}", sanitize_filename(&self.title), self.id)
    }

    /// Convert to a string representation suitable for list files
    pub fn to_title_id_string(&self) -> String {
        format!("{}-{}", self.title, self.id)
    }
}

impl fmt::Display for Video {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.title, self.id)
    }
}

/// Strongly-typed YouTube video ID (11 character alphanumeric string)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoId(String);

impl VideoId {
    /// Validate and create a new VideoId
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id: String = id.into();
        if Self::is_valid(&id) {
            Ok(Self(id))
        } else {
            bail!(
                "Invalid YouTube video ID: {}. Must be 11 alphanumeric characters.",
                id
            )
        }
    }

    /// Check if a string is a valid video ID
    fn is_valid(id: &str) -> bool {
        let re = Regex::new(r"^[A-Za-z0-9_-]{11}$").unwrap();
        re.is_match(id)
    }
}

impl FromStr for VideoId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

impl fmt::Display for VideoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for VideoId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Represents a YouTube channel
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Channel {
    pub name: ChannelName,
}

impl Channel {
    pub fn new(name: ChannelName) -> Self {
        Self { name }
    }

    pub fn url(&self) -> String {
        format!("https://www.youtube.com/@{}", self.name)
    }

    /// Get the base directory for this channel
    pub fn base_dir(&self) -> PathBuf {
        PathBuf::from(self.name.to_string())
    }

    /// Get the lists directory for this channel
    pub fn lists_dir(&self) -> PathBuf {
        self.base_dir().join("lists")
    }

    /// Get the videos directory for this channel
    pub fn videos_dir(&self) -> PathBuf {
        self.base_dir().join("videos")
    }

    /// Get the comments directory for this channel
    pub fn comments_dir(&self) -> PathBuf {
        self.base_dir().join("comments")
    }
}

/// Strongly-typed YouTube channel name (alphanumeric, underscores, hyphens only)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelName(String);

impl ChannelName {
    const VALID_PATTERN: &str = r"^[a-zA-Z0-9_-]+$";

    /// Validate and create a new ChannelName
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name: String = name.into();
        if Self::is_valid(&name) {
            Ok(Self(name))
        } else {
            bail!(
                "Invalid channel name '{}': must contain only alphanumeric characters, underscores, or hyphens",
                name
            )
        }
    }

    /// Parse from various input formats (URL, @name, or plain name)
    pub fn parse(input: &str) -> Result<Self> {
        let re = Regex::new(r"/@([^/]+)").unwrap();
        let name = if let Some(caps) = re.captures(input) {
            caps[1].to_string()
        } else {
            input.strip_prefix('@').unwrap_or(input).to_string()
        };

        Self::new(name)
    }

    /// Check if a string is a valid channel name
    fn is_valid(name: &str) -> bool {
        let re = Regex::new(Self::VALID_PATTERN).unwrap();
        re.is_match(name)
    }
}

impl FromStr for ChannelName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

impl fmt::Display for ChannelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ChannelName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Represents a list file containing video URLs
#[derive(Debug, Clone)]
pub struct ListFile {
    pub path: PathBuf,
    pub channel: Channel,
}

impl ListFile {
    /// Create a new ListFile from a path
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            bail!("List file does not exist: {:?}", path);
        }

        let channel = Self::extract_channel_from_path(path)?;

        Ok(Self {
            path: path.to_path_buf(),
            channel,
        })
    }

    /// Extract channel name from list file path
    fn extract_channel_from_path(path: &Path) -> Result<Channel> {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Failed to extract filename from path")?;

        let channel_name = file_name
            .split('-')
            .next()
            .context("Failed to extract channel name from filename")?;

        let name = ChannelName::new(channel_name)?;
        Ok(Channel::new(name))
    }

    /// Read videos from this list file
    pub fn read_videos(&self) -> Result<Vec<Video>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(&self.path)
            .with_context(|| format!("Failed to open list file: {:?}", self.path))?;
        let reader = BufReader::new(file);

        let mut videos = Vec::new();

        let id_regex = Regex::new(r"[A-Za-z0-9_-]{11}$").unwrap();

        for line in reader.lines() {
            let line = line.context("Failed to read line from list file")?;
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            // Try parsing as URL first
            if let Some(video) = Self::parse_video_from_url(line, &self.channel) {
                videos.push(video);
            }
            // Then try parsing as title-id string
            else if let Some(video) = Video::from_title_id_string(line) {
                let video = Video::new(video.id, video.title, self.channel.clone());
                videos.push(video);
            }
            // Try extracting ID from end of line
            if let Some(caps) = id_regex.captures(line)
                && let Ok(id) = VideoId::from_str(&caps[0])
            {
                let title = line[..line.len() - 12].to_string();
                videos.push(Video::new(id, title, self.channel.clone()));
            }
        }

        Ok(videos)
    }

    /// Parse a video from a YouTube URL
    fn parse_video_from_url(url: &str, channel: &Channel) -> Option<Video> {
        // Match watch?v= format
        let watch_regex = Regex::new(r"youtube\.com/watch\?v=([A-Za-z0-9_-]{11})").unwrap();
        if let Some(caps) = watch_regex.captures(url)
            && let Ok(id) = VideoId::from_str(&caps[1])
        {
            return Some(Video::new(id, "", channel.clone())); // Title unknown
        }

        // Match youtu.be/ format
        let short_regex = Regex::new(r"youtu\.be/([A-Za-z0-9_-]{11})").unwrap();
        if let Some(caps) = short_regex.captures(url)
            && let Ok(id) = VideoId::from_str(&caps[1])
        {
            return Some(Video::new(id, "", channel.clone())); // Title unknown
        }

        None
    }
}

/// Sanitize a string for use as a filename
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_id_valid() {
        let id = VideoId::new("dQw4w9WgXcQ").unwrap();
        assert_eq!(id.to_string(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_video_id_invalid() {
        assert!(VideoId::new("too-short").is_err());
        assert!(VideoId::new("way-too-long-for-youtube").is_err());
        assert!(VideoId::new("invalid-chars!!!").is_err());
    }

    #[test]
    fn test_channel_name_valid() {
        let name = ChannelName::new("test-channel_123").unwrap();
        assert_eq!(name.to_string(), "test-channel_123");
    }

    #[test]
    fn test_channel_name_invalid() {
        assert!(ChannelName::new("invalid@chars").is_err());
        assert!(ChannelName::new("no spaces").is_err());
    }

    #[test]
    fn test_channel_name_parse() {
        let from_url = ChannelName::parse("https://youtube.com/@testchannel").unwrap();
        assert_eq!(from_url.to_string(), "testchannel");

        let from_at = ChannelName::parse("@testchannel").unwrap();
        assert_eq!(from_at.to_string(), "testchannel");

        let plain = ChannelName::parse("testchannel").unwrap();
        assert_eq!(plain.to_string(), "testchannel");
    }

    #[test]
    fn test_video_from_title_id() {
        // YouTube video IDs are exactly 11 characters
        let video = Video::from_title_id_string("My Video Title-abc123xyz78").unwrap();
        assert_eq!(video.title, "My Video Title");
        assert_eq!(video.id.to_string(), "abc123xyz78");
    }

    #[test]
    fn test_video_url() {
        let id = VideoId::new("dQw4w9WgXcQ").unwrap();
        let video = Video::new(id, "Test", Channel::default());
        assert_eq!(video.url(), "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }

    #[test]
    fn test_channel_directories() {
        let channel = Channel::new(ChannelName::new("testchannel").unwrap());
        assert_eq!(channel.lists_dir(), PathBuf::from("testchannel/lists"));
        assert_eq!(channel.videos_dir(), PathBuf::from("testchannel/videos"));
        assert_eq!(
            channel.comments_dir(),
            PathBuf::from("testchannel/comments")
        );
    }
}
