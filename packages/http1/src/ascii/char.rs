use std::ops::Deref;

macro_rules! declare_ascii_char {
    ($struct_name:ident => {$($name:ident = $value:literal),*}) => {
        #[repr(u8)]
        #[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
        #[doc="Represents a ASCII character"]
        pub enum $struct_name {
            $($name = $value),*
        }

        impl $struct_name {
            pub fn from_u8(value: u8) -> Option<Self> {
                match value {
                    $($value => Some(Self::$name),)*
                    _ => None
                }
            }

            pub(crate) fn as_u8(&self) -> &u8 {
                match *self as u8 {
                    $($value => &$value,)*
                    _ => unreachable!()
                }
            }
        }
    };
}

declare_ascii_char!(AsciiChar => {
    Null = 0,
    StartOfHeading = 1,
    StartOfText = 2,
    EndOfText = 3,
    EndOfTransmission = 4,
    Enquiry = 5,
    Acknowledgment = 6,
    Bell = 7,
    Backspace = 8,
    HorizontalTab = 9,
    LineFeed = 10,
    VerticalTab = 11,
    FormFeed = 12,
    CarriageReturn = 13,
    ShiftOut = 14,
    ShiftIn = 15,
    DataLinkEscape = 16,
    DeviceControl1 = 17,
    DeviceControl2 = 18,
    DeviceControl3 = 19,
    DeviceControl4 = 20,
    NegativeAcknowledge = 21,
    SynchronousIdle = 22,
    EndOfBlock = 23,
    Cancel = 24,
    EndOfMedium = 25,
    Substitute = 26,
    Escape = 27,
    FileSeparator = 28,
    GroupSeparator = 29,
    RecordSeparator = 30,
    UnitSeparator = 31,
    Space = 32,
    ExclamationMark = 33,
    DoubleQuote = 34,
    Hash = 35,
    Dollar = 36,
    Percent = 37,
    Ampersand = 38,
    SingleQuote = 39,
    LeftParenthesis = 40,
    RightParenthesis = 41,
    Asterisk = 42,
    Plus = 43,
    Comma = 44,
    Hyphen = 45,
    Period = 46,
    Slash = 47,
    Num0 = 48,
    Num1 = 49,
    Num2 = 50,
    Num3 = 51,
    Num4 = 52,
    Num5 = 53,
    Num6 = 54,
    Num7 = 55,
    Num8 = 56,
    Num9 = 57,
    Colon = 58,
    Semicolon = 59,
    LessThan = 60,
    Equals = 61,
    GreaterThan = 62,
    QuestionMark = 63,
    AtSign = 64,
    A = 65,
    B = 66,
    C = 67,
    D = 68,
    E = 69,
    F = 70,
    G = 71,
    H = 72,
    I = 73,
    J = 74,
    K = 75,
    L = 76,
    M = 77,
    N = 78,
    O = 79,
    P = 80,
    Q = 81,
    R = 82,
    S = 83,
    T = 84,
    U = 85,
    V = 86,
    W = 87,
    X = 88,
    Y = 89,
    Z = 90,
    LeftBracket = 91,
    Backslash = 92,
    RightBracket = 93,
    Caret = 94,
    Underscore = 95,
    Backtick = 96,
    LowerA = 97,
    LowerB = 98,
    LowerC = 99,
    LowerD = 100,
    LowerE = 101,
    LowerF = 102,
    LowerG = 103,
    LowerH = 104,
    LowerI = 105,
    LowerJ = 106,
    LowerK = 107,
    LowerL = 108,
    LowerM = 109,
    LowerN = 110,
    LowerO = 111,
    LowerP = 112,
    LowerQ = 113,
    LowerR = 114,
    LowerS = 115,
    LowerT = 116,
    LowerU = 117,
    LowerV = 118,
    LowerW = 119,
    LowerX = 120,
    LowerY = 121,
    LowerZ = 122,
    LeftCurlyBrace = 123,
    VerticalBar = 124,
    RightCurlyBrace = 125,
    Tilde = 126,
    Delete = 127
});

#[allow(clippy::derivable_impls)]
impl Default for AsciiChar {
    fn default() -> Self {
        AsciiChar::Null
    }
}

impl AsciiChar {
    pub fn from_char(ch: char) -> Option<Self> {
        Self::from_u8(ch as u8)
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }

    pub fn to_char(&self) -> char {
        self.to_u8() as char
    }
}

impl Deref for AsciiChar {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        self.as_u8()
    }
}
