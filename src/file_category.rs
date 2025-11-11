/// File categorization system for organizing files by type.
///
/// This module provides a clean, maintainable way to map MIME types or file extensions
/// to broader categories (e.g., "image", "audio", "document").
///
/// # Examples
///
/// ```
/// use dirtidy::file_category::{Category, FileMapper};
///
/// let mapper = FileMapper::default();
/// assert_eq!(mapper.mime_to_category("image/png"), Some(Category::Image));
/// assert_eq!(mapper.mime_to_category("audio/mpeg"), Some(Category::Audio));
/// assert_eq!(mapper.mime_to_category("text/plain"), Some(Category::Document));
/// ```
use std::collections::HashMap;

/// Represents a broad file category.
///
/// Categories are used to organize files into meaningful groups
/// for directory-based organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Image files (PNG, JPG, GIF, etc.)
    Image,
    /// Audio files (MP3, WAV, FLAC, etc.)
    Audio,
    /// Video files (MP4, MKV, AVI, etc.)
    Video,
    /// Document files (PDF, DOCX, TXT, etc.)
    Document,
    /// Archive files (ZIP, RAR, 7Z, etc.)
    Archive,
    /// Code/Source files (Rust, Python, JavaScript, etc.)
    Code,
    /// Spreadsheet files (XLSX, CSV, ODS, etc.)
    Spreadsheet,
    /// Presentation files (PPTX, ODP, etc.)
    Presentation,
    /// Font files (TTF, OTF, WOFF, etc.)
    Font,
    /// Unknown or uncategorized files
    Other,
}

impl Category {
    /// Returns the directory name for this category.
    ///
    /// # Examples
    ///
    /// ```
    /// use dirtidy::file_category::Category;
    ///
    /// assert_eq!(Category::Image.dir_name(), "images");
    /// assert_eq!(Category::Audio.dir_name(), "audio");
    /// assert_eq!(Category::Other.dir_name(), "other");
    /// ```
    pub fn dir_name(&self) -> &'static str {
        match self {
            Category::Image => "images",
            Category::Audio => "audio",
            Category::Video => "videos",
            Category::Document => "documents",
            Category::Archive => "archives",
            Category::Code => "code",
            Category::Spreadsheet => "spreadsheets",
            Category::Presentation => "presentations",
            Category::Font => "fonts",
            Category::Other => "other",
        }
    }

    /// Returns a human-readable description of this category.
    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            Category::Image => "Image files",
            Category::Audio => "Audio files",
            Category::Video => "Video files",
            Category::Document => "Document files",
            Category::Archive => "Archive files",
            Category::Code => "Source code files",
            Category::Spreadsheet => "Spreadsheet files",
            Category::Presentation => "Presentation files",
            Category::Font => "Font files",
            Category::Other => "Other files",
        }
    }
}

/// Maps MIME types and file extensions to categories.
///
/// This struct encapsulates the logic for categorizing files.
/// It uses a HashMap for efficient lookups and can be extended
/// to support custom mappings.
#[derive(Debug, Clone)]
pub struct FileMapper {
    mime_map: HashMap<String, Category>,
    extension_map: HashMap<String, Category>,
}

impl FileMapper {
    /// Creates a new `FileMapper` with all standard mappings.
    pub fn new() -> Self {
        let mut mapper = Self {
            mime_map: HashMap::new(),
            extension_map: HashMap::new(),
        };
        mapper.populate_standard_mappings();
        mapper
    }

