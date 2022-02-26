# OpMark

[![Latest version](https://img.shields.io/crates/v/opmark.svg)](https://crates.io/crates/opmark)
[![Documentation](https://docs.rs/opmark/badge.svg)](https://docs.rs/opmark)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

`OpMark` is an experimental markup language focused on presentation making. It's still in pre-alpha stage.

## Features

- Rich Text
- Ordered/Unordered list
- Image
- Hyperlink

# Example

### A simple OpMark document

```text
## This is Page 1

This is a simple example of *OpMark*.

---

## This is Page 2

### Rich Text
You can markup text using the following syntax:
*bold*
`code`
/italics/
$small$
~strikethrough~
_underline_

### Lists
You can make lists:

- unordered list

1. ordered list as well

### Images
![title of the image](src.png)

### Hyperlinks
[Github](https://github.com/)
```

### Using the parser

```rust
use opmark::Parser;
use std::{
    fs::read_to_string,
    path::Path,
};

fn main() {
    let path = Path::new("./foo/bar.opmark");
    let file_content = read_to_string(path).expect("Failed at reading file");
    let parser = Parser::new(file_content);
    for mark in parser {
        // do some stuff
    }
}
```
