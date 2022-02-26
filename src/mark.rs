//! Defines the marks used in OpMark.

/// How the element aligns. Currently work for `Image` only.
#[derive(Debug)]
pub enum AlignHorizontal {
    Auto,
    Left,
    Right,
    Center,
}

impl Default for AlignHorizontal {
    fn default() -> Self {
        AlignHorizontal::Auto
    }
}

/// The heading level of the text element.
#[derive(Clone, Debug)]
pub enum Heading {
    None,
    H1,
    H2,
    H3,
    H4,
    H5,
}

impl Default for Heading {
    fn default() -> Self {
        Heading::None
    }
}

impl From<u8> for Heading {
    #[inline]
    fn from(n: u8) -> Self {
        match n {
            0 => Heading::None,
            1 => Heading::H1,
            2 => Heading::H2,
            3 => Heading::H3,
            4 => Heading::H4,
            _ => Heading::H5,
        }
    }
}

impl Heading {
    #[inline]
    pub fn to_int(&self) -> u8 {
        match *self {
            Heading::None => 0,
            Heading::H1 => 1,
            Heading::H2 => 2,
            Heading::H3 => 3,
            Heading::H4 => 4,
            Heading::H5 => 5,
        }
    }
}

/// The intent level of the text element.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum IndentLevel {
    None,
    I1,
    I2,
    I3,
    I4,
    I5,
}

impl Default for IndentLevel {
    fn default() -> Self {
        IndentLevel::None
    }
}

impl From<u8> for IndentLevel {
    #[inline]
    fn from(n: u8) -> Self {
        match n {
            0 => IndentLevel::None,
            1 => IndentLevel::I1,
            2 => IndentLevel::I2,
            3 => IndentLevel::I3,
            4 => IndentLevel::I4,
            _ => IndentLevel::I5,
        }
    }
}

impl IndentLevel {
    #[inline]
    pub fn to_int(&self) -> u8 {
        match *self {
            IndentLevel::None => 0,
            IndentLevel::I1 => 1,
            IndentLevel::I2 => 2,
            IndentLevel::I3 => 3,
            IndentLevel::I4 => 4,
            IndentLevel::I5 => 5,
        }
    }
}

/// Whether the text element is within a list.
#[derive(Clone, Debug)]
pub enum Listing {
    /// Text is not in a list.
    None,
    /// Text is in an ordered list.
    Ordered(u8, IndentLevel),
    /// Text is in an unordered list.
    Unordered(IndentLevel),
}

impl Default for Listing {
    fn default() -> Self {
        Listing::None
    }
}

/// The marks used in OpMark.
#[derive(Debug)]
pub enum Mark {
    /// A code block element:
    /// ````text
    /// ```language
    /// code
    /// ```
    CodeBlock(String, Option<String>),
    /// An image element:
    /// ```
    /// ![title](src)<options>
    /// ```
    /// You can specify the size and the alignment of the image in options:
    /// ```
    /// // Image with width of 50.
    /// ![test](test.png)<w50>
    /// // Image with height of 50.
    /// ![test](test.png)<h50>
    /// // Image with center alignment. Available values: auto, left, right, center.
    /// ![test](test.png)<center>
    /// ```
    /// You can combine options together, and each option is separated by `|`.
    ///
    /// `options` is optional.
    Image(String, String, StyleImage),
    /// A new line element.
    NewLine,
    /// A transition element:
    /// ```
    /// ---t
    /// ```
    /// A transition is a group of elements which show up together after interaction (usually mouse click or keyboard input).
    ///
    /// A transition starts at a transition mark (`---t`), and ends at either the next transition mark, next page mark, or a transition end mark.
    ///
    /// Number can be appended to a transition mark, indicates that the order of the appearence of this transition group (noted that the index starts from 0), otherwise the transitions show up from top to bottom:
    /// ```
    /// ---t
    /// This line will show up after the first interaction.
    /// ---t3
    /// This line will show up after the fourth interaction.
    /// ---t1
    /// This line will show up after the second interaction.
    /// ```
    Transition(usize, Vec<Mark>),
    /// An element marks where the previous transition ends:
    /// ```
    /// t----
    /// ```
    TransitionEnd,
    /// A page mark:
    /// ```
    /// ---
    /// ```
    ///
    /// A page is a group of transitions. Transitions from different pages would never appear in the window at the same time.
    Page(Vec<Mark>),
    /// A separator element:
    /// ```
    /// ---- // A horizontal separator.
    /// ----v // A vertical separator.
    /// ```
    Separator(SeparatorDir),
    /// A text element:
    /// ```text
    /// normal text
    /// ## heading 1
    /// ### heading 2
    /// #### heading 3
    /// ##### heading 4
    /// ###### heading 5
    ///
    /// *bold*
    /// `code`
    /// /italics/
    /// $small$
    /// ~strikethrough~
    /// _underline_
    ///
    /// <hyperlink>
    /// [hyperlink title](hyperlink)
    ///
    /// - unordered list
    ///
    /// 1. ordered list
    Text(String, StyleText),
}