    /// Populates the mapper with standard MIME type and extension mappings.
    fn populate_standard_mappings(&mut self) {
        // Image MIME types
        self.add_mime_mapping("image/png", Category::Image);
        self.add_mime_mapping("image/jpeg", Category::Image);
        self.add_mime_mapping("image/jpg", Category::Image);
        self.add_mime_mapping("image/gif", Category::Image);
        self.add_mime_mapping("image/webp", Category::Image);
        self.add_mime_mapping("image/svg+xml", Category::Image);
        self.add_mime_mapping("image/bmp", Category::Image);
        self.add_mime_mapping("image/tiff", Category::Image);
        self.add_mime_mapping("image/heic", Category::Image);
        self.add_mime_mapping("image/heif", Category::Image);

        // Audio MIME types
        self.add_mime_mapping("audio/mpeg", Category::Audio);
        self.add_mime_mapping("audio/wav", Category::Audio);
        self.add_mime_mapping("audio/ogg", Category::Audio);
        self.add_mime_mapping("audio/flac", Category::Audio);
        self.add_mime_mapping("audio/aac", Category::Audio);
        self.add_mime_mapping("audio/x-m4a", Category::Audio);
        self.add_mime_mapping("audio/webm", Category::Audio);

        // Video MIME types
        self.add_mime_mapping("video/mp4", Category::Video);
        self.add_mime_mapping("video/mpeg", Category::Video);
        self.add_mime_mapping("video/quicktime", Category::Video);
        self.add_mime_mapping("video/x-msvideo", Category::Video);
        self.add_mime_mapping("video/x-matroska", Category::Video);
        self.add_mime_mapping("video/webm", Category::Video);
        self.add_mime_mapping("video/x-flv", Category::Video);
        self.add_mime_mapping("video/3gpp", Category::Video);

        // Document MIME types
        self.add_mime_mapping("application/pdf", Category::Document);
        self.add_mime_mapping("text/plain", Category::Document);
        self.add_mime_mapping("text/html", Category::Document);
        self.add_mime_mapping("text/markdown", Category::Document);
        self.add_mime_mapping("application/msword", Category::Document);
        self.add_mime_mapping(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Category::Document,
        );
        self.add_mime_mapping("application/rtf", Category::Document);
        self.add_mime_mapping(
            "application/vnd.oasis.opendocument.text",
            Category::Document,
        );

        // Archive MIME types
        self.add_mime_mapping("application/zip", Category::Archive);
        self.add_mime_mapping("application/x-rar-compressed", Category::Archive);
        self.add_mime_mapping("application/x-7z-compressed", Category::Archive);
        self.add_mime_mapping("application/x-tar", Category::Archive);
        self.add_mime_mapping("application/gzip", Category::Archive);
        self.add_mime_mapping("application/x-bzip2", Category::Archive);

        // Code MIME types
        self.add_mime_mapping("text/x-python", Category::Code);
        self.add_mime_mapping("text/x-java", Category::Code);
        self.add_mime_mapping("text/x-c", Category::Code);
        self.add_mime_mapping("text/x-c++src", Category::Code);
        self.add_mime_mapping("text/x-javascript", Category::Code);
        self.add_mime_mapping("application/javascript", Category::Code);
        self.add_mime_mapping("text/x-shellscript", Category::Code);
        self.add_mime_mapping("text/x-rust", Category::Code);
        self.add_mime_mapping("application/json", Category::Code);
        self.add_mime_mapping("application/xml", Category::Code);
        self.add_mime_mapping("text/xml", Category::Code);
        self.add_mime_mapping("text/x-yaml", Category::Code);
        self.add_mime_mapping("text/x-toml", Category::Code);

        // Spreadsheet MIME types
        self.add_mime_mapping("text/csv", Category::Spreadsheet);
        self.add_mime_mapping("application/vnd.ms-excel", Category::Spreadsheet);
        self.add_mime_mapping(
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Category::Spreadsheet,
        );
        self.add_mime_mapping(
            "application/vnd.oasis.opendocument.spreadsheet",
            Category::Spreadsheet,
        );

        // Presentation MIME types
        self.add_mime_mapping("application/vnd.ms-powerpoint", Category::Presentation);
        self.add_mime_mapping(
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            Category::Presentation,
        );
        self.add_mime_mapping(
            "application/vnd.oasis.opendocument.presentation",
            Category::Presentation,
        );

        // Font MIME types
        self.add_mime_mapping("font/ttf", Category::Font);
        self.add_mime_mapping("font/otf", Category::Font);
        self.add_mime_mapping("font/woff", Category::Font);
        self.add_mime_mapping("font/woff2", Category::Font);
        self.add_mime_mapping("application/x-font-ttf", Category::Font);
        self.add_mime_mapping("application/x-font-otf", Category::Font);

        // File extension mappings (case-insensitive)
        // Image extensions
        self.add_extension_mapping("png", Category::Image);
        self.add_extension_mapping("jpg", Category::Image);
        self.add_extension_mapping("jpeg", Category::Image);
        self.add_extension_mapping("gif", Category::Image);
        self.add_extension_mapping("webp", Category::Image);
        self.add_extension_mapping("svg", Category::Image);
        self.add_extension_mapping("bmp", Category::Image);
        self.add_extension_mapping("tiff", Category::Image);
        self.add_extension_mapping("ico", Category::Image);
        self.add_extension_mapping("heic", Category::Image);

        // Audio extensions
        self.add_extension_mapping("mp3", Category::Audio);
        self.add_extension_mapping("wav", Category::Audio);
        self.add_extension_mapping("ogg", Category::Audio);
        self.add_extension_mapping("flac", Category::Audio);
        self.add_extension_mapping("aac", Category::Audio);
        self.add_extension_mapping("m4a", Category::Audio);
        self.add_extension_mapping("wma", Category::Audio);

        // Video extensions
        self.add_extension_mapping("mp4", Category::Video);
        self.add_extension_mapping("mkv", Category::Video);
        self.add_extension_mapping("avi", Category::Video);
        self.add_extension_mapping("mov", Category::Video);
        self.add_extension_mapping("flv", Category::Video);
        self.add_extension_mapping("wmv", Category::Video);
        self.add_extension_mapping("webm", Category::Video);
        self.add_extension_mapping("3gp", Category::Video);

        // Document extensions
        self.add_extension_mapping("pdf", Category::Document);
        self.add_extension_mapping("txt", Category::Document);
        self.add_extension_mapping("doc", Category::Document);
        self.add_extension_mapping("docx", Category::Document);
        self.add_extension_mapping("html", Category::Document);
        self.add_extension_mapping("htm", Category::Document);
        self.add_extension_mapping("md", Category::Document);
        self.add_extension_mapping("rtf", Category::Document);
        self.add_extension_mapping("odt", Category::Document);

        // Archive extensions
        self.add_extension_mapping("zip", Category::Archive);
        self.add_extension_mapping("rar", Category::Archive);
        self.add_extension_mapping("7z", Category::Archive);
        self.add_extension_mapping("tar", Category::Archive);
        self.add_extension_mapping("gz", Category::Archive);
        self.add_extension_mapping("bz2", Category::Archive);
        self.add_extension_mapping("xz", Category::Archive);

        // Code extensions
        self.add_extension_mapping("py", Category::Code);
        self.add_extension_mapping("java", Category::Code);
        self.add_extension_mapping("c", Category::Code);
        self.add_extension_mapping("cpp", Category::Code);
        self.add_extension_mapping("h", Category::Code);
        self.add_extension_mapping("hpp", Category::Code);
        self.add_extension_mapping("js", Category::Code);
        self.add_extension_mapping("ts", Category::Code);
        self.add_extension_mapping("rs", Category::Code);
        self.add_extension_mapping("go", Category::Code);
        self.add_extension_mapping("sh", Category::Code);
        self.add_extension_mapping("bash", Category::Code);
        self.add_extension_mapping("json", Category::Code);
        self.add_extension_mapping("xml", Category::Code);
        self.add_extension_mapping("yaml", Category::Code);
        self.add_extension_mapping("yml", Category::Code);
        self.add_extension_mapping("toml", Category::Code);

        // Spreadsheet extensions
        self.add_extension_mapping("csv", Category::Spreadsheet);
        self.add_extension_mapping("xls", Category::Spreadsheet);
        self.add_extension_mapping("xlsx", Category::Spreadsheet);
        self.add_extension_mapping("ods", Category::Spreadsheet);

        // Presentation extensions
        self.add_extension_mapping("ppt", Category::Presentation);
        self.add_extension_mapping("pptx", Category::Presentation);
        self.add_extension_mapping("odp", Category::Presentation);

        // Font extensions
        self.add_extension_mapping("ttf", Category::Font);
        self.add_extension_mapping("otf", Category::Font);
        self.add_extension_mapping("woff", Category::Font);
        self.add_extension_mapping("woff2", Category::Font);
    }

