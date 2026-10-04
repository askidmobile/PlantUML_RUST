//! Метрики и позиции элементов sequence diagram

use super::SequenceLayoutConfig;
use indexmap::IndexMap;
use plantuml_model::Rect;

/// Информация о позиции участника
#[derive(Debug, Clone)]
pub struct ParticipantMetrics {
    /// ID участника (alias или имя, используется для идентификации)
    #[allow(dead_code)]
    pub id: String,
    /// Отображаемое имя участника (для header и footer)
    pub display_name: String,
    /// Центр X участника (для lifeline)
    pub center_x: f64,
    /// Ширина блока
    pub width: f64,
    /// Прямоугольник заголовка (для будущего использования)
    #[allow(dead_code)]
    pub header_bounds: Rect,
}

/// Информация об активации
#[derive(Debug, Clone)]
pub struct ActivationInfo {
    /// Участник
    pub participant: String,
    /// Y координата начала активации
    pub start_y: f64,
    /// Уровень вложенности (для смещения по X)
    pub level: u32,
}

/// Состояние autonumber (поддержка multi-level: 1.1.1)
#[derive(Debug, Clone)]
pub struct AutonumberState {
    /// Включена ли автонумерация
    pub enabled: bool,
    /// Уровни нумерации (например: [1, 2, 3] -> "1.2.3")
    pub levels: Vec<u32>,
    /// Шаг нумерации для последнего уровня
    pub step: u32,
    /// Формат нумерации (например: "[00]", "<b>[0]</b>")
    pub format: Option<String>,
}

impl Default for AutonumberState {
    fn default() -> Self {
        Self {
            enabled: false,
            levels: vec![1], // По умолчанию один уровень, начинаем с 1
            step: 1,
            format: None,
        }
    }
}

impl AutonumberState {
    /// Устанавливает начальное значение (может быть multi-level: "1.2.3" или простое число)
    pub fn set_start(&mut self, start: u32) {
        // Простое число устанавливает только первый уровень
        self.levels = vec![start];
    }

    /// Инкрементирует указанный уровень и сбрасывает нижние уровни
    /// level: 'A' = первый уровень, 'B' = второй, 'C' = третий, ...
    ///
    /// Если указан уровень, которого нет, автоматически добавляется ещё один уровень
    /// для создания multi-level нумерации. Например:
    /// `levels = [2]`, `inc A` → `levels = [3, 1]`
    pub fn increment_level(&mut self, level: char) {
        let level_idx = (level.to_ascii_uppercase() as usize).saturating_sub('A' as usize);

        // Если указан уровень A (первый), и у нас только один уровень,
        // добавляем второй уровень для multi-level нумерации
        if level_idx == 0 && self.levels.len() == 1 {
            self.levels.push(1);
        }

        // Расширяем вектор если нужно
        while self.levels.len() <= level_idx {
            self.levels.push(1);
        }

        // Инкрементируем указанный уровень
        self.levels[level_idx] += 1;

        // Сбрасываем все нижние уровни на 1
        for i in (level_idx + 1)..self.levels.len() {
            self.levels[i] = 1;
        }
    }

    /// Возвращает текущий номер и увеличивает последний уровень на step
    pub fn next(&mut self) -> String {
        // Формируем строку номера
        let num_str = self.format_number();

        // Инкрементируем последний уровень
        if let Some(last) = self.levels.last_mut() {
            *last += self.step;
        }

        // Применяем формат если есть
        if let Some(fmt) = &self.format {
            self.apply_format(fmt, &num_str)
        } else {
            num_str
        }
    }

    /// Форматирует текущий номер (например: "1.2.3")
    fn format_number(&self) -> String {
        self.levels
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Применяет формат к номеру
    fn apply_format(&self, fmt: &str, num_str: &str) -> String {
        // Простая замена: 0 -> число
        if fmt.contains("00") {
            // Для multi-level берём последний уровень для форматирования с нулями
            let last = self
                .levels
                .last()
                .copied()
                .unwrap_or(1)
                .saturating_sub(self.step);
            fmt.replace("00", &format!("{:02}", last))
        } else if fmt.contains('0') {
            fmt.replace('0', num_str)
        } else {
            // Если формат не содержит 0, добавляем номер в начало
            format!("{}{}", num_str, fmt)
        }
    }
}

/// Метрики всей диаграммы
#[derive(Debug, Clone)]
pub struct DiagramMetrics {
    /// Позиции участников (id -> metrics)
    pub participants: IndexMap<String, ParticipantMetrics>,
    /// Текущая Y позиция (для размещения следующего элемента)
    pub current_y: f64,
    /// Y позиция последнего сообщения (для активации после сообщения)
    pub last_message_y: f64,
    /// Максимальная X координата
    pub max_x: f64,
    /// Стек активаций (participant_id -> count)
    pub activation_stack: IndexMap<String, u32>,
    /// Активные активации (стек для каждого участника)
    pub active_activations: IndexMap<String, Vec<ActivationInfo>>,
    /// Завершённые активации (для отрисовки)
    pub completed_activations: Vec<(ActivationInfo, f64)>, // (info, end_y)
    /// Состояние autonumber
    pub autonumber: AutonumberState,
    /// Стек вызовов для return (caller, callee)
    pub call_stack: Vec<(String, String)>,
}

impl DiagramMetrics {
    /// Создаёт новые метрики
    pub fn new() -> Self {
        Self {
            participants: IndexMap::new(),
            current_y: 0.0,
            last_message_y: 0.0,
            max_x: 0.0,
            activation_stack: IndexMap::new(),
            active_activations: IndexMap::new(),
            completed_activations: Vec::new(),
            autonumber: AutonumberState::default(),
            call_stack: Vec::new(),
        }
    }

