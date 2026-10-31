//! Парсер `@startebnf`: правила `имя = выражение;` в дерево [`RailNode`].
//!
//! Операторы: `,` (конкатенация), `|`, `( )`, `[ ]`, `{ }`, `{ }-`,
//! `n * x`, `x - y`; терминалы в кавычках, `? ... ?`, комментарии `(* *)`.

use plantuml_ast::common::DiagramMetadata;
use plantuml_ast::railroad::{EbnfDiagram, EbnfRule, RailNode};

use crate::error::ParseError;

/// Предел вложенности скобок: глубже разбор мог бы переполнить стек.
const MAX_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Str(String),
    Special(String),
    Comment(String),
    Number(u32),
    Sym(char),
}

/// Парсит диаграмму EBNF.
pub fn parse_ebnf(source: &str) -> crate::Result<EbnfDiagram> {
    let body: String = source
        .lines()
        .filter(|l| {
            let t = l.trim();
            !(t.starts_with("@start") || t.starts_with("@end") || t.starts_with("!pragma"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    let tokens = tokenize(&body)?;
    let mut parser = Parser {
        tokens,
        pos: 0,
        notes: Vec::new(),
    };
    let mut rules = Vec::new();
    while parser.peek().is_some() {
        rules.push(parser.rule()?);
    }
    if rules.is_empty() {
        return Err(ParseError::EmptyDiagram);
    }
    Ok(EbnfDiagram {
        rules,
        notes: parser.notes,
        metadata: DiagramMetadata::default(),
    })
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn tokenize(text: &str) -> crate::Result<Vec<Token>> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    let line_of = |at: usize| chars[..at].iter().filter(|c| **c == '\n').count() + 1;
    let err = |at: usize, message: &str| ParseError::SyntaxError {
        line: line_of(at),
        message: message.to_string(),
    };
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c == '(' && chars.get(i + 1) == Some(&'*') {
            let start = i + 2;
            let mut j = start;
            while j + 1 < chars.len() && !(chars[j] == '*' && chars[j + 1] == ')') {
                j += 1;
            }
            if j + 1 >= chars.len() {
                return Err(err(i, "не закрыт комментарий (* ... *)"));
            }
            let note: String = chars[start..j].iter().collect();
            tokens.push(Token::Comment(note.trim().to_string()));
            i = j + 2;
        } else if c == '"' || c == '\'' {
            let Some(len) = chars[i + 1..].iter().position(|x| *x == c) else {
                return Err(err(i, "не закрыта кавычка"));
            };
            tokens.push(Token::Str(chars[i + 1..i + 1 + len].iter().collect()));
            i += len + 2;
        } else if c == '?' {
            let Some(len) = chars[i + 1..].iter().position(|x| *x == '?') else {
                return Err(err(i, "не закрыта специальная последовательность ? ... ?"));
            };
            let text: String = chars[i + 1..i + 1 + len].iter().collect();
            tokens.push(Token::Special(text.trim().to_string()));
            i += len + 2;
        } else if c.is_ascii_digit()
            && !chars[i..]
                .iter()
                .take_while(|x| is_ident_char(**x))
                .any(|x| !x.is_ascii_digit())
        {
            let digits: String = chars[i..]
                .iter()
                .take_while(|x| x.is_ascii_digit())
                .collect();
            let value = digits
                .parse()
                .map_err(|_| err(i, "слишком большое число"))?;
            i += digits.len();
            tokens.push(Token::Number(value));
        } else if is_ident_char(c) {
            // Дефис внутри имени (`h-tab`) — его часть; с пробелами это уже вычитание.
            let start = i;
            while i < chars.len()
                && (is_ident_char(chars[i])
                    || (chars[i] == '-' && chars.get(i + 1).is_some_and(|n| is_ident_char(*n))))
            {
                i += 1;
            }
            tokens.push(Token::Ident(chars[start..i].iter().collect()));
        } else if "=;,|()[]{}-*".contains(c) {
            tokens.push(Token::Sym(c));
            i += 1;
        } else {
            return Err(err(i, &format!("неожиданный символ «{c}»")));
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    notes: Vec<String>,
}

impl Parser {
    /// Следующий значимый токен; комментарии откладываются в заметки.
    fn peek(&mut self) -> Option<&Token> {
        while let Some(Token::Comment(text)) = self.tokens.get(self.pos) {
            self.notes.push(text.clone());
            self.pos += 1;
        }
        self.tokens.get(self.pos)
    }

    fn eat(&mut self, sym: char) -> bool {
        if self.peek() == Some(&Token::Sym(sym)) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn fail<T>(&self, message: &str) -> crate::Result<T> {
        Err(ParseError::GrammarError(message.to_string()))
    }

    fn rule(&mut self) -> crate::Result<EbnfRule> {
        let name = match self.peek() {
            Some(Token::Ident(name)) => name.clone(),
            _ => return self.fail("ожидалось имя правила"),
        };
        self.pos += 1;
        if !self.eat('=') {
            return self.fail(&format!("после имени «{name}» ожидался знак «=»"));
        }
        let body = if self.peek() == Some(&Token::Sym(';')) {
            RailNode::Empty
        } else {
            self.alternation(0)?
        };
        if !self.eat(';') {
            return self.fail(&format!("правило «{name}» не завершено знаком «;»"));
        }
        Ok(EbnfRule {
            name,
            body,
            notes: std::mem::take(&mut self.notes),
        })
    }

    fn alternation(&mut self, depth: usize) -> crate::Result<RailNode> {
        if depth > MAX_DEPTH {
            return self.fail("слишком глубокая вложенность скобок");
        }
        let mut branches = vec![self.sequence(depth)?];
        while self.eat('|') {
            branches.push(self.sequence(depth)?);
        }
        Ok(if branches.len() == 1 {
            branches.swap_remove(0)
        } else {
            RailNode::Alt(branches)
        })
    }

    fn sequence(&mut self, depth: usize) -> crate::Result<RailNode> {
        let mut items = vec![self.term(depth)?];
        while self.eat(',') {
            items.push(self.term(depth)?);
        }
        Ok(if items.len() == 1 {
            items.swap_remove(0)
        } else {
            RailNode::Seq(items)
        })
    }

    /// `n * x` и `x - y`.
    fn term(&mut self, depth: usize) -> crate::Result<RailNode> {
        if let Some(Token::Number(n)) = self.peek().cloned() {
            self.pos += 1;
            if !self.eat('*') {
                return self.fail("после числа ожидался знак «*»");
            }
            let body = self.factor(depth)?;
            return Ok(RailNode::Loop {
                body: Box::new(body),
                label: Some(format!("{n} times")),
            });
        }
        let first = self.factor(depth)?;
        if self.eat('-') {
            let second = self.factor(depth)?;
            return Ok(RailNode::Seq(vec![
                first,
                RailNode::Label("except".to_string()),
                second,
            ]));
        }
        Ok(first)
    }

    fn factor(&mut self, depth: usize) -> crate::Result<RailNode> {
        let Some(token) = self.peek().cloned() else {
            return self.fail("выражение оборвано");
        };
        self.pos += 1;
        match token {
            Token::Str(text) => Ok(RailNode::Terminal(text)),
            Token::Ident(name) => Ok(RailNode::NonTerminal(name)),
            Token::Special(text) => Ok(RailNode::Special(text)),
            Token::Sym('(') => {
                let inner = self.alternation(depth + 1)?;
                self.close(')', inner)
            }
            Token::Sym('[') => {
                let inner = self.alternation(depth + 1)?;
                let inner = self.close(']', inner)?;
                Ok(RailNode::Alt(vec![RailNode::Empty, inner]))
            }
            Token::Sym('{') => {
                let inner = self.alternation(depth + 1)?;
                let inner = self.close('}', inner)?;
                let looped = RailNode::Loop {
                    body: Box::new(inner),
                    label: None,
                };
                // `{ x }-` — один и более раз, без обхода
                if self.peek() == Some(&Token::Sym('-')) && self.minus_is_postfix() {
                    self.pos += 1;
                    return Ok(looped);
                }
                Ok(RailNode::Alt(vec![RailNode::Empty, looped]))
            }
            other => self.fail(&format!("неожиданный токен {other:?}")),
        }
    }

    /// После `}` знак `-` — постфикс, если за ним конец терма, иначе вычитание.
    fn minus_is_postfix(&self) -> bool {
        matches!(
            self.tokens.get(self.pos + 1),
            None | Some(Token::Sym(';' | ',' | '|' | ')' | ']' | '}'))
        )
    }

    fn close(&mut self, sym: char, inner: RailNode) -> crate::Result<RailNode> {
        if self.eat(sym) {
            Ok(inner)
        } else {
            self.fail(&format!("ожидалась закрывающая скобка «{sym}»"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> EbnfDiagram {
        parse_ebnf(&format!("@startebnf\n{src}\n@endebnf")).unwrap()
    }

    #[test]
    fn alternation_of_terminals() {
        let d = parse(r#"e1 = "a1" | "b1";"#);
        assert_eq!(d.rules[0].name, "e1");
        assert_eq!(
            d.rules[0].body,
            RailNode::Alt(vec![
                RailNode::Terminal("a1".into()),
                RailNode::Terminal("b1".into())
            ])
        );
    }

    #[test]
    fn sequence_optional_repeat_group() {
        let d = parse("r = a, [b], {c}, (d | e), {f}-;");
        let RailNode::Seq(items) = &d.rules[0].body else {
            panic!("ожидалась последовательность");
        };
        assert_eq!(items[0], RailNode::NonTerminal("a".into()));
        assert!(matches!(&items[1], RailNode::Alt(v) if v[0] == RailNode::Empty));
        assert!(matches!(&items[2], RailNode::Alt(_)));
        assert!(matches!(&items[3], RailNode::Alt(v) if v.len() == 2));
        assert!(matches!(&items[4], RailNode::Loop { .. }));
    }

    #[test]
    fn special_exception_repetition_and_names() {
        let d = parse("a = ? a-z ?;\nh-tab = letter - vowel;\nt = 4 * '2';");
        assert_eq!(d.rules[0].body, RailNode::Special("a-z".into()));
        assert_eq!(d.rules[1].name, "h-tab");
        assert!(matches!(&d.rules[1].body, RailNode::Seq(v) if v.len() == 3));
        assert!(
            matches!(&d.rules[2].body, RailNode::Loop { label: Some(l), .. } if l == "4 times")
        );
    }

    #[test]
    fn comments_attach_to_rules_or_end() {
        let d = parse("(* before *) r = \"a\" (* inside *);\n(* global *)");
        assert_eq!(d.rules[0].notes, vec!["before", "inside"]);
        assert_eq!(d.notes, vec!["global"]);
    }

    #[test]
    fn malformed_rules_are_errors() {
        assert!(parse_ebnf("@startebnf\nr = a\n@endebnf").is_err());
        assert!(parse_ebnf("@startebnf\nr = (a;\n@endebnf").is_err());
        assert!(parse_ebnf("@startebnf\nr = \"a;\n@endebnf").is_err());
    }
}
