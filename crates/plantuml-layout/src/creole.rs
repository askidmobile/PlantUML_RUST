//! Разметка внутри подписей: creole и HTML-подобные теги PlantUML.
//!
//! Разбор живёт здесь, потому что им пользуются и замер ширины, и рендерер:
//! подпись `**важно**` обязана измеряться как `важно`, а не со звёздочками.

/// Кусок подписи с единым оформлением.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub mono: bool,
    /// Цвет; проверен на безопасность для атрибута SVG
    pub color: Option<String>,
    pub size: Option<f64>,
    /// Ссылка; только разрешённые схемы
    pub href: Option<String>,
}

impl Span {
    fn same_style(&self, other: &Span) -> bool {
        self.bold == other.bold
            && self.italic == other.italic
            && self.underline == other.underline
            && self.strike == other.strike
            && self.mono == other.mono
            && self.color == other.color
            && self.size == other.size
            && self.href == other.href
    }
}

/// Есть ли в строке символы, с которых начинается разметка.
pub fn has_markup(line: &str) -> bool {
    line.contains(['*', '/', '_', '-', '"', '~', '<', '[', '|'])
}

/// Подпись без разметки: то, что видит читатель.
pub fn plain(line: &str) -> String {
    if !has_markup(line) {
        return line.to_string();
    }
    parse(line).into_iter().map(|span| span.text).collect()
}

/// Цвет для атрибута SVG: имя или шестнадцатеричное значение, без спецсимволов.
///
/// Подпись может прийти из чужой заметки, поэтому значение не должно закрыть атрибут.
fn safe_color(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let valid = !raw.is_empty()
        && raw.len() <= 32
        && raw
            .strip_prefix('#')
            .unwrap_or(raw)
            .chars()
            .all(|c| c.is_ascii_alphanumeric());
    valid.then(|| raw.to_string())
}

/// Ссылка только с безопасной схемой. Прочие, например `javascript:`, не выводятся.
fn safe_href(raw: &str) -> Option<String> {
    let url = raw.trim();
    let lower = url.to_lowercase();
    let allowed = ["http://", "https://", "mailto:"]
        .iter()
        .any(|scheme| lower.starts_with(scheme));
    (allowed && !url.chars().any(char::is_control)).then(|| url.to_string())
}

/// Значок Open Iconic как символ Unicode. Набор охватывает частые имена.
fn icon_char(name: &str) -> &'static str {
    match name {
        "heart" => "♥",
        "star" => "★",
        "check" => "✓",
        "x" | "circle-x" => "✕",
        "warning" => "⚠",
        "info" => "ℹ",
        "person" => "👤",
        "people" => "👥",
        "cog" => "⚙",
        "wrench" => "🔧",
        "lock-locked" => "🔒",
        "lock-unlocked" => "🔓",
        "key" => "🔑",
        "home" => "⌂",
        "envelope-closed" | "envelope-open" => "✉",
        "file" | "document" => "📄",
        "folder" => "📁",
        "cloud" => "☁",
        "clock" | "timer" => "⏱",
        "calendar" => "📅",
        "flag" => "⚑",
        "bell" => "🔔",
        "magnifying-glass" => "🔍",
        "pencil" => "✎",
        "trash" => "🗑",
        "plus" => "+",
        "minus" => "−",
        "arrow-right" => "→",
        "arrow-left" => "←",
        "arrow-top" => "↑",
        "arrow-bottom" => "↓",
        "chevron-right" => "›",
        "chevron-left" => "‹",
        "account-login" => "⇥",
        "account-logout" => "⇤",
        "bolt" => "⚡",
        "bug" => "🐞",
        "code" => "⌨",
        "globe" => "🌐",
        "link-intact" => "🔗",
        "list" | "list-rich" => "☰",
        "phone" => "☎",
        "print" => "⎙",
        "signal" => "📶",
        "sun" => "☀",
        "moon" => "☾",
        "target" => "◎",
        "thumb-up" => "👍",
        "thumb-down" => "👎",
        _ => "▪",
    }
}

/// Значок emoji по имени или коду `<:1f600:>`; неизвестное имя остаётся именем.
fn emoji_text(name: &str) -> String {
    if let Some(c) = u32::from_str_radix(name, 16).ok().and_then(char::from_u32) {
        return c.to_string();
    }
    let known = match name {
        "smile" | "smiley" => "😄",
        "grinning" => "😀",
        "joy" => "😂",
        "wink" => "😉",
        "heart" => "❤",
        "thumbsup" | "+1" => "👍",
        "thumbsdown" | "-1" => "👎",
        "star" => "⭐",
        "fire" => "🔥",
        "rocket" => "🚀",
        "warning" => "⚠",
        "x" => "❌",
        "white_check_mark" | "heavy_check_mark" => "✅",
        "bulb" => "💡",
        "bug" => "🐛",
        "book" => "📖",
        "lock" => "🔒",
        "key" => "🔑",
        "tada" => "🎉",
        "zap" => "⚡",
        "cloud" => "☁",
        "computer" => "💻",
        "email" | "envelope" => "✉",
        "question" => "❓",
        "exclamation" => "❗",
        _ => return format!(":{name}:"),
    };
    known.to_string()
}