    /// Adds a MIME type to category mapping.
    pub fn add_mime_mapping(&mut self, mime: &str, category: Category) {
        self.mime_map.insert(mime.to_lowercase(), category);
    }

    /// Adds a file extension to category mapping.
    pub fn add_extension_mapping(&mut self, ext: &str, category: Category) {
        self.extension_map.insert(ext.to_lowercase(), category);
    }

    /// Maps a MIME type to a category.
    ///
    /// # Examples
    ///
    /// ```
    /// use dirtidy::file_category::{Category, FileMapper};
    ///
    /// let mapper = FileMapper::default();
    /// assert_eq!(mapper.mime_to_category("image/png"), Some(Category::Image));
    /// assert_eq!(mapper.mime_to_category("unknown/type"), None);
    /// ```
    pub fn mime_to_category(&self, mime_type: &str) -> Option<Category> {
        self.mime_map.get(&mime_type.to_lowercase()).copied()
    }

    /// Maps a file extension to a category.
    ///
    /// # Examples
    ///
    /// ```
    /// use dirtidy::file_category::{Category, FileMapper};
    ///
    /// let mapper = FileMapper::default();
    /// assert_eq!(mapper.extension_to_category("pdf"), Some(Category::Document));
    /// assert_eq!(mapper.extension_to_category("PNG"), Some(Category::Image));
    /// ```
    pub fn extension_to_category(&self, ext: &str) -> Option<Category> {
        self.extension_map.get(&ext.to_lowercase()).copied()
    }

