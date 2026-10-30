//! Предел вложенности исходника.
//!
//! Грамматики и обходы AST рекурсивны, поэтому глубина схемы превращается в
//! глубину стека. Переполнение стека нельзя перехватить: оно завершает весь
//! процесс встраивающего приложения. Поэтому глубина проверяется до разбора,
//! и лишняя вложенность становится обычной ошибкой.

use crate::error::ParseError;
use crate::Result;

/// Сколько уровней вложенности допускает разбор.
///
/// Настоящие схемы не превышают пары десятков уровней, а 64 уровня проходят
/// весь конвейер на стеке 2 МБ (`robustness_tests`).
pub const MAX_NESTING: usize = 64;

/// Ключевые слова, открывающие блок без фигурной скобки.
const OPENERS: &[&str] = &[
    "alt",
    "opt",
    "loop",
    "par",
    "break",
    "critical",
    "group",
    "box",
    "partition",
    "if",
    "while",
    "repeat",
    "fork",
    "split",
    "switch",
];

/// Ключевые слова, закрывающие такой блок.
const CLOSERS: &[&str] = &[
    "end",
    "endif",
    "endwhile",
    "endfor",
    "endswitch",
    "end if",
    "end while",
    "end fork",
    "end split",
    "end box",
    "end group",
    "end switch",
    "repeat while",
    "repeatwhile",
];

/// Строка начинается с этого слова целиком.
fn starts_with_word(line: &str, word: &str) -> bool {
    line.strip_prefix(word)
        .is_some_and(|rest| !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
}

/// Проверяет, что вложенность исходника не превышает [`MAX_NESTING`].
pub fn check_nesting(source: &str) -> Result<()> {
    let mut blocks = 0usize;
    let mut brackets = 0usize;

    for (index, raw) in source.lines().enumerate() {
        let line = raw.trim();
        let too_deep = || ParseError::TooDeep {
            limit: MAX_NESTING,
            line: index + 1,
        };

        // Дерево задаётся длиной маркера: `***** узел` в mindmap и WBS.
        let marker = line
            .chars()
            .take_while(|c| matches!(c, '*' | '+' | '-' | '#'))
            .count();
        // YAML задаёт глубину отступом; два пробела считаем за уровень.
        let indent = raw.chars().take_while(|c| *c == ' ').count() / 2;
        if marker > MAX_NESTING || indent > MAX_NESTING {
            return Err(too_deep());
        }

        let lower = line.to_lowercase();
        let is_message = lower.contains("->") || lower.contains("<-") || lower.contains("--");
        if !is_message {
            if CLOSERS.iter().any(|w| starts_with_word(&lower, w)) {
                blocks = blocks.saturating_sub(1);
            } else if OPENERS.iter().any(|w| starts_with_word(&lower, w)) {
                blocks += 1;
            }
        }

        // Круглые скобки не переходят на другую строку, а `{` и `[` могут: JSON.
        let mut parens = 0usize;
        for c in line.chars() {
            match c {
                '{' | '[' => brackets += 1,
                '}' | ']' => brackets = brackets.saturating_sub(1),
                '(' => parens += 1,
                ')' => parens = parens.saturating_sub(1),
                _ => {}
            }
            if parens > MAX_NESTING || brackets > MAX_NESTING {
                return Err(too_deep());
            }
        }
        if blocks > MAX_NESTING {
            return Err(too_deep());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nesting_at_limit_is_allowed() {
        let source = "alt a\n".repeat(MAX_NESTING) + &"end\n".repeat(MAX_NESTING);
        assert!(check_nesting(&source).is_ok());
    }

    #[test]
    fn test_nesting_over_limit_reports_line() {
        let source = "alt a\n".repeat(MAX_NESTING + 1);
        match check_nesting(&source) {
            Err(ParseError::TooDeep { line, .. }) => assert_eq!(line, MAX_NESTING + 1),
            other => panic!("ожидалась ошибка глубины: {other:?}"),
        }
    }

    /// Много блоков подряд — это не вложенность.
    #[test]
    fn test_sequential_blocks_are_not_nesting() {
        let source = "alt a\nend\n".repeat(1000);
        assert!(check_nesting(&source).is_ok());
    }

    /// Участник с именем ключевого слова не открывает блок.
    #[test]
    fn test_message_from_keyword_named_participant_is_not_a_block() {
        let source = "box -> Alice : x\n".repeat(1000);
        assert!(check_nesting(&source).is_ok());
    }
}