/// Стили, включаемые парными маркерами и тегами.
#[derive(Default, Clone)]
struct State {
    span: Span,
    colors: Vec<Option<String>>,
    sizes: Vec<Option<f64>>,
}

/// Разбирает одну строку подписи на куски.
pub fn parse(line: &str) -> Vec<Span> {
    let chars: Vec<char> = line.chars().collect();
    let mut spans: Vec<Span> = Vec::new();
    let mut state = State::default();
    let mut i = 0;

    while i < chars.len() {
        let rest = &chars[i..];
        let rest_str = || rest.iter().collect::<String>();

        // `~` экранирует разметку; `~~текст~~` — волнистое подчёркивание.
        if rest[0] == '~' {
            if rest.len() > 1 && rest[1] == '~' && closes(&chars, i + 2, ['~', '~']) {
                state.span.underline = !state.span.underline;
                i += 2;
                continue;
            }
            if let Some(next) = rest.get(1) {
                if matches!(next, '*' | '/' | '_' | '-' | '"' | '<' | '[' | '~' | '|') {
                    push(&mut spans, &state.span, *next);
                    i += 2;
                    continue;
                }
            }
        }

        if let Some((a, b)) = rest.first().zip(rest.get(1)) {
            let toggled = match (a, b) {
                ('*', '*') => Some(&mut state.span.bold),
                ('/', '/') => Some(&mut state.span.italic),
                ('_', '_') => Some(&mut state.span.underline),
                ('-', '-') => Some(&mut state.span.strike),
                ('"', '"') => Some(&mut state.span.mono),
                _ => None,
            };
            if let Some(flag) = toggled {
                // Парный маркер действует только при наличии пары: `http://`
                // и стрелка `-->` в подписи остаются текстом.
                if *flag || closes(&chars, i + 2, [*a, *b]) {
                    *flag = !*flag;
                    i += 2;
                    continue;
                }
            }
        }

        if rest[0] == '[' && rest.get(1) == Some(&'[') {
            let text = rest_str();
            if let Some(end) = text.find("]]") {
                let body = &text[2..end];
                add_link(&mut spans, &state.span, body);
                i += text[..end + 2].chars().count();
                continue;
            }
        }

        if rest[0] == '<' && i.checked_sub(1).and_then(|p| chars.get(p)) != Some(&'<') {
            let text = rest_str();
            if let Some(end) = text.find('>') {
                let tag = &text[1..end];
                if apply_tag(tag, &mut state, &mut spans) {
                    i += text[..=end].chars().count();
                    continue;
                }
            }
        }

        push(&mut spans, &state.span, rest[0]);
        i += 1;
    }
    spans
}

/// После позиции есть закрывающая пара маркера.
fn closes(chars: &[char], from: usize, pair: [char; 2]) -> bool {
    chars
        .get(from..)
        .is_some_and(|tail| tail.windows(2).any(|w| w[0] == pair[0] && w[1] == pair[1]))
}

fn push(spans: &mut Vec<Span>, style: &Span, c: char) {
    if let Some(last) = spans.last_mut() {
        if last.same_style(style) {
            last.text.push(c);
            return;
        }
    }
    let mut span = style.clone();
    span.text = c.to_string();
    spans.push(span);
}

fn push_str(spans: &mut Vec<Span>, style: &Span, text: &str) {
    text.chars().for_each(|c| push(spans, style, c));
}

/// `[[url]]`, `[[url подпись]]`, `[[url{подсказка} подпись]]`, `[[{подсказка} подпись]]`.
fn add_link(spans: &mut Vec<Span>, style: &Span, body: &str) {
    let body = body.trim();
    let (head, label) = match body.split_once(char::is_whitespace) {
        Some((head, label)) => (head, label.trim()),
        None => (body, ""),
    };
    let url = head.split('{').next().unwrap_or("");
    let shown = if label.is_empty() { url } else { label };
    let mut linked = style.clone();
    linked.href = safe_href(url);
    linked.underline = linked.href.is_some() || style.underline;
    if linked.href.is_some() {
        linked.color = Some("#00F".to_string());
    }
    push_str(spans, &linked, shown);
}

