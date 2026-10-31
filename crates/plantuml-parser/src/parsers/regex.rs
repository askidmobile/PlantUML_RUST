//! Парсер `@startregex`: регулярное выражение в дерево [`RailNode`].
//!
//! Поддерживается привычный синтаксис: литералы, `\Q..\E`, классы символов,
//! группы, альтернативы и повторы. Грамматика рекурсивная, поэтому pest не
//! нужен: разбор идёт спуском по символам.

use plantuml_ast::common::DiagramMetadata;
use plantuml_ast::railroad::{RailNode, RegexDiagram};

use crate::error::ParseError;

/// Предел вложенности групп: глубже разбор мог бы переполнить стек.
const MAX_DEPTH: usize = 64;

/// Парсит диаграмму: каждая непустая строка — отдельное выражение.
pub fn parse_regex(source: &str) -> crate::Result<RegexDiagram> {
    let mut patterns = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with("@start") || line.starts_with("@end") {
            continue;
        }
        let mut parser = Parser {
            chars: line.chars().collect(),
            pos: 0,
            line: index + 1,
        };
        let node = parser.alternation(0)?;
        if parser.pos < parser.chars.len() {
            return Err(parser.error("extra closing parenthesis"));
        }
        patterns.push(node);
    }
    if patterns.is_empty() {
        return Err(ParseError::EmptyDiagram);
    }
    Ok(RegexDiagram {
        patterns,
        metadata: DiagramMetadata::default(),
    })
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    line: usize,
}

