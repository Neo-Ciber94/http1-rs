use std::{borrow::Cow, fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mime {
    type_: Cow<'static, str>,
    subtype: Cow<'static, str>,
    parameter: Option<Cow<'static, str>>,
}

impl Mime {
    const fn const_static(
        type_: &'static str,
        subtype: &'static str,
        parameter: Option<&'static str>,
    ) -> Self {
        match parameter {
            Some(parameter) => Mime {
                type_: Cow::Borrowed(type_),
                subtype: Cow::Borrowed(subtype),
                parameter: Some(Cow::Borrowed(parameter)),
            },
            None => Mime {
                type_: Cow::Borrowed(type_),
                subtype: Cow::Borrowed(subtype),
                parameter: None,
            },
        }
    }

    fn new_(
        type_: Cow<'static, str>,
        subtype: Cow<'static, str>,
        parameter: Option<Cow<'static, str>>,
    ) -> Result<Self, InvalidStr> {
        validate_str("type", type_.as_ref())?;
        validate_str("subtype", subtype.as_ref())?;

        if let Some(p) = &parameter {
            validate_str("parameter", p.as_ref())?;
        }

        Ok(Mime {
            type_,
            subtype,
            parameter,
        })
    }

    pub fn new(
        type_: impl Into<Cow<'static, str>>,
        subtype: impl Into<Cow<'static, str>>,
        parameter: Option<&str>,
    ) -> Self {
        let parameter: Option<Cow<'static, str>> = parameter.map(|x| x.to_owned().into());
        let type_: Cow<'static, str> = type_.into();
        let subtype: Cow<'static, str> = subtype.into();

        match Mime::get_any_mime(type_.as_ref(), subtype.as_ref(), parameter.as_deref()) {
            Some(mime) => return mime,
            _ => {}
        }

        match Mime::get_mime(type_.as_ref(), subtype.as_ref(), parameter.as_deref()) {
            Some(mime) => mime,
            None => Self::new_(type_, subtype, parameter).expect("invalid mime values"),
        }
    }

    pub fn from_extension(file_extension: &str) -> Result<Mime, &str> {
        Mime::get_mime_from_extension(file_extension).ok_or(file_extension)
    }

    pub fn guess_mime(filename: &str) -> Result<Mime, &str> {
        if let Some(extension) = filename.split('.').last() {
            Mime::from_extension(extension)
        } else {
            Err(filename)
        }
    }

    #[inline]
    pub fn ty(&self) -> &str {
        &self.type_.as_ref()
    }

    #[inline]
    pub fn subtype(&self) -> &str {
        &self.subtype.as_ref()
    }

    pub fn matches(&self, other: &Mime) -> bool {
        match (self.ty(), other.ty()) {
            ("*", "*") | ("*", _) | (_, "*") => {}
            (t1, t2) if t1 != t2 => return false,
            _ => {}
        }

        match (self.subtype(), other.subtype()) {
            ("*", "*") | ("*", _) | (_, "*") => {}
            (s1, s2) if s1 != s2 => return false,
            _ => {}
        }

        match ((&self.parameter, &other.parameter)) {
            (None, None) => true,
            (Some(p1), Some(p2)) => p1 == p2,
            _ => false,
        }
    }
}

impl Mime {
    /// Any content-type: `*/*`
    pub const ANY: Mime = Mime::const_static("*", "*", None);

    /// Any content-type: `image/*`
    pub const ANY_IMAGE: Mime = Mime::const_static("image", "*", None);

    /// Any content-type: `audio/*`
    pub const ANY_AUDIO: Mime = Mime::const_static("audio", "*", None);

    /// Any content-type: `video/*`
    pub const ANY_VIDEO: Mime = Mime::const_static("video", "*", None);

    /// Any content-type: `text/*`
    pub const ANY_TEXT: Mime = Mime::const_static("text", "*", None);

    fn get_any_mime(type_: &str, subtype: &str, parameter: Option<&str>) -> Option<Mime> {
        match (type_, subtype, parameter) {
            ("*", "*", None) => Some(Mime::ANY),
            ("image", "*", None) => Some(Mime::ANY_IMAGE),
            ("audio", "*", None) => Some(Mime::ANY_AUDIO),
            ("text", "*", None) => Some(Mime::ANY_TEXT),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum InvalidMimeType {
    InvalidStr,
    EmptyOrWhitespace,
}

impl FromStr for Mime {
    type Err = InvalidMimeType;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let parameter = match s.find(";") {
            Some(parameter_idx) => {
                let rest = &s[(parameter_idx + 1)..];
                if rest.is_empty() {
                    return Err(InvalidMimeType::InvalidStr);
                }

                s = &s[..parameter_idx];

                Some(rest)
            }
            None => None,
        };

        let (type_, subtype) = s.split_once("/").ok_or(InvalidMimeType::InvalidStr)?;

        match Mime::get_mime(type_.as_ref(), subtype.as_ref(), parameter.as_deref()) {
            Some(mime) => Ok(mime),
            None => {
                let parameter = parameter.map(|x| Cow::Owned(x.to_owned()));

                Mime::new_(
                    type_.to_owned().into(),
                    subtype.to_owned().into(),
                    parameter,
                )
                .map_err(|_| InvalidMimeType::EmptyOrWhitespace)
            }
        }
    }
}

impl Display for Mime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(param) = self.parameter.as_ref() {
            write!(f, "{}/{};{param}", self.type_, self.subtype)
        } else {
            write!(f, "{}/{}", self.type_, self.subtype)
        }
    }
}

macro_rules! declare_mime_types {
    ($($NAME:ident => [$EXT:literal, $TYPE:literal, $SUBTYPE:literal, $($PARAM:tt)+]),*) => {
        impl Mime {
            $(
                pub const $NAME: Mime = Mime::const_static($TYPE, $SUBTYPE, $($PARAM)*);
            )*

            fn get_mime(type_: &str, subtype: &str, parameter: Option<&str>) -> Option<Mime> {
                match ((type_, subtype, parameter)) {
                    $(
                        ($TYPE, $SUBTYPE, $($PARAM)*) => Some(Mime::$NAME),
                    )*
                    _ => None
                }
            }

            #[allow(unreachable_patterns)]
            fn get_mime_from_extension(extension: &str) -> Option<Mime> {
                if extension.trim().is_empty() {
                    return None;
                }

                match extension {
                    $(
                        $EXT => Some(Mime::$NAME),
                    )*
                    _ => None
                }
            }
        }
    };
}

declare_mime_types! {
    // Application types
    APPLICATION_OCTET_STREAM => ["bin", "application", "octet-stream", None],
    APPLICATION_JSON => ["json", "application", "json", None],
    APPLICATION_JSON_UTF8 => ["", "application", "json", Some("charset=UTF-8")],
    APPLICATION_JAVASCRIPT => ["js", "application", "javascript", None],
    APPLICATION_XML => ["", "application", "xml", None],
    APPLICATION_PDF => ["pdf", "application", "pdf", None],
    APPLICATION_ZIP => ["zip", "application", "zip", None],
    APPLICATION_RAR => ["rar", "application", "vnd.rar", None],
    APPLICATION_7Z => ["7z", "application", "x-7z-compressed", None],
    APPLICATION_GZIP => ["gz", "application", "gzip", None],
    APPLICATION_RTF => ["rtf", "application", "rtf", None],
    APPLICATION_SQL => ["sql", "application", "sql", None],
    APPLICATION_WASM => ["wasm", "application", "wasm", None],
    APPLICATION_XHTML => ["xhtml", "application", "xhtml+xml", None],
    APPLICATION_TAR => ["tar", "application", "x-tar", None],
    APPLICATION_MSWORD => ["doc", "application", "msword", None],
    APPLICATION_MSWORD_OPENXML => ["docx", "application", "vnd.openxmlformats-officedocument.wordprocessingml.document", None],
    APPLICATION_POWERPOINT => ["ppt", "application", "vnd.ms-powerpoint", None],
    APPLICATION_POWERPOINT_OPENXML => ["pptx", "application", "vnd.openxmlformats-officedocument.presentationml.presentation", None],
    APPLICATION_EXCEL => ["xls", "application", "vnd.ms-excel", None],
    APPLICATION_EXCEL_OPENXML => ["xlsx", "application", "vnd.openxmlformats-officedocument.spreadsheetml.sheet", None],
    APPLICATION_EPUB => ["epub", "application", "epub+zip", None],
    APPLICATION_OGG => ["ogx", "application", "ogg", None],

    // Text types
    TEXT_PLAIN => ["txt", "text", "plain", None],
    TEXT_HTML => ["html", "text", "html", None],
    TEXT_CSS => ["css", "text", "css", None],
    TEXT_CSV => ["csv", "text", "csv", None],
    TEXT_XML => ["xml", "text", "xml", None],
    TEXT_MARKDOWN => ["md", "text", "markdown", None],
    TEXT_YAML => ["yaml", "text", "yaml", None],
    TEXT_VCARD => ["vcf", "text", "vcard", None],

    // Image types
    IMAGE_JPEG => ["jpg", "image", "jpeg", None],
    IMAGE_PNG => ["png", "image", "png", None],
    IMAGE_GIF => ["gif", "image", "gif", None],
    IMAGE_SVG => ["svg", "image", "svg+xml", None],
    IMAGE_TIFF => ["tiff", "image", "tiff", None],
    IMAGE_WEBP => ["webp", "image", "webp", None],
    IMAGE_BMP => ["bmp", "image", "bmp", None],
    IMAGE_ICO => ["ico", "image", "vnd.microsoft.icon", None],
    IMAGE_HEIF => ["heif", "image", "heif", None],
    IMAGE_HEIC => ["heic", "image", "heic", None],

    // Audio types
    AUDIO_MP3 => ["mp3", "audio", "mpeg", None],
    AUDIO_WAV => ["wav", "audio", "wav", None],
    AUDIO_OGG => ["ogg", "audio", "ogg", None],
    AUDIO_FLAC => ["flac", "audio", "flac", None],
    AUDIO_MIDI => ["midi", "audio", "midi", None],
    AUDIO_WEBM => ["weba", "audio", "webm", None],
    AUDIO_AAC => ["aac", "audio", "aac", None],
    AUDIO_M4A => ["m4a", "audio", "mp4", None],

    // Video types
    VIDEO_MP4 => ["mp4", "video", "mp4", None],
    VIDEO_WEBM => ["webm", "video", "webm", None],
    VIDEO_OGG => ["ogv", "video", "ogg", None],
    VIDEO_MPEG => ["mpeg", "video", "mpeg", None],
    VIDEO_3GP => ["3gp", "video", "3gpp", None],
    VIDEO_FLV => ["flv", "video", "x-flv", None],
    VIDEO_AVI => ["avi", "video", "x-msvideo", None],
    VIDEO_MOV => ["mov", "video", "quicktime", None],
    VIDEO_MKV => ["mkv", "video", "x-matroska", None],

    // Font types
    FONT_TTF => ["ttf", "font", "ttf", None],
    FONT_OTF => ["otf", "font", "otf", None],
    FONT_WOFF => ["woff", "font", "woff", None],
    FONT_WOFF2 => ["woff2", "font", "woff2", None]
}

#[allow(dead_code)]
#[derive(Debug)]
enum InvalidStr {
    Empty(&'static str),
    ContainsWhitespace(&'static str),
}

fn validate_str(name: &'static str, s: &str) -> Result<(), InvalidStr> {
    if s.is_empty() {
        return Err(InvalidStr::Empty(name));
    }

    if s.chars().any(|c| c.is_whitespace()) {
        return Err(InvalidStr::ContainsWhitespace(name));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_mime_type() {
        let mime = Mime::new("text", "plain", None);
        assert_eq!(mime.ty(), "text");
        assert_eq!(mime.subtype(), "plain");
        assert_eq!(mime.parameter, None);
    }

    #[test]
    fn should_create_mime_type_with_parameter() {
        let mime = Mime::new("application", "xml", Some("charset=UTF-8"));
        assert_eq!(mime.ty(), "application");
        assert_eq!(mime.subtype(), "xml");
        assert_eq!(mime.parameter.as_deref(), Some("charset=UTF-8"));
    }

    #[test]
    fn should_return_correct_mime_from_extension() {
        let mime = Mime::from_extension("pdf").unwrap();
        assert_eq!(mime.ty(), "application");
        assert_eq!(mime.subtype(), "pdf");
    }

    #[test]
    fn should_return_error_for_unknown_extension() {
        let result = Mime::from_extension("xyz");
        assert!(result.is_err());
    }

    #[test]
    fn should_guess_mime_type_from_filename() {
        let mime = Mime::guess_mime("document.docx").unwrap();
        assert_eq!(mime.ty(), "application");
        assert_eq!(
            mime.subtype(),
            "vnd.openxmlformats-officedocument.wordprocessingml.document"
        );
    }

    #[test]
    fn should_return_error_for_filename_without_extension() {
        let result = Mime::guess_mime("file_without_extension");
        assert!(result.is_err());
    }

    #[test]
    fn should_match_const_static_mime_types() {
        let mime = &Mime::IMAGE_JPEG;
        assert_eq!(mime.ty(), "image");
        assert_eq!(mime.subtype(), "jpeg");
        assert_eq!(mime.parameter, None);
    }

    #[test]
    fn should_parse_valid_mime_from_string() {
        let mime: Mime = "audio/mpeg".parse().unwrap();
        assert_eq!(mime.ty(), "audio");
        assert_eq!(mime.subtype(), "mpeg");
    }

    #[test]
    fn should_return_error_for_invalid_mime_string() {
        let result: Result<Mime, _> = "invalid_mime_type".parse();
        assert!(result.is_err());
    }

    #[test]
    fn should_display_mime_as_string_without_parameter() {
        let mime = Mime::new("video", "mp4", None);
        assert_eq!(format!("{}", mime), "video/mp4");
    }

    #[test]
    fn should_display_mime_as_string_with_parameter() {
        let mime = Mime::new("application", "xml", Some("charset=UTF-8"));
        assert_eq!(format!("{}", mime), "application/xml;charset=UTF-8");
    }

    #[test]
    fn should_return_mime_type_for_known_file_extension() {
        let mime = Mime::from_extension("css").unwrap();
        assert_eq!(mime.ty(), "text");
        assert_eq!(mime.subtype(), "css");
    }

    #[test]
    fn should_handle_7z_extension() {
        let mime = Mime::from_extension("7z").unwrap();
        assert_eq!(mime.ty(), "application");
        assert_eq!(mime.subtype(), "x-7z-compressed");
    }

    #[test]
    fn should_correctly_match_mime_types() {
        // Test identical MIME types
        let mime1 = Mime::new("text", "plain", None);
        let mime2 = Mime::new("text", "plain", None);
        assert!(mime1.matches(&mime2));

        let mime3 = Mime::new("image", "png", None);
        let mime4 = Mime::new("image", "png", None);
        assert!(mime3.matches(&mime4));

        // Test different MIME types
        let mime5 = Mime::new("text", "plain", None);
        let mime6 = Mime::new("text", "html", None);
        assert!(!mime5.matches(&mime6));

        let mime7 = Mime::new("image", "png", None);
        let mime8 = Mime::new("image", "jpeg", None);
        assert!(!mime7.matches(&mime8));

        // Test matching with wildcard types
        let mime9 = Mime::new("*", "json", None);
        let mime10 = Mime::new("application", "json", None);
        assert!(mime9.matches(&mime10));

        let mime11 = Mime::new("*", "json", None);
        let mime12 = Mime::new("image", "png", None);
        assert!(!mime11.matches(&mime12));

        // Test matching with wildcard subtypes
        let mime13 = Mime::new("text", "*", None);
        let mime14 = Mime::new("text", "plain", None);
        assert!(mime13.matches(&mime14));

        let mime15 = Mime::new("text", "*", None);
        let mime16 = Mime::new("application", "json", None);
        assert!(!mime15.matches(&mime16));

        // Test matching with both wildcards
        let mime17 = Mime::new("*", "*", None);
        let mime18 = Mime::new("text", "plain", None);
        assert!(mime17.matches(&mime18));

        let mime19 = Mime::new("*", "*", None);
        let mime20 = Mime::new("image", "png", None);
        assert!(mime19.matches(&mime20));

        let mime21 = Mime::new("*", "*", None);
        let mime22 = Mime::new("application", "json", None);
        assert!(mime21.matches(&mime22));
    }
}
