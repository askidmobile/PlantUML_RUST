//! AST типы для Sequence Diagrams (диаграмм последовательностей).

use serde::{Deserialize, Serialize};

use crate::common::{Color, DiagramMetadata, Identifier, LineStyle, Note, Stereotype};

/// Диаграмма последовательностей
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SequenceDiagram {
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
    /// Участники диаграммы
    pub participants: Vec<Participant>,
    /// Элементы диаграммы (сообщения, фрагменты, заметки)
    pub elements: Vec<SequenceElement>,
    /// Box группировки участников
    pub boxes: Vec<ParticipantBox>,
}

impl SequenceDiagram {
    /// Создаёт новую пустую диаграмму
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет участника
    pub fn add_participant(&mut self, participant: Participant) {
        self.participants.push(participant);
    }

    /// Добавляет элемент
    pub fn add_element(&mut self, element: SequenceElement) {
        self.elements.push(element);
    }

    /// Добавляет box группировку
    pub fn add_box(&mut self, participant_box: ParticipantBox) {
        self.boxes.push(participant_box);
    }
}

/// Тип участника диаграммы
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ParticipantType {
    #[default]
    Participant,
    Actor,
    Boundary,
    Control,
    Entity,
    Database,
    Collections,
    Queue,
}

impl ParticipantType {
    /// Парсит тип участника из строки
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "participant" => Some(Self::Participant),
            "actor" => Some(Self::Actor),
            "boundary" => Some(Self::Boundary),
            "control" => Some(Self::Control),
            "entity" => Some(Self::Entity),
            "database" => Some(Self::Database),
            "collections" => Some(Self::Collections),
            "queue" => Some(Self::Queue),
            _ => None,
        }
    }
}

/// Участник диаграммы последовательностей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// Идентификатор участника
    pub id: Identifier,
    /// Тип участника
    pub participant_type: ParticipantType,
    /// Стереотип
    pub stereotype: Option<Stereotype>,
    /// Цвет
    pub color: Option<Color>,
    /// Порядок (для явного указания позиции)
    pub order: Option<i32>,
}

impl Participant {
    /// Создаёт нового участника
    pub fn new(name: impl Into<String>, participant_type: ParticipantType) -> Self {
        Self {
            id: Identifier::new(name),
            participant_type,
            stereotype: None,
            color: None,
            order: None,
        }
    }

    /// Создаёт участника типа participant
    pub fn as_participant(name: impl Into<String>) -> Self {
        Self::new(name, ParticipantType::Participant)
    }

    /// Создаёт участника типа actor
    pub fn actor(name: impl Into<String>) -> Self {
        Self::new(name, ParticipantType::Actor)
    }
}

/// Элемент диаграммы последовательностей
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SequenceElement {
    /// Сообщение между участниками
    Message(Message),
    /// Фрагмент (alt, opt, loop, etc.)
    Fragment(Fragment),
    /// Заметка
    Note(Note),
    /// Активация участника
    Activation(Activation),
    /// Разделитель
    Divider(Divider),
    /// Задержка
    Delay(Delay),
    /// Пробел
    Space(u32),
    /// Ссылка (ref)
    Reference(Reference),
    /// Команда autonumber
    Autonumber(AutonumberCommand),
    /// Return (возврат к вызывающему)
    Return(Return),
}

/// Тип стрелки сообщения
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ArrowType {
    #[default]
    /// -> обычная стрелка
    Normal,
    /// ->> тонкая стрелка
    Thin,
    /// ->o стрелка с кружком
    Circle,
    /// ->x стрелка с крестом (потерянное сообщение)
    Cross,
    /// -\ верхняя половина
    HalfTop,
    /// -/ нижняя половина
    HalfBottom,
}

/// Сообщение между участниками
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Отправитель
    pub from: String,
    /// Получатель
    pub to: String,
    /// Текст сообщения
    pub label: String,
    /// Стиль линии
    pub line_style: LineStyle,
    /// Тип стрелки
    pub arrow_type: ArrowType,
    /// Направление стрелки (true = влево)
    pub arrow_left: bool,
    /// Цвет стрелки
    pub color: Option<Color>,
    /// Активировать получателя
    pub activate: bool,
    /// Деактивировать отправителя
    pub deactivate: bool,
    /// Создать нового участника
    pub create: bool,
    /// Уничтожить участника
    pub destroy: bool,
}

impl Message {
    /// Создаёт новое сообщение
    pub fn new(from: impl Into<String>, to: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: label.into(),
            line_style: LineStyle::Solid,
            arrow_type: ArrowType::Normal,
            arrow_left: false,
            color: None,
            activate: false,
            deactivate: false,
            create: false,
            destroy: false,
        }
    }

    /// Устанавливает пунктирную линию (return message)
    pub fn dashed(mut self) -> Self {
        self.line_style = LineStyle::Dashed;
        self
    }

    /// Устанавливает активацию получателя
    pub fn with_activation(mut self) -> Self {
        self.activate = true;
        self
    }
}