/// The direction of the seperator element.
#[derive(Debug)]
pub enum SeparatorDir {
    Horizontal,
    Vertical,
}

/// The configuration of the image element.
#[derive(Debug, Default)]
pub struct StyleImage {
    /// How the image should be aligned horizontally.
    pub align_h: AlignHorizontal,
    /// A string defines the url where the image should link to.
    pub hyperlink: String,
    /// The width of the image. If `None`, the ordinary width of the image will be used.
    pub width: Option<f32>,
    /// The height of the image. If `None`, the ordinary height of the image will be used.
    pub height: Option<f32>,
}

impl StyleImage {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_align_h(mut self, align_h: AlignHorizontal) -> Self {
        self.align_h = align_h;
        self
    }

    #[inline]
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    #[inline]
    pub fn with_hyperlink(mut self, hyperlink: String) -> Self {
        self.hyperlink = hyperlink;
        self
    }

    #[inline]
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

/// The configuration of the text element.
#[derive(Debug, Default)]
pub struct StyleText {
    /// Should the text be bold.
    pub bold: bool,
    /// Should the text be code-style.
    pub code: bool,
    /// The heading level of the text.
    pub heading: Heading,
    /// The hyperlink the text links to.
    pub hyperlink: String,
    /// Should the text be italics.
    pub italics: bool,
    /// Whether the text is within an ordered/unordered list.
    pub listing: Listing,
    /// Should the text be quote-style.
    pub quote: bool,
    /// Should the text be small.
    pub small: bool,
    /// Should the text be strikethroughed.
    pub strikethrough: bool,
    /// Should the text be underlined.
    pub underline: bool,
}

impl Clone for StyleText {
    fn clone(&self) -> Self {
        StyleText {
            bold: self.bold,
            code: self.code,
            heading: self.heading.clone(),
            hyperlink: self.hyperlink.clone(),
            italics: self.italics,
            listing: self.listing.clone(),
            quote: self.quote,
            small: self.small,
            strikethrough: self.strikethrough,
            underline: self.underline,
        }
    }
}

impl StyleText {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_bold(mut self) -> Self {
        self.bold = true;
        self
    }

    #[inline]
    pub fn with_code(mut self) -> Self {
        self.code = true;
        self
    }

    #[inline]
    pub fn with_heading(mut self, heading: Heading) -> Self {
        self.heading = heading;
        self
    }

    #[inline]
    pub fn with_hyperlink(mut self, hyperlink: String) -> Self {
        self.hyperlink = hyperlink;
        self
    }

    #[inline]
    pub fn with_italics(mut self) -> Self {
        self.italics = true;
        self
    }

    #[inline]
    pub fn with_listing(mut self, listing: Listing) -> Self {
        self.listing = listing;
        self
    }

    #[inline]
    pub fn with_quote(mut self) -> Self {
        self.quote = true;
        self
    }

    #[inline]
    pub fn with_small(mut self) -> Self {
        self.small = true;
        self
    }

    #[inline]
    pub fn with_strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    #[inline]
    pub fn with_underline(mut self) -> Self {
        self.underline = true;
        self
    }
}