    /// Determines the category for a file given its MIME type and/or extension.
    ///
    /// The function uses the following strategy:
    /// 1. Try to match by MIME type first (more reliable)
    /// 2. Fall back to file extension if MIME type is not recognized
    /// 3. Return `Category::Other` if neither matches
    ///
    /// # Examples
    ///
    /// ```
    /// use dirtidy::file_category::{Category, FileMapper};
    ///
    /// let mapper = FileMapper::default();
    /// assert_eq!(
    ///     mapper.categorize(Some("image/png"), Some("png")),
    ///     Category::Image
    /// );
    /// assert_eq!(
    ///     mapper.categorize(None, Some("pdf")),
    ///     Category::Document
    /// );
    /// assert_eq!(
    ///     mapper.categorize(None, None),
    ///     Category::Other
    /// );
    /// ```
    pub fn categorize(&self, mime_type: Option<&str>, ext: Option<&str>) -> Category {
        // Try MIME type first
        if let Some(mime) = mime_type
            && let Some(category) = self.mime_to_category(mime)
        {
            return category;
        }

        // Fall back to extension
        if let Some(extension) = ext
            && let Some(category) = self.extension_to_category(extension)
        {
            return category;
        }

        // Default to "Other"
        Category::Other
    }
}

impl Default for FileMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_dir_names() {
        assert_eq!(Category::Image.dir_name(), "images");
        assert_eq!(Category::Audio.dir_name(), "audio");
        assert_eq!(Category::Video.dir_name(), "videos");
        assert_eq!(Category::Document.dir_name(), "documents");
        assert_eq!(Category::Archive.dir_name(), "archives");
        assert_eq!(Category::Code.dir_name(), "code");
        assert_eq!(Category::Spreadsheet.dir_name(), "spreadsheets");
        assert_eq!(Category::Presentation.dir_name(), "presentations");
        assert_eq!(Category::Font.dir_name(), "fonts");
        assert_eq!(Category::Other.dir_name(), "other");
    }

    #[test]
    fn test_mime_to_category_images() {
        let mapper = FileMapper::default();
        assert_eq!(mapper.mime_to_category("image/png"), Some(Category::Image));
        assert_eq!(mapper.mime_to_category("image/jpeg"), Some(Category::Image));
        assert_eq!(mapper.mime_to_category("image/gif"), Some(Category::Image));
    }

    #[test]
    fn test_mime_to_category_audio() {
        let mapper = FileMapper::default();
        assert_eq!(mapper.mime_to_category("audio/mpeg"), Some(Category::Audio));
        assert_eq!(mapper.mime_to_category("audio/wav"), Some(Category::Audio));
    }

    #[test]
    fn test_mime_to_category_unknown() {
        let mapper = FileMapper::default();
        assert_eq!(mapper.mime_to_category("unknown/type"), None);
    }

    #[test]
    fn test_mime_to_category_case_insensitive() {
        let mapper = FileMapper::default();
        assert_eq!(mapper.mime_to_category("IMAGE/PNG"), Some(Category::Image));
        assert_eq!(mapper.mime_to_category("Image/Png"), Some(Category::Image));
    }

    #[test]
    fn test_extension_to_category() {
        let mapper = FileMapper::default();
        assert_eq!(
            mapper.extension_to_category("pdf"),
            Some(Category::Document)
        );
        assert_eq!(mapper.extension_to_category("mp3"), Some(Category::Audio));
        assert_eq!(mapper.extension_to_category("rs"), Some(Category::Code));
    }

    #[test]
    fn test_extension_to_category_case_insensitive() {
        let mapper = FileMapper::default();
        assert_eq!(
            mapper.extension_to_category("PDF"),
            Some(Category::Document)
        );
        assert_eq!(mapper.extension_to_category("Mp3"), Some(Category::Audio));
    }

    #[test]
    fn test_categorize_with_mime() {
        let mapper = FileMapper::default();
        assert_eq!(
            mapper.categorize(Some("image/png"), Some("txt")),
            Category::Image
        );
    }

    #[test]
    fn test_categorize_fallback_to_extension() {
        let mapper = FileMapper::default();
        assert_eq!(mapper.categorize(None, Some("pdf")), Category::Document);
    }

    #[test]
    fn test_categorize_defaults_to_other() {
        let mapper = FileMapper::default();
        assert_eq!(mapper.categorize(None, None), Category::Other);
        assert_eq!(
            mapper.categorize(Some("unknown/type"), Some("xyz")),
            Category::Other
        );
    }

    #[test]
    fn test_custom_mapping() {
        let mut mapper = FileMapper::default();
        mapper.add_mime_mapping("application/custom", Category::Code);
        mapper.add_extension_mapping("custom", Category::Code);

        assert_eq!(
            mapper.mime_to_category("application/custom"),
            Some(Category::Code)
        );
        assert_eq!(mapper.extension_to_category("custom"), Some(Category::Code));
    }
}
