//! Parser for OpMark.

use crate::mark::{
    AlignHorizontal, Heading, IndentLevel, Listing, Mark, SeparatorDir, StyleImage, StyleText,
};
use std::collections::HashMap;

/// Parser for OpMark.
#[derive(Debug, Default)]
pub struct Parser {
    s: String,
    first_page_return: bool,
    indent_level: u8,
    indent_orderer_number_map: HashMap<IndentLevel, u8>,
    is_line_start: bool,
    is_ordered: bool,
    is_unordered: bool,
    style_text: StyleText,
    transition_order: usize,
    ordered_list_current_indent_level_int: u8,
}

impl Parser {
    /// Create a new Parser for OpMark, where `s` is the text content of the OpMark document.
    pub fn new(s: String) -> Self {
        let mut indent_orderer_number_map = HashMap::new();
        indent_orderer_number_map.insert(IndentLevel::None, 0);
        indent_orderer_number_map.insert(IndentLevel::I1, 0);
        indent_orderer_number_map.insert(IndentLevel::I2, 0);
        indent_orderer_number_map.insert(IndentLevel::I3, 0);
        indent_orderer_number_map.insert(IndentLevel::I4, 0);
        indent_orderer_number_map.insert(IndentLevel::I5, 0);
        Self {
            s,
            indent_orderer_number_map,
            is_line_start: true,
            ..Default::default()
        }
    }

    /// Convert the OpMark text content into vector of pages.
    ///
    /// A page would contain three fields:
    ///
    /// 1. The Mark::Page element.
    /// 2. Count of Mark::Transition element in this page.
    /// 3. The index of the Mark::Transition element which just appeared.
    pub fn into_pages(iter: Self) -> Vec<(Mark, usize, usize)> {
        let mut pages: Vec<(Mark, usize, usize)> = vec![];
        for mark in iter {
            match mark {
                Mark::Page(..) => {
                    pages.push((mark, 0, 0));
                }
                _ => {
                    let mut pages_len = pages.len();
                    // push empty page
                    if pages_len <= 0 {
                        pages.push((Mark::Page(vec![]), 0, 0));
                        pages_len = 1;
                    }
                    if let (Mark::Page(transitions), max_transition_idx, _) =
                        &mut pages[pages_len - 1]
                    {
                        let transitions_len = transitions.len();
                        let is_transition_end = if let Mark::TransitionEnd = mark {
                            true
                        } else {
                            false
                        };
                        // push transition
                        if let Mark::Transition(order, _) = mark {
                            transitions.push(mark);
                            if order > *max_transition_idx {
                                *max_transition_idx = order;
                            }
                            continue;
                        }
                        // push empty transition
                        if transitions_len <= 0 || is_transition_end {
                            transitions.push(Mark::Transition(0, vec![]));
                            if is_transition_end {
                                continue;
                            }
                        }
                        // push mark to current transition
                        if let Mark::Transition(_, marks) = &mut transitions[transitions_len - 1] {
                            marks.push(mark);
                        }
                    }
                }
            }
        }

        pages
    }

    /// ``code``
    fn code(&mut self) -> Option<Mark> {
        if self.s.starts_with("`") {
            let this_line = &self.s[..self.s.find("\n").unwrap_or_else(|| self.s.len())];
            if let Some(end) = this_line[1..].find("`") {
                let text = this_line[1..end + 1].to_owned();
                self.s = self.s[end + 3..].to_owned();
                return Some(Mark::Text(text, StyleText::new().with_code()));
            }
        }
        None
    }

    /// ````language
    /// code
    /// ````
    fn code_block(&mut self) -> Option<Mark> {
        if self.s.starts_with("```") {
            if let Some(cb_end) = self.s.find("\n```") {
                let first_line_end = self.s.find('\n').unwrap_or_else(|| self.s.len());
                let first_line = self.s[3..first_line_end].to_owned();
                let language = if first_line.len() > 0 {
                    Some(first_line)
                } else {
                    None
                };
                let code = self.s[first_line_end + 1..cb_end].to_owned();
                self.s = self.s[cb_end + 4..].to_owned();
                return Some(Mark::CodeBlock(code, language));
            }
        }
        None
    }

