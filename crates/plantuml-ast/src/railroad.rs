//! AST железнодорожных диаграмм: `@startregex` и `@startebnf`.
//!
//! Обе диаграммы рисуются одним движком, поэтому делят дерево [`RailNode`].

use serde::{Deserialize, Serialize};

use crate::common::DiagramMetadata;

/// Узел железнодорожной диаграммы.
///
/// Необязательность и «ноль или больше» отдельных вариантов не имеют:
/// это `Alt([Empty, x])` и `Alt([Empty, Loop(x)])`, как их и рисует PlantUML.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RailNode {
    /// Пустая ветка (обход)
    Empty,
    /// Терминал: литерал в прямоугольнике
    Terminal(String),
    /// Нетерминал: имя другого правила (скруглённая рамка)
    NonTerminal(String),
    /// Специальная последовательность `? ... ?` (шестиугольник)
    Special(String),
    /// Класс символов или якорь regex (серая заливка)
    Charset(String),
    /// Пояснение без рамки
    Label(String),
    /// Последовательность
    Seq(Vec<RailNode>),
    /// Альтернативы
    Alt(Vec<RailNode>),
    /// Повтор: тело и обратная дорожка под ним
    Loop {
        body: Box<RailNode>,
        label: Option<String>,
    },
    /// Группа в рамке (группа захвата regex)
    Group {
        body: Box<RailNode>,
        label: Option<String>,
    },
}

/// Диаграмма регулярных выражений
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegexDiagram {
    /// Одно выражение на непустую строку
    pub patterns: Vec<RailNode>,
    pub metadata: DiagramMetadata,
}

/// Правило EBNF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EbnfRule {
    pub name: String,
    pub body: RailNode,
    /// Комментарии `(* ... *)`, относящиеся к правилу
    pub notes: Vec<String>,
}

/// Диаграмма EBNF
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EbnfDiagram {
    pub rules: Vec<EbnfRule>,
    /// Комментарии вне правил
    pub notes: Vec<String>,
    pub metadata: DiagramMetadata,
}