/// Применяет тег. `false` — это не разметка, например `List<T>`.
fn apply_tag(tag: &str, state: &mut State, spans: &mut Vec<Span>) -> bool {
    match tag {
        "b" => state.span.bold = true,
        "/b" => state.span.bold = false,
        "i" => state.span.italic = true,
        "/i" => state.span.italic = false,
        "u" => state.span.underline = true,
        "/u" => state.span.underline = false,
        "s" | "strike" | "del" => state.span.strike = true,
        "/s" | "/strike" | "/del" => state.span.strike = false,
        "w" | "/w" | "plain" | "/plain" | "sub" | "/sub" | "sup" | "/sup" | "/back" | "/font" => {}
        "/color" => state.span.color = state.colors.pop().flatten(),
        "/size" => state.span.size = state.sizes.pop().flatten(),
        _ => {
            if let Some(name) = tag.strip_prefix('&') {
                let name = name.split('{').next().unwrap_or(name);
                push_str(spans, &state.span, icon_char(name));
            } else if let Some(name) = tag.strip_prefix(':').and_then(|t| t.strip_suffix(':')) {
                push_str(spans, &state.span, &emoji_text(name));
            } else if tag.starts_with('$') {
                // Спрайт в подписи рисуется не текстом: здесь он только не должен быть виден.
            } else if let Some(value) = tag
                .strip_prefix("color:")
                .or_else(|| tag.strip_prefix("color "))
            {
                let color = safe_color(value);
                if color.is_none() {
                    return false;
                }
                state.colors.push(state.span.color.take());
                state.span.color = color;
            } else if let Some(value) = tag
                .strip_prefix("size:")
                .or_else(|| tag.strip_prefix("size "))
            {
                let Some(size) = value
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .filter(|v| *v > 0.0 && *v < 200.0)
                else {
                    return false;
                };
                state.sizes.push(state.span.size.take());
                state.span.size = Some(size);
            } else if tag.starts_with("back:")
                || tag.starts_with("back ")
                || tag.starts_with("font")
            {
                // Фон и гарнитура не меняют текст; цвет шрифта пока не поддержан.
            } else {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(line: &str) -> Span {
        let spans = parse(line);
        assert_eq!(spans.len(), 1, "{spans:?}");
        spans.into_iter().next().unwrap_or_default()
    }

    #[test]
    fn test_paired_markers() {
        assert!(one("**а**").bold);
        assert!(one("//а//").italic);
        assert!(one("__а__").underline);
        assert!(one("--а--").strike);
        assert!(one("\"\"а\"\"").mono);
        assert!(one("~~а~~").underline);
    }

    #[test]
    fn test_unpaired_markers_stay_text() {
        assert_eq!(plain("http://example.com"), "http://example.com");
        assert_eq!(plain("a --> b"), "a --> b");
        assert_eq!(plain("2*3 и 4**5"), "2*3 и 4**5");
        assert_eq!(plain("snake_case__x"), "snake_case__x");
    }

    #[test]
    fn test_tags_and_nesting() {
        let spans = parse("<color:blue>синий</color> и <b>жирный</b>");
        assert_eq!(spans[0].color.as_deref(), Some("blue"));
        assert_eq!(spans[1].text, " и ");
        assert_eq!(spans[1].color, None);
        assert!(spans[2].bold);
        assert_eq!(parse("<size:20>x</size>")[0].size, Some(20.0));
    }

    #[test]
    fn test_generics_and_stereotypes_stay_text() {
        assert_eq!(plain("List<T>"), "List<T>");
        assert_eq!(plain("<<include>>"), "<<include>>");
        assert_eq!(plain("<<s>>"), "<<s>>");
        assert_eq!(plain("a < b > c"), "a < b > c");
    }

    #[test]
    fn test_escape() {
        assert_eq!(plain("~**нет~**"), "**нет**");
    }

    #[test]
    fn test_links_only_safe_schemes() {
        let span = one("[[https://e.example подпись]]");
        assert_eq!(span.text, "подпись");
        assert_eq!(span.href.as_deref(), Some("https://e.example"));

        let bad = one("[[javascript:alert(1) клик]]");
        assert_eq!(bad.text, "клик");
        assert_eq!(bad.href, None);

        let mixed = one("[[JaVaScRiPt:alert(1)]]");
        assert_eq!(mixed.href, None);
    }

    /// Значение цвета не должно выйти за пределы атрибута SVG.
    #[test]
    fn test_color_cannot_break_attribute() {
        assert_eq!(
            plain("<color:red\" onload=\"x>т</color>"),
            "<color:red\" onload=\"x>т"
        );
        assert_eq!(
            parse("<color:#00FF00>т</color>")[0].color.as_deref(),
            Some("#00FF00")
        );
    }

    #[test]
    fn test_icons_and_emoji_have_no_raw_names() {
        assert_eq!(plain("<&heart> x"), "♥ x");
        assert_eq!(plain("<:smile:> x"), "😄 x");
        assert_eq!(plain("<:1f600:>"), "😀");
        assert_eq!(plain("<$спрайт>x"), "x");
    }
}