    /// `# Heading`
    fn heading(&mut self) -> Option<Mark> {
        if self.s.starts_with("#") {
            let line_end = self.s.find("\n").unwrap_or_else(|| self.s.len());
            let this_line = &self.s[..line_end];
            if this_line.len() > 2 {
                let mut idx = 1;
                let mut c = this_line.chars().nth(idx).unwrap();
                let mut heading_level = 1;
                while c == '#' && idx < this_line.len() - 1 {
                    heading_level += 1;
                    idx += 1;
                    c = this_line.chars().nth(idx).unwrap();
                }

                let hash_end = idx - 1;
                if let Some(text) = this_line[hash_end + 1..].strip_prefix(' ') {
                    let text = text.to_owned();
                    let heading = Heading::from(heading_level);
                    let style = StyleText::new().with_heading(heading);
                    self.s = self.s[line_end..].to_owned();
                    self.is_line_start = false;

                    return Some(Mark::Text(text, style));
                }
            }
        }
        None
    }

    /// `<url>`, `[title](url)`
    fn hyperlink(&mut self) -> Option<Mark> {
        if self.s.starts_with("<") {
            let this_line = &self.s[..self.s.find('\n').unwrap_or_else(|| self.s.len())];
            if let Some(angle_end) = this_line.find('>') {
                let url = this_line[1..angle_end].to_owned();
                self.s = self.s[angle_end + 1..].to_owned();
                self.is_line_start = false;
                return Some(Mark::Text(
                    url.clone(),
                    StyleText::new().with_hyperlink(url),
                ));
            }
        }
        if self.s.starts_with("[") {
            let this_line = &self.s[..self.s.find('\n').unwrap_or_else(|| self.s.len())];
            if let Some(bracket_end) = this_line.find(']') {
                if this_line[bracket_end + 1..].starts_with('(') {
                    if let Some(parens_end) = this_line[bracket_end + 2..].find(')') {
                        let parens_end = bracket_end + 2 + parens_end;
                        let title = this_line[1..bracket_end].to_owned();
                        let url = this_line[bracket_end + 2..parens_end].to_owned();
                        self.s = self.s[parens_end + 1..].to_owned();
                        self.is_line_start = false;
                        return Some(Mark::Text(title, StyleText::new().with_hyperlink(url)));
                    }
                }
            }
        }
        None
    }

    /// `![title](src)<options>`
    fn image(&mut self) -> Option<Mark> {
        if self.s.starts_with("![") {
            let this_line = &self.s[..self.s.find('\n').unwrap_or_else(|| self.s.len())];
            if let Some(bracket_end) = this_line.find(']') {
                if this_line[bracket_end + 1..].starts_with('(') {
                    if let Some(parens_end) = this_line[bracket_end + 2..].find(')') {
                        let parens_end = bracket_end + 2 + parens_end;
                        let title = this_line[2..bracket_end].to_owned();
                        let url = this_line[bracket_end + 2..parens_end].to_owned();
                        let mut image_end = parens_end;
                        let mut style = StyleImage::new();
                        if this_line[parens_end + 1..].starts_with('<') {
                            // find image options
                            if let Some(angle_end) = this_line[parens_end + 2..].find('>') {
                                image_end = image_end + angle_end + 2;
                                let angle_end = parens_end + 2 + angle_end;
                                let options = this_line[parens_end + 2..angle_end].to_owned();
                                for option in options.split('|').collect::<Vec<&str>>() {
                                    style = match option {
                                        "auto" => style.with_align_h(AlignHorizontal::Auto),
                                        "left" => style.with_align_h(AlignHorizontal::Left),
                                        "right" => style.with_align_h(AlignHorizontal::Right),
                                        "center" => style.with_align_h(AlignHorizontal::Center),
                                        _ => {
                                            if option.starts_with('w') {
                                                match option[1..].parse::<f32>() {
                                                    Ok(n) => style.with_width(n),
                                                    _ => style.with_hyperlink(option.to_owned()),
                                                }
                                            } else if option.starts_with('h') {
                                                match option[1..].parse::<f32>() {
                                                    Ok(n) => style.with_height(n),
                                                    _ => style.with_hyperlink(option.to_owned()),
                                                }
                                            } else {
                                                style.with_hyperlink(option.to_owned())
                                            }
                                        }
                                    };
                                }
                            }
                        }
                        self.s = self.s[image_end + 1..].to_owned();
                        self.is_line_start = false;

                        return Some(Mark::Image(url, title, style));
                    }
                }
            }
        }
        None
    }