impl Parser {
    fn error(&self, message: &str) -> ParseError {
        ParseError::SyntaxError {
            line: self.line,
            message: message.to_string(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn alternation(&mut self, depth: usize) -> crate::Result<RailNode> {
        if depth > MAX_DEPTH {
            return Err(self.error("groups are nested too deeply"));
        }
        let mut branches = vec![self.sequence(depth)?];
        while self.peek() == Some('|') {
            self.pos += 1;
            branches.push(self.sequence(depth)?);
        }
        Ok(if branches.len() == 1 {
            branches.swap_remove(0)
        } else {
            RailNode::Alt(branches)
        })
    }

    fn sequence(&mut self, depth: usize) -> crate::Result<RailNode> {
        let mut items: Vec<RailNode> = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            let atom = self.atom(depth)?;
            let node = self.quantifier(atom);
            // Соседние литералы сливаются в одну рамку, как у PlantUML.
            match (items.last_mut(), &node) {
                (Some(RailNode::Terminal(prev)), RailNode::Terminal(next)) => prev.push_str(next),
                _ => items.push(node),
            }
        }
        Ok(match items.len() {
            0 => RailNode::Empty,
            1 => items.swap_remove(0),
            _ => RailNode::Seq(items),
        })
    }

    fn atom(&mut self, depth: usize) -> crate::Result<RailNode> {
        let c = self.chars[self.pos];
        self.pos += 1;
        match c {
            '(' => self.group(depth),
            '[' => self.class(),
            '\\' => self.escape(),
            '.' | '^' | '$' => Ok(RailNode::Charset(c.to_string())),
            _ => Ok(RailNode::Terminal(c.to_string())),
        }
    }

    fn group(&mut self, depth: usize) -> crate::Result<RailNode> {
        let mut label = None;
        let mut frame = true;
        if self.peek() == Some('?') {
            let rest: String = self.chars[self.pos..].iter().take(4).collect();
            let (skip, text) = if rest.starts_with("?:") {
                frame = false;
                (2, None)
            } else if rest.starts_with("?=") {
                (2, Some("Lookahead"))
            } else if rest.starts_with("?!") {
                (2, Some("Negative lookahead"))
            } else if rest.starts_with("?<=") {
                (3, Some("Lookbehind"))
            } else if rest.starts_with("?<!") {
                (3, Some("Negative lookbehind"))
            } else {
                (0, None)
            };
            if skip > 0 {
                self.pos += skip;
                label = text.map(str::to_string);
            } else if let Some(name) = self.group_name() {
                label = Some(name);
            }
        }
        let body = self.alternation(depth + 1)?;
        if self.peek() != Some(')') {
            return Err(self.error("group parenthesis is not closed"));
        }
        self.pos += 1;
        Ok(if frame {
            RailNode::Group {
                body: Box::new(body),
                label,
            }
        } else {
            body
        })
    }

    /// Имя группы `(?<name>..)` или `(?P<name>..)`; позицию двигает только при успехе.
    fn group_name(&mut self) -> Option<String> {
        let mut at = self.pos + 1;
        if self.chars.get(at) == Some(&'P') {
            at += 1;
        }
        if self.chars.get(at) != Some(&'<') {
            return None;
        }
        let start = at + 1;
        let len = self.chars[start..].iter().position(|c| *c == '>')?;
        let name: String = self.chars[start..start + len].iter().collect();
        self.pos = start + len + 1;
        Some(name)
    }

    fn class(&mut self) -> crate::Result<RailNode> {
        let start = self.pos - 1;
        if self.peek() == Some('^') {
            self.pos += 1;
        }
        // `]` первым символом — обычный символ класса
        if self.peek() == Some(']') {
            self.pos += 1;
        }
        while let Some(c) = self.peek() {
            self.pos += 1;
            match c {
                '\\' => self.pos += 1,
                ']' => {
                    let end = self.pos.min(self.chars.len());
                    return Ok(RailNode::Charset(self.chars[start..end].iter().collect()));
                }
                _ => {}
            }
        }
        Err(self.error("character class is not closed"))
    }

    fn escape(&mut self) -> crate::Result<RailNode> {
        let Some(c) = self.peek() else {
            return Err(self.error("backslash at the end of the expression"));
        };
        self.pos += 1;
        match c {
            'Q' => {
                let mut text = String::new();
                while self.pos < self.chars.len() {
                    if self.chars[self.pos] == '\\' && self.chars.get(self.pos + 1) == Some(&'E') {
                        self.pos += 2;
                        return Ok(RailNode::Terminal(text));
                    }
                    text.push(self.chars[self.pos]);
                    self.pos += 1;
                }
                Ok(RailNode::Terminal(text))
            }
            'p' | 'P' | 'x' | 'u' => {
                let mut text = format!("\\{c}");
                if self.peek() == Some('{') {
                    while let Some(n) = self.peek() {
                        self.pos += 1;
                        text.push(n);
                        if n == '}' {
                            break;
                        }
                    }
                } else if c == 'p' || c == 'P' {
                    if let Some(n) = self.peek() {
                        self.pos += 1;
                        text.push(n);
                    }
                } else {
                    let digits = if c == 'u' { 4 } else { 2 };
                    while text.len() < 2 + digits
                        && self.peek().is_some_and(|n| n.is_ascii_hexdigit())
                    {
                        text.push(self.chars[self.pos]);
                        self.pos += 1;
                    }
                }
                Ok(RailNode::Charset(text))
            }
            '0' => {
                let mut text = String::from("\\0");
                while text.len() < 5 && self.peek().is_some_and(|n| ('0'..='7').contains(&n)) {
                    text.push(self.chars[self.pos]);
                    self.pos += 1;
                }
                Ok(RailNode::Charset(text))
            }
            'd' | 'D' | 'w' | 'W' | 's' | 'S' | 'b' | 'B' | 'A' | 'z' | 'Z' | 't' | 'r' | 'n'
            | 'a' | 'e' | 'f' => Ok(RailNode::Charset(format!("\\{c}"))),
            _ => Ok(RailNode::Terminal(c.to_string())),
        }
    }

    /// Применяет повтор `* + ? {n,m}` (и ленивый/сверхжадный суффикс) к атому.
    fn quantifier(&mut self, atom: RailNode) -> RailNode {
        let (min, max) = match self.peek() {
            Some('*') => (0, None),
            Some('+') => (1, None),
            Some('?') => (0, Some(1)),
            Some('{') => match self.braces() {
                Some(range) => {
                    self.skip_greed_suffix();
                    return wrap_repeat(atom, range.0, range.1);
                }
                None => return atom,
            },
            _ => return atom,
        };
        self.pos += 1;
        self.skip_greed_suffix();
        wrap_repeat(atom, min, max)
    }

    /// Ленивость `?` и сверхжадность `+` на форму схемы не влияют.
    fn skip_greed_suffix(&mut self) {
        if matches!(self.peek(), Some('?' | '+')) {
            self.pos += 1;
        }
    }

    /// Разбирает `{n}`, `{n,}`, `{n,m}`; на любой другой форме `{` — обычный символ.
    fn braces(&mut self) -> Option<(u32, Option<u32>)> {
        let close = self.chars[self.pos..].iter().position(|c| *c == '}')?;
        let inner: String = self.chars[self.pos + 1..self.pos + close].iter().collect();
        let parsed = match inner.split_once(',') {
            None => inner.parse().ok().map(|n| (n, Some(n))),
            Some((lo, "")) => lo.parse().ok().map(|n| (n, None)),
            Some((lo, hi)) => lo
                .parse()
                .ok()
                .zip(hi.parse().ok())
                .map(|(a, b)| (a, Some(b))),
        }?;
        self.pos += close + 1;
        Some(parsed)
    }
}

/// Оборачивает атом в повтор: необязательность — это ветка-обход рядом с телом.
fn wrap_repeat(atom: RailNode, min: u32, max: Option<u32>) -> RailNode {
    let looped = |label: Option<String>| RailNode::Loop {
        body: Box::new(atom.clone()),
        label,
    };
    let body = match (min, max) {
        (0 | 1, Some(0 | 1)) => atom.clone(),
        (0 | 1, None) => looped(None),
        (n, None) => looped(Some(format!("{n}+ times"))),
        (n, Some(m)) if n == m => looped(Some(format!("{n} times"))),
        (0, Some(m)) => looped(Some(format!("up to {m} times"))),
        (n, Some(m)) => looped(Some(format!("{n} to {m} times"))),
    };
    if min == 0 {
        RailNode::Alt(vec![RailNode::Empty, body])
    } else {
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(src: &str) -> RailNode {
        parse_regex(&format!("@startregex\n{src}\n@endregex"))
            .unwrap()
            .patterns
            .remove(0)
    }

    #[test]
    fn literals_merge_into_one_terminal() {
        assert_eq!(one("abc"), RailNode::Terminal("abc".into()));
    }

    #[test]
    fn alternation_and_classes() {
        assert_eq!(
            one("a|[0-9]"),
            RailNode::Alt(vec![
                RailNode::Terminal("a".into()),
                RailNode::Charset("[0-9]".into())
            ])
        );
        assert_eq!(one("\\d"), RailNode::Charset("\\d".into()));
    }

    #[test]
    fn quantifiers_wrap_only_last_atom() {
        let RailNode::Seq(items) = one("ab+") else {
            panic!("expected a sequence");
        };
        assert_eq!(items[0], RailNode::Terminal("a".into()));
        assert!(matches!(items[1], RailNode::Loop { label: None, .. }));
        assert!(matches!(one("x*"), RailNode::Alt(_)));
        assert!(matches!(
            one("x{2,5}"),
            RailNode::Loop { label: Some(_), .. }
        ));
    }

    #[test]
    fn groups_named_and_plain() {
        assert!(matches!(one("(a|b)"), RailNode::Group { label: None, .. }));
        assert!(matches!(
            one("(?<id>a)"),
            RailNode::Group { label: Some(ref n), .. } if n == "id"
        ));
        assert_eq!(one("(?:a)"), RailNode::Terminal("a".into()));
    }

    #[test]
    fn unbalanced_input_is_an_error() {
        assert!(parse_regex("@startregex\n(a\n@endregex").is_err());
        assert!(parse_regex("@startregex\na)\n@endregex").is_err());
        assert!(parse_regex("@startregex\n[a\n@endregex").is_err());
    }
}
