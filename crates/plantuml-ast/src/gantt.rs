//! AST типы для Gantt Diagrams (диаграммы Ганта).
//!
//! Gantt Diagram показывает задачи проекта на временной шкале.

use serde::{Deserialize, Serialize};

use crate::common::{Color, DiagramMetadata};

/// Диаграмма Ганта
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GanttDiagram {
    /// Метаданные диаграммы
    pub metadata: DiagramMetadata,
    /// Дата начала проекта
    pub project_start: Option<GanttDate>,
    /// Масштаб (daily, weekly, monthly)
    pub scale: GanttScale,
    /// Задачи
    pub tasks: Vec<GanttTask>,
    /// Разделители (separators)
    pub separators: Vec<GanttSeparator>,
    /// Вехи (milestones)
    pub milestones: Vec<GanttMilestone>,
    /// Выходные дни
    pub closed_days: Vec<ClosedDay>,
    /// Праздники
    pub holidays: Vec<Holiday>,
}

impl GanttDiagram {
    /// Создаёт новую пустую диаграмму
    pub fn new() -> Self {
        Self::default()
    }

    /// Добавляет задачу
    pub fn add_task(&mut self, task: GanttTask) {
        self.tasks.push(task);
    }

    /// Добавляет веху
    pub fn add_milestone(&mut self, milestone: GanttMilestone) {
        self.milestones.push(milestone);
    }
}

/// Масштаб диаграммы Ганта
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GanttScale {
    /// Ежедневный масштаб
    #[default]
    Daily,
    /// Еженедельный масштаб
    Weekly,
    /// Ежемесячный масштаб
    Monthly,
    /// Ежеквартальный масштаб
    Quarterly,
    /// Ежегодный масштаб
    Yearly,
}

impl GanttScale {
    /// Парсит масштаб из строки
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "daily" => Some(Self::Daily),
            "weekly" => Some(Self::Weekly),
            "monthly" => Some(Self::Monthly),
            "quarterly" => Some(Self::Quarterly),
            "yearly" => Some(Self::Yearly),
            _ => None,
        }
    }
}

/// Дата в формате Gantt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl GanttDate {
    /// Создаёт новую дату
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Парсит дату из строки формата "YYYY-MM-DD" или "YYYY/MM/DD"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(['-', '/']).collect();
        if parts.len() == 3 {
            let year = parts[0].parse().ok()?;
            let month = parts[1].parse().ok()?;
            let day = parts[2].parse().ok()?;
            Some(Self { year, month, day })
        } else {
            None
        }
    }

}

impl std::fmt::Display for GanttDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Задача в диаграмме Ганта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttTask {
    /// Идентификатор задачи (для ссылок)
    pub id: Option<String>,
    /// Название задачи
    pub name: String,
    /// Алиас задачи
    pub alias: Option<String>,
    /// Начало задачи
    pub start: TaskStart,
    /// Продолжительность или дата окончания
    pub duration: TaskDuration,
    /// Процент выполнения (0-100)
    pub complete: Option<u8>,
    /// Цвет задачи
    pub color: Option<Color>,
    /// Ресурс (исполнитель)
    pub resource: Option<String>,
    /// Ссылки на другие задачи (зависимости)
    pub links: Vec<String>,
    /// Задача активна/неактивна
    pub is_active: bool,
}

impl GanttTask {
    /// Создаёт новую задачу
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            alias: None,
            start: TaskStart::AfterPrevious,
            duration: TaskDuration::Days(1),
            complete: None,
            color: None,
            resource: None,
            links: Vec::new(),
            is_active: true,
        }
    }

    /// Устанавливает ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Устанавливает продолжительность в днях
    pub fn lasts_days(mut self, days: u32) -> Self {
        self.duration = TaskDuration::Days(days);
        self
    }

    /// Устанавливает процент выполнения
    pub fn with_complete(mut self, percent: u8) -> Self {
        self.complete = Some(percent.min(100));
        self
    }

    /// Добавляет зависимость
    pub fn starts_after(mut self, task_id: impl Into<String>) -> Self {
        self.start = TaskStart::After(task_id.into());
        self
    }
}

/// Начало задачи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStart {
    /// Начинается сразу после предыдущей задачи
    AfterPrevious,
    /// Начинается после указанной задачи
    After(String),
    /// Начинается с указанной даты
    AtDate(GanttDate),
    /// Начинается одновременно с указанной задачей
    With(String),
    /// Начинается в конце указанной задачи
    AtEnd(String),
}