    /// `1. ordered list`
    fn ordered_list(&mut self) -> Option<Mark> {
        let line_end = self.s.find("\n").unwrap_or_else(|| self.s.len());
        let this_line = &self.s[..line_end];
        let indent_level = indent(this_line);
        let indent = (indent_level.to_int() * 2) as usize;

        let mut idx = indent;
        let mut b = this_line.as_bytes()[idx];
        while b.is_ascii_digit() && idx < this_line.len() - 1 {
            idx += 1;
            b = this_line.as_bytes()[idx];
        }

        if this_line[idx..].starts_with(". ") {
            let ordered_number = if self.is_ordered
                && self.ordered_list_current_indent_level_int >= indent_level.to_int()
            {
                let previous_number = self
                    .indent_orderer_number_map
                    .get(&indent_level)
                    .unwrap_or(&0);
                previous_number + 1
            } else {
                1
            };
            if let Some(number) = self.indent_orderer_number_map.get_mut(&indent_level) {
                *number = ordered_number;
            }
            let text = this_line[idx + 2..].to_owned();
            self.s = self.s[line_end..].to_owned();
            self.is_line_start = false;
            self.is_ordered = true;
            self.ordered_list_current_indent_level_int = indent_level.to_int();
            return Some(Mark::Text(
                text,
                StyleText::new().with_listing(Listing::Ordered(ordered_number, indent_level)),
            ));
        }
        None
    }

    fn reset_indent_orderer_number_map(&mut self) {
        self.indent_orderer_number_map.insert(IndentLevel::None, 0);
        self.indent_orderer_number_map.insert(IndentLevel::I1, 0);
        self.indent_orderer_number_map.insert(IndentLevel::I2, 0);
        self.indent_orderer_number_map.insert(IndentLevel::I3, 0);
        self.indent_orderer_number_map.insert(IndentLevel::I4, 0);
        self.indent_orderer_number_map.insert(IndentLevel::I5, 0);
    }

    /// `> quote`
    fn quote(&mut self) -> Option<Mark> {
        if self.s.starts_with("> ") {
            let line_end = self.s.find("\n").unwrap_or_else(|| self.s.len());
            let this_line = &self.s[..line_end];
            let text = this_line[2..].to_owned();
            self.s = self.s[line_end..].to_owned();
            self.is_line_start = false;
            return Some(Mark::Text(text, StyleText::new().with_quote()));
        }
        None
    }

    fn separator(&mut self) -> Option<Mark> {
        if let Some(rest) = self.s.strip_prefix("----\n") {
            self.s = rest.to_owned();
            return Some(Mark::Separator(SeparatorDir::Horizontal));
        }
        if let Some(rest) = self.s.strip_prefix("----v\n") {
            self.s = rest.to_owned();
            return Some(Mark::Separator(SeparatorDir::Vertical));
        }
        None
    }

    /// `---t`, `---t1`
    fn transition(&mut self) -> Option<Mark> {
        if self.s.starts_with("---t") {
            let this_line = &self.s[..self.s.find('\n').unwrap_or_else(|| self.s.len())];
            let order = if this_line.len() > 4 {
                let mut idx = 4;
                let mut b = this_line.as_bytes()[idx];
                while b.is_ascii_digit() && idx < this_line.len() - 1 {
                    idx += 1;
                    b = this_line.as_bytes()[idx];
                }
                idx += 1;
                if idx == this_line.len() {
                    match this_line[4..idx].parse::<usize>() {
                        Ok(x) => x,
                        _ => self.transition_order,
                    }
                } else {
                    self.transition_order
                }
            } else {
                self.transition_order
            };
            self.transition_order = order + 1;
            self.s = self.s[this_line.len()..].to_owned();
            return Some(Mark::Transition(order, vec![]));
        }
        None
    }

    /// `- unordered list`
    fn unordered_list(&mut self) -> Option<Mark> {
        let line_end = self.s.find("\n").unwrap_or_else(|| self.s.len());
        let this_line = &self.s[..line_end];
        let indent_level = indent(this_line);
        let indent = (indent_level.to_int() * 2) as usize;
        if self.s[indent..].starts_with("- ") {
            let text = this_line[indent + 1..].to_owned();
            self.s = self.s[line_end..].to_owned();
            self.is_line_start = false;
            self.is_unordered = true;
            return Some(Mark::Text(
                text,
                StyleText::new().with_listing(Listing::Unordered(indent_level)),
            ));
        }
        None
    }
}