/// Тип фрагмента
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FragmentType {
    /// alt - альтернативные ветки
    Alt,
    /// opt - опциональный блок
    Opt,
    /// loop - цикл
    Loop,
    /// par - параллельные ветки
    Par,
    /// break - прерывание
    Break,
    /// critical - критическая секция
    Critical,
    /// group - произвольная группа
    Group,
    /// ref - ссылка на другую диаграмму
    Ref,
}

impl FragmentType {
    /// Парсит тип фрагмента из строки
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "alt" => Some(Self::Alt),
            "opt" => Some(Self::Opt),
            "loop" => Some(Self::Loop),
            "par" => Some(Self::Par),
            "break" => Some(Self::Break),
            "critical" => Some(Self::Critical),
            "group" => Some(Self::Group),
            "ref" => Some(Self::Ref),
            _ => None,
        }
    }
}

/// Фрагмент (combined fragment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fragment {
    /// Тип фрагмента
    pub fragment_type: FragmentType,
    /// Условие/заголовок
    pub condition: Option<String>,
    /// Секции фрагмента (для alt/par)
    pub sections: Vec<FragmentSection>,
}

impl Fragment {
    /// Создаёт новый фрагмент
    pub fn new(fragment_type: FragmentType) -> Self {
        Self {
            fragment_type,
            condition: None,
            sections: vec![FragmentSection::default()],
        }
    }

    /// Устанавливает условие
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }
}

/// Секция фрагмента (для alt: then/else)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FragmentSection {
    /// Условие секции (для else ветки)
    pub condition: Option<String>,
    /// Элементы секции
    pub elements: Vec<SequenceElement>,
}

/// Активация участника
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activation {
    /// Имя участника
    pub participant: String,
    /// Тип активации
    pub activation_type: ActivationType,
    /// Цвет
    pub color: Option<Color>,
}

/// Тип активации
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivationType {
    Activate,
    Deactivate,
    Destroy,
}

/// Разделитель (== text ==)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divider {
    /// Текст разделителя
    pub text: String,
}

/// Задержка (...)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delay {
    /// Текст задержки (опционально)
    pub text: Option<String>,
}

/// Ссылка на другую диаграмму
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    /// Текст ссылки
    pub text: String,
    /// Участники, охваченные ссылкой
    pub participants: Vec<String>,
}

/// Box группировка участников
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantBox {
    /// Название box (отображается в заголовке)
    pub title: Option<String>,
    /// Цвет фона
    pub color: Option<Color>,
    /// Участники внутри box
    pub participants: Vec<String>,
}

/// Команда autonumber в диаграмме
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutonumberCommand {
    /// autonumber [start] [step] [format]
    Start(AutonumberStart),
    /// autonumber stop
    Stop,
    /// autonumber resume [start] [step] [format]
    Resume(Option<AutonumberStart>),
    /// autonumber inc <level>
    Inc(String),
}

/// Параметры запуска autonumber
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutonumberStart {
    /// Начальное значение (по умолчанию 1)
    pub start: Option<u32>,
    /// Шаг инкремента (по умолчанию 1)
    pub step: Option<u32>,
    /// Формат отображения (например "[00]", "<b>[000]</b>")
    pub format: Option<String>,
}

impl AutonumberStart {
    /// Создаёт параметры с указанным начальным значением
    pub fn with_start(start: u32) -> Self {
        Self {
            start: Some(start),
            step: None,
            format: None,
        }
    }

    /// Создаёт параметры с форматом
    pub fn with_format(format: impl Into<String>) -> Self {
        Self {
            start: None,
            step: None,
            format: Some(format.into()),
        }
    }

    /// Создаёт параметры со всеми значениями
    pub fn new(start: Option<u32>, step: Option<u32>, format: Option<String>) -> Self {
        Self { start, step, format }
    }
}

/// Return statement - возврат к вызывающему участнику
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Return {
    /// Текст возвращаемого сообщения (опционально)
    pub label: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_participant() {
        let alice = Participant::as_participant("Alice");
        assert_eq!(alice.id.name, "Alice");
        assert_eq!(alice.participant_type, ParticipantType::Participant);

        let bob = Participant::actor("Bob");
        assert_eq!(bob.participant_type, ParticipantType::Actor);
    }

    #[test]
    fn test_create_message() {
        let msg = Message::new("Alice", "Bob", "Hello");
        assert_eq!(msg.from, "Alice");
        assert_eq!(msg.to, "Bob");
        assert_eq!(msg.label, "Hello");
        assert_eq!(msg.line_style, LineStyle::Solid);

        let return_msg = Message::new("Bob", "Alice", "Hi").dashed();
        assert_eq!(return_msg.line_style, LineStyle::Dashed);
    }

    #[test]
    fn test_create_diagram() {
        let mut diagram = SequenceDiagram::new();
        diagram.add_participant(Participant::as_participant("Alice"));
        diagram.add_participant(Participant::as_participant("Bob"));
        diagram.add_element(SequenceElement::Message(Message::new(
            "Alice", "Bob", "Hello",
        )));

        assert_eq!(diagram.participants.len(), 2);
        assert_eq!(diagram.elements.len(), 1);
    }
}