    /// Получает центр X участника по имени
    pub fn participant_center_x(&self, name: &str) -> Option<f64> {
        self.participants.get(name).map(|p| p.center_x)
    }

    /// Получает уровень активации участника
    pub fn activation_level(&self, name: &str) -> u32 {
        self.activation_stack.get(name).copied().unwrap_or(0)
    }

    /// Увеличивает уровень активации и запоминает начало
    /// Использует last_message_y — позицию последнего сообщения (как в PlantUML)
    pub fn activate(&mut self, name: &str) {
        // Активация начинается от последнего сообщения, а не от текущей позиции
        self.activate_at(name, self.last_message_y);
    }

    /// Увеличивает уровень активации и запоминает начало на указанной Y позиции
    pub fn activate_at(&mut self, name: &str, start_y: f64) {
        let level = self.activation_stack.entry(name.to_string()).or_insert(0);
        *level += 1;

        // Запоминаем начало активации
        let info = ActivationInfo {
            participant: name.to_string(),
            start_y,
            level: *level,
        };

        self.active_activations
            .entry(name.to_string())
            .or_default()
            .push(info);
    }

    /// Уменьшает уровень активации и сохраняет завершённую
    pub fn deactivate(&mut self, name: &str) {
        if let Some(level) = self.activation_stack.get_mut(name) {
            if *level > 0 {
                *level -= 1;

                // Извлекаем и сохраняем завершённую активацию
                if let Some(stack) = self.active_activations.get_mut(name) {
                    if let Some(info) = stack.pop() {
                        self.completed_activations.push((info, self.current_y));
                    }
                }
            }
        }
    }

    /// Завершает все активные активации (для конца диаграммы)
    pub fn finalize_activations(&mut self, end_y: f64) {
        for (_, stack) in self.active_activations.iter_mut() {
            while let Some(info) = stack.pop() {
                self.completed_activations.push((info, end_y));
            }
        }
        self.activation_stack.clear();
    }

    /// Продвигает Y позицию
    pub fn advance_y(&mut self, delta: f64) {
        self.current_y += delta;
    }

    /// Вычисляет X позицию на lifeline с учётом активации
    /// Возвращает правый край activation box (для исходящих стрелок)
    pub fn lifeline_x(&self, name: &str, config: &SequenceLayoutConfig) -> f64 {
        let center_x = self.participant_center_x(name).unwrap_or(0.0);
        let level = self.activation_level(name);

        if level > 0 {
            // Правый край activation box = center_x + activation_width/2
            // При вложенных активациях смещаем дополнительно
            let offset = (level as f64 - 1.0) * config.activation_width / 2.0;
            center_x + config.activation_width / 2.0 + offset
        } else {
            center_x
        }
    }

    /// Проверяет, есть ли участник
    pub fn has_participant(&self, name: &str) -> bool {
        self.participants.contains_key(name)
    }

    /// Находит или создаёт участника автоматически
    #[allow(dead_code)]
    pub fn ensure_participant(&mut self, name: &str, config: &SequenceLayoutConfig) {
        if !self.has_participant(name) {
            // Добавляем участника в конец
            let x = if self.participants.is_empty() {
                config.margin + config.participant_width / 2.0
            } else {
                self.max_x + config.participant_spacing + config.participant_width / 2.0
            };

            let width = config.participant_width_for_name(name);

            self.participants.insert(
                name.to_string(),
                ParticipantMetrics {
                    id: name.to_string(),
                    display_name: name.to_string(), // Для автоматических участников display_name = id
                    center_x: x,
                    width,
                    header_bounds: Rect::new(
                        x - width / 2.0,
                        config.margin,
                        width,
                        config.participant_height,
                    ),
                },
            );

            self.max_x = x + width / 2.0;
        }
    }
}

impl Default for DiagramMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autonumber_simple() {
        let mut state = AutonumberState {
            enabled: true,
            ..Default::default()
        };

