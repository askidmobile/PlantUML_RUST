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

/// Состояние autonumber
#[derive(Debug, Clone, Default)]
pub struct AutonumberState {
    /// Включена ли автонумерация
    pub enabled: bool,
    /// Текущий номер
    pub current: u32,
    /// Шаг нумерации
    pub step: u32,
    /// Формат нумерации (например: "[00]", "<b>[0]</b>")
    pub format: Option<String>,
}

impl AutonumberState {
    /// Возвращает следующий номер и увеличивает счётчик
    pub fn next(&mut self) -> String {
        let num = self.current;
        self.current += self.step;

        if let Some(fmt) = &self.format {
            // Простая замена: 0 -> число, 00 -> с ведущими нулями
            if fmt.contains("00") {
                fmt.replace("00", &format!("{:02}", num))
            } else if fmt.contains('0') {
                fmt.replace('0', &num.to_string())
            } else {
                // Если формат не содержит 0, просто добавляем номер в начало
                format!("{}{}", num, fmt)
            }
        } else {
            num.to_string()
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
