//! AST типы для Salt (Wireframe/GUI) диаграмм
//!
//! Salt диаграммы используются для создания wireframes и UI mockups.
//!
//! Синтаксис PlantUML:
//! ```text
//! @startsalt
//! {
//!   Login    | "MyName   "
//!   Password | "****     "
//!   [Cancel] | [  OK   ]
//! }
//! @endsalt
//! ```

use serde::{Deserialize, Serialize};

use crate::common::DiagramMetadata;

/// Salt (Wireframe) диаграмма
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaltDiagram {
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
    /// Корневой элемент (контейнер)
    pub root: SaltWidget,
}

impl SaltDiagram {
    /// Создаёт новую пустую Salt диаграмму
    pub fn new() -> Self {
        Self::default()
    }

    /// Устанавливает корневой элемент
    pub fn with_root(mut self, root: SaltWidget) -> Self {
        self.root = root;
        self
    }
}

/// Виджет Salt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SaltWidget {
    /// Контейнер (grid layout)
    Container(Container),
    /// Текст
    Text(String),
    /// Кнопка `[Button]`
    Button(String),
    /// Текстовое поле "text"
    TextField(String),
    /// Радио-кнопка () или (X)
    Radio { label: String, checked: bool },
    /// Чекбокс `[]` или `[X]`
    Checkbox { label: String, checked: bool },
    /// Выпадающий список ^item^
    Droplist { items: Vec<String>, open: bool },
    /// Разделитель (.., ==, ~~, --)
    Separator(SeparatorType),
    /// Дерево {T}
    Tree(TreeNode),
    /// Вкладки {/}
    Tabs { items: Vec<String>, selected: usize },
    /// Меню {*}
    Menu { items: Vec<MenuItem> },
    /// Группа с заголовком {^"title"}
    GroupBox {
        title: String,
        content: Box<SaltWidget>,
    },
    /// Скроллируемая область {S}, {SI}, {S-}
    ScrollArea {
        content: Box<SaltWidget>,
        scrollbar: ScrollbarType,
    },
    /// Пустая ячейка (.)
    Empty,
    /// Ячейка span (*)
    Span,
}

impl Default for SaltWidget {
    fn default() -> Self {
        Self::Container(Container::default())
    }
}

/// Контейнер (grid layout)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Container {
    /// Строки контейнера
    pub rows: Vec<Vec<SaltWidget>>,
    /// Стиль границ
    pub border_style: BorderStyle,
}

impl Container {
    /// Создаёт новый контейнер
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет строку
    pub fn add_row(&mut self, row: Vec<SaltWidget>) {
        self.rows.push(row);
    }

    /// Устанавливает стиль границ
    pub fn with_border(mut self, style: BorderStyle) -> Self {
        self.border_style = style;
        self
    }
}

/// Стиль границ контейнера
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BorderStyle {
    /// Без границ (по умолчанию)
    #[default]
    None,
    /// Все линии (#)
    All,
    /// Только вертикальные (!)
    Vertical,
    /// Только горизонтальные (-)
    Horizontal,
    /// Только внешние (+)
    External,
}

impl BorderStyle {
    /// Парсит стиль границ из символа
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '#' => Some(Self::All),
            '!' => Some(Self::Vertical),
            '-' => Some(Self::Horizontal),
            '+' => Some(Self::External),
            _ => None,
        }
    }
}

/// Тип разделителя
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeparatorType {
    /// Точечный (..)
    Dotted,
    /// Двойная линия (==)
    Double,
    /// Волнистый (~~)
    Wavy,
    /// Одинарная линия (--)
    Single,
}

/// Узел дерева
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    /// Текст узла
    pub text: String,
    /// Дочерние узлы
    pub children: Vec<TreeNode>,
    /// Уровень вложенности
    pub level: usize,
}

impl TreeNode {
    /// Создаёт новый узел дерева
    pub fn new(text: impl Into<String>, level: usize) -> Self {
        Self {
            text: text.into(),
            children: Vec::new(),
            level,
        }
    }

    /// Добавляет дочерний узел
    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }
}

/// Элемент меню
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    /// Текст элемента
    pub text: String,
    /// Подменю (если есть)
    pub submenu: Option<Vec<MenuItem>>,
    /// Разделитель
    pub is_separator: bool,
}

impl MenuItem {
    /// Создаёт новый элемент меню
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            submenu: None,
            is_separator: false,
        }
    }

    /// Создаёт разделитель
    pub fn separator() -> Self {
        Self {
            text: String::new(),
            submenu: None,
            is_separator: true,
        }
    }
}

/// Тип скроллбара
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ScrollbarType {
    /// Оба скроллбара {S}
    #[default]
    Both,
    /// Только вертикальный {SI}
    Vertical,
    /// Только горизонтальный {S-}
    Horizontal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_container() {
        let mut container = Container::new().with_border(BorderStyle::External);
        container.add_row(vec![
            SaltWidget::Text("Login".to_string()),
            SaltWidget::TextField("MyName".to_string()),
        ]);
        container.add_row(vec![
            SaltWidget::Button("Cancel".to_string()),
            SaltWidget::Button("OK".to_string()),
        ]);

        assert_eq!(container.rows.len(), 2);
        assert_eq!(container.border_style, BorderStyle::External);
    }

    #[test]
    fn test_tree_node() {
        let mut root = TreeNode::new("World", 0);
        let mut america = TreeNode::new("America", 1);
        america.add_child(TreeNode::new("USA", 2));
        america.add_child(TreeNode::new("Canada", 2));
        root.add_child(america);

        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].children.len(), 2);
    }

    #[test]
    fn test_border_style_from_char() {
        assert_eq!(BorderStyle::from_char('#'), Some(BorderStyle::All));
        assert_eq!(BorderStyle::from_char('+'), Some(BorderStyle::External));
        assert_eq!(BorderStyle::from_char('x'), None);
    }
}