        assert_eq!(state.next(), "1");
        assert_eq!(state.next(), "2");
        assert_eq!(state.next(), "3");
    }

    #[test]
    fn test_autonumber_with_step() {
        let mut state = AutonumberState {
            enabled: true,
            step: 5,
            ..Default::default()
        };

        assert_eq!(state.next(), "1");
        assert_eq!(state.next(), "6");
        assert_eq!(state.next(), "11");
    }

    #[test]
    fn test_autonumber_multilevel_basic() {
        let mut state = AutonumberState {
            enabled: true,
            levels: vec![1, 1], // Начинаем с 1.1
            ..Default::default()
        };

        assert_eq!(state.next(), "1.1");
        assert_eq!(state.next(), "1.2");
        assert_eq!(state.next(), "1.3");
    }

    #[test]
    fn test_autonumber_multilevel_three_levels() {
        let mut state = AutonumberState {
            enabled: true,
            levels: vec![1, 1, 1], // Начинаем с 1.1.1
            ..Default::default()
        };

        assert_eq!(state.next(), "1.1.1");
        assert_eq!(state.next(), "1.1.2");
        assert_eq!(state.next(), "1.1.3");
    }

    #[test]
    fn test_autonumber_increment_level_a() {
        let mut state = AutonumberState {
            enabled: true,
            levels: vec![1, 3], // 1.3
            ..Default::default()
        };

        state.increment_level('A'); // Инкремент первого уровня

        assert_eq!(state.levels, vec![2, 1]); // 2.1 (первый +1, второй сброшен)
    }

    #[test]
    fn test_autonumber_increment_level_b() {
        let mut state = AutonumberState {
            enabled: true,
            levels: vec![1, 1, 5], // 1.1.5
            ..Default::default()
        };

        state.increment_level('B'); // Инкремент второго уровня

        assert_eq!(state.levels, vec![1, 2, 1]); // 1.2.1 (второй +1, третий сброшен)
    }

    #[test]
    fn test_autonumber_full_scenario() {
        // Симуляция PlantUML сценария:
        // autonumber 1.1.1
        // Alice -> Bob: msg1   ' 1.1.1
        // Alice -> Bob: msg2   ' 1.1.2
        // autonumber inc A
        // Alice -> Bob: msg3   ' 2.1.1
        // autonumber inc B
        // Alice -> Bob: msg4   ' 2.2.1

        let mut state = AutonumberState {
            enabled: true,
            levels: vec![1, 1, 1],
            ..Default::default()
        };

        assert_eq!(state.next(), "1.1.1");
        assert_eq!(state.next(), "1.1.2");

        state.increment_level('A'); // autonumber inc A
        assert_eq!(state.next(), "2.1.1");

        state.increment_level('B'); // autonumber inc B
        assert_eq!(state.next(), "2.2.1");
    }

    #[test]
    fn test_autonumber_with_format() {
        let mut state = AutonumberState {
            enabled: true,
            format: Some("[0]".to_string()),
            ..Default::default()
        };

        assert_eq!(state.next(), "[1]");
        assert_eq!(state.next(), "[2]");
    }

    #[test]
    fn test_autonumber_set_start() {
        let mut state = AutonumberState::default();
        state.set_start(10);

        assert_eq!(state.levels, vec![10]);
    }

    #[test]
    fn test_autonumber_inc_a_creates_multilevel() {
        // Когда autonumber начинается с простого числа и вызывается inc A,
        // автоматически создаётся второй уровень для multi-level нумерации
        let mut state = AutonumberState {
            enabled: true,
            levels: vec![2], // Простое состояние после пары next()
            ..Default::default()
        };

        state.increment_level('A');

        // Должен добавить второй уровень и инкрементировать первый
        assert_eq!(state.levels, vec![3, 1]); // 3.1
    }

    #[test]
    fn test_autonumber_simple_then_inc() {
        // Реальный сценарий: autonumber, несколько сообщений, затем inc A
        let mut state = AutonumberState {
            enabled: true,
            ..Default::default()
        };

        assert_eq!(state.next(), "1"); // levels: [1] -> [2]
        assert_eq!(state.next(), "2"); // levels: [2] -> [3]

        // Теперь levels = [3]
        state.increment_level('A'); // autonumber inc A -> [4, 1]

        assert_eq!(state.next(), "4.1"); // levels: [4, 1] -> [4, 2]
        assert_eq!(state.next(), "4.2"); // levels: [4, 2] -> [4, 3]

        state.increment_level('A'); // autonumber inc A again -> [5, 1]

        assert_eq!(state.next(), "5.1");
    }
}