/// Продолжительность задачи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskDuration {
    /// Количество дней
    Days(u32),
    /// Количество недель
    Weeks(u32),
    /// До указанной даты
    Until(GanttDate),
    /// До конца указанной задачи
    EndsAt(String),
}

impl Default for TaskDuration {
    fn default() -> Self {
        Self::Days(1)
    }
}

/// Разделитель (горизонтальная линия между группами задач)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttSeparator {
    /// Метка разделителя
    pub label: Option<String>,
}

impl GanttSeparator {
    /// Создаёт новый разделитель
    pub fn new() -> Self {
        Self { label: None }
    }

    /// Создаёт разделитель с меткой
    pub fn with_label(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
        }
    }
}

impl Default for GanttSeparator {
    fn default() -> Self {
        Self::new()
    }
}

/// Веха (milestone) — важная точка в проекте
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttMilestone {
    /// Название вехи
    pub name: String,
    /// Когда наступает веха
    pub happens: MilestoneTime,
    /// Цвет
    pub color: Option<Color>,
}

impl GanttMilestone {
    /// Создаёт новую веху
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            happens: MilestoneTime::AfterPrevious,
            color: None,
        }
    }

    /// Веха после указанной задачи
    pub fn after(name: impl Into<String>, task: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            happens: MilestoneTime::After(task.into()),
            color: None,
        }
    }
}

/// Время наступления вехи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneTime {
    /// После предыдущей задачи
    AfterPrevious,
    /// После указанной задачи
    After(String),
    /// В указанную дату
    AtDate(GanttDate),
}

/// Выходной день недели
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedDay {
    /// День недели (saturday, sunday, etc.)
    pub day: Weekday,
}

/// День недели
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    /// Парсит день недели из строки
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "monday" | "mon" => Some(Self::Monday),
            "tuesday" | "tue" => Some(Self::Tuesday),
            "wednesday" | "wed" => Some(Self::Wednesday),
            "thursday" | "thu" => Some(Self::Thursday),
            "friday" | "fri" => Some(Self::Friday),
            "saturday" | "sat" => Some(Self::Saturday),
            "sunday" | "sun" => Some(Self::Sunday),
            _ => None,
        }
    }
}

/// Праздничный день
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holiday {
    /// Дата праздника
    pub date: GanttDate,
    /// Название праздника
    pub name: Option<String>,
}

impl Holiday {
    /// Создаёт праздник
    pub fn new(date: GanttDate) -> Self {
        Self { date, name: None }
    }

    /// Создаёт праздник с названием
    pub fn with_name(date: GanttDate, name: impl Into<String>) -> Self {
        Self {
            date,
            name: Some(name.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task() {
        let task = GanttTask::new("Design")
            .with_id("T1")
            .lasts_days(5)
            .with_complete(50);

        assert_eq!(task.name, "Design");
        assert_eq!(task.id, Some("T1".to_string()));
        assert!(matches!(task.duration, TaskDuration::Days(5)));
        assert_eq!(task.complete, Some(50));
    }

    #[test]
    fn test_parse_date() {
        let date = GanttDate::parse("2024-01-15").unwrap();
        assert_eq!(date.year, 2024);
        assert_eq!(date.month, 1);
        assert_eq!(date.day, 15);

        let date2 = GanttDate::parse("2024/06/30").unwrap();
        assert_eq!(date2.month, 6);
    }

    #[test]
    fn test_task_dependencies() {
        let task = GanttTask::new("Implementation")
            .starts_after("T1");

        assert!(matches!(task.start, TaskStart::After(ref id) if id == "T1"));
    }

    #[test]
    fn test_weekday_parse() {
        assert_eq!(Weekday::parse("saturday"), Some(Weekday::Saturday));
        assert_eq!(Weekday::parse("Sun"), Some(Weekday::Sunday));
        assert_eq!(Weekday::parse("Mon"), Some(Weekday::Monday));
    }

    #[test]
    fn test_scale_parse() {
        assert_eq!(GanttScale::parse("daily"), Some(GanttScale::Daily));
        assert_eq!(GanttScale::parse("Weekly"), Some(GanttScale::Weekly));
        assert_eq!(GanttScale::parse("monthly"), Some(GanttScale::Monthly));
    }
}
