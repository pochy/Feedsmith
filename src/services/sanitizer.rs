use scraper::{Html, Selector};

pub fn sanitize_preview_html(html: &str) -> String {
    let mut out = html.to_string();
    for tag in ["script", "iframe", "object", "embed"] {
        out = remove_tag_blocks(&out, tag);
    }
    remove_event_handlers(&out)
}

fn remove_tag_blocks(input: &str, tag: &str) -> String {
    let document = Html::parse_document(input);
    let selector = Selector::parse(tag).expect("static selector");
    if document.select(&selector).next().is_none() {
        return input.to_string();
    }
    // Lightweight fallback sanitizer: remove common dangerous blocks without trying to
    // preserve exact broken-HTML structure.
    let mut result = input.to_string();
    loop {
        let lower = result.to_ascii_lowercase();
        let Some(start) = lower.find(&format!("<{tag}")) else {
            break;
        };
        let end_tag = format!("</{tag}>");
        let end = lower[start..]
            .find(&end_tag)
            .map(|idx| start + idx + end_tag.len())
            .or_else(|| lower[start..].find('>').map(|idx| start + idx + 1))
            .unwrap_or(result.len());
        result.replace_range(start..end, "");
    }
    result
}

fn remove_event_handlers(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_whitespace() && matches!(chars.peek(), Some('o' | 'O')) {
            let mut lookahead = String::new();
            while let Some(next) = chars.peek() {
                if *next == '=' || next.is_whitespace() || lookahead.len() > 32 {
                    break;
                }
                lookahead.push(*next);
                chars.next();
            }
            if lookahead.to_ascii_lowercase().starts_with("on") {
                while let Some(next) = chars.peek() {
                    if *next == '=' {
                        chars.next();
                        break;
                    }
                    if !next.is_whitespace() {
                        break;
                    }
                    chars.next();
                }
                if let Some(quote @ ('"' | '\'')) = chars.peek().copied() {
                    chars.next();
                    for next in chars.by_ref() {
                        if next == quote {
                            break;
                        }
                    }
                } else {
                    while let Some(next) = chars.peek() {
                        if next.is_whitespace() || *next == '>' {
                            break;
                        }
                        chars.next();
                    }
                }
                continue;
            }
            result.push(ch);
            result.push_str(&lookahead);
        } else {
            result.push(ch);
        }
    }
    result
}