impl Iterator for Parser {
    type Item = Mark;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.first_page_return {
            self.first_page_return = true;
            return Some(Mark::Page(vec![]));
        }

        if self.transition_order == 0 {
            self.transition_order = 1;
            return Some(Mark::Transition(0, vec![]));
        }

        loop {
            if self.s.is_empty() {
                return None;
            }

            if let Some(rest) = self.s.strip_prefix('\n') {
                self.s = rest.to_owned();
                self.indent_level = 0;
                self.is_line_start = true;
                self.style_text = StyleText::new();
                let is_empty = self.s.is_empty();
                if self.s.starts_with('\n') || is_empty {
                    if !is_empty {
                        self.s = self.s[1..].to_owned();
                    }
                    self.is_ordered = false;
                    self.is_unordered = false;
                    self.ordered_list_current_indent_level_int = 0;
                    self.reset_indent_orderer_number_map();
                    return Some(Mark::NewLine);
                }
            }

            if self.is_line_start {
                if let Some(rest) = self.s.strip_prefix("---\n") {
                    self.s = rest.to_owned();
                    self.transition_order = 0;
                    return Some(Mark::Page(vec![]));
                }

                if let Some(mark) = self.transition() {
                    return Some(mark);
                }

                if let Some(rest) = self.s.strip_prefix("t---\n") {
                    self.s = rest.to_owned();
                    return Some(Mark::TransitionEnd);
                }

                if let Some(mark) = self.code_block() {
                    return Some(mark);
                }

                if let Some(mark) = self.heading() {
                    return Some(mark);
                }

                if let Some(mark) = self.image() {
                    return Some(mark);
                }

                if let Some(mark) = self.ordered_list() {
                    return Some(mark);
                }

                if let Some(mark) = self.quote() {
                    return Some(mark);
                }

                if let Some(mark) = self.separator() {
                    return Some(mark);
                }

                if let Some(mark) = self.unordered_list() {
                    return Some(mark);
                }
            }

            // `*bold*`
            if let Some(rest) = self.s.strip_prefix('*') {
                self.s = rest.to_owned();
                self.is_line_start = false;
                self.style_text.bold = !self.style_text.bold;
                continue;
            }

            if let Some(mark) = self.code() {
                return Some(mark);
            }

            if let Some(mark) = self.hyperlink() {
                return Some(mark);
            }

            // `/italics/`
            if let Some(rest) = self.s.strip_prefix('/') {
                self.s = rest.to_owned();
                self.is_line_start = false;
                self.style_text.italics = !self.style_text.italics;
                continue;
            }

            // `$small$`
            if let Some(rest) = self.s.strip_prefix('$') {
                self.s = rest.to_owned();
                self.is_line_start = false;
                self.style_text.small = !self.style_text.small;
                continue;
            }

            // `~strikethrough~`
            if let Some(rest) = self.s.strip_prefix('~') {
                self.s = rest.to_owned();
                self.is_line_start = false;
                self.style_text.strikethrough = !self.style_text.strikethrough;
                continue;
            }

            // `_underline_`
            if let Some(rest) = self.s.strip_prefix('_') {
                self.s = rest.to_owned();
                self.is_line_start = false;
                self.style_text.underline = !self.style_text.underline;
                continue;
            }

            // \ escape
            if self.s.starts_with('\\') && self.s.len() >= 2 {
                let text = self.s[1..2].to_owned();
                self.s = self.s[2..].to_owned();
                self.is_line_start = false;
                return Some(Mark::Text(text, StyleText::new()));
            }

            let end = self
                .s
                .find(&['*', '`', '~', '_', '/', '$', '^', '\\', '<', '[', '\n'][..])
                .map_or_else(|| self.s.len(), |special| special.max(1));
            let text = Mark::Text(self.s[..end].to_owned(), self.style_text.clone());
            self.s = self.s[end..].to_owned();
            self.is_line_start = false;
            return Some(text);
        }
    }
}

/// find indent level
fn indent(s: &str) -> IndentLevel {
    let mut idx = 0;
    let mut c = s.chars().nth(idx).unwrap();
    let mut indent_level = 0;
    while c == ' ' && idx < s.len() - 1 {
        indent_level += 1;
        idx += 1;
        c = s.chars().nth(idx).unwrap();
    }
    indent_level = indent_level / 2;
    IndentLevel::from(indent_level)
}
