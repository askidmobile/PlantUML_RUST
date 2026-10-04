//! Общие определения и утилиты для стандартной библиотеки
//!
//! Базовые спрайты, цвета и вспомогательные макросы.

use std::collections::HashMap;

/// Регистрирует общие включения
pub fn register(registry: &mut HashMap<&'static str, &'static str>) {
    // Общие определения
    registry.insert("common", COMMON);
    registry.insert("common.puml", COMMON);

    // Цвета
    registry.insert("colors", COLORS);
    registry.insert("colors.puml", COLORS);

    // Базовые спрайты
    registry.insert("sprites/common", SPRITES_COMMON);
}

/// Общие определения и утилиты
const COMMON: &str = r#"' common.puml
' Общие определения для PlantUML

' Версия стандартной библиотеки
!define STDLIB_VERSION "0.2.0"

' Вспомогательные макросы
!define SHOW_LEGEND() legend right \n Легенда \n endlegend
!define HIDE_LEGEND() hide legend

' Направление layout
!define LAYOUT_TOP_DOWN() top to bottom direction
!define LAYOUT_LEFT_RIGHT() left to right direction
!define LAYOUT_LANDSCAPE() left to right direction

' Стиль "от руки"
!define LAYOUT_AS_SKETCH() skinparam handwritten true

' Скрытие/показ элементов
!define HIDE_STEREOTYPE() hide stereotype
!define SHOW_STEREOTYPE() show stereotype

' Отладочный вывод
!define DEBUG(text) note "text" as DEBUG_NOTE

' Условные включения
!define IF_DEF(var) !ifdef var
!define ELSE_DEF() !else
!define END_DEF() !endif
"#;

/// Определения цветов
const COLORS: &str = r#"' colors.puml
' Стандартные цвета PlantUML

' Основные цвета
!define COLOR_PRIMARY #1168BD
!define COLOR_SECONDARY #438DD5
!define COLOR_ACCENT #85BBF0

' Серые оттенки
!define COLOR_GRAY_DARK #333333
!define COLOR_GRAY #666666
!define COLOR_GRAY_LIGHT #999999
!define COLOR_GRAY_LIGHTER #CCCCCC

' Семантические цвета
!define COLOR_SUCCESS #28A745
!define COLOR_WARNING #FFC107
!define COLOR_DANGER #DC3545
!define COLOR_INFO #17A2B8

' Цвета для UML элементов
!define COLOR_ACTOR #08427B
!define COLOR_SYSTEM #1168BD
!define COLOR_EXTERNAL #999999
!define COLOR_DATABASE #438DD5
!define COLOR_BOUNDARY #444444

' Цвета C4 Model
!define C4_PERSON_COLOR #08427B
!define C4_SYSTEM_COLOR #1168BD
!define C4_CONTAINER_COLOR #438DD5
!define C4_COMPONENT_COLOR #85BBF0
!define C4_EXTERNAL_COLOR #999999
"#;

/// Базовые спрайты
const SPRITES_COMMON: &str = r#"' sprites/common.puml
' Базовые спрайты

' Спрайт информации (i в круге)
sprite $info [16x16/4] {
0000000000000000
0000011111000000
0000111111100000
0001111111110000
0011110001111000
0011110001111000
0011111111111000
0011111111111000
0011110001111000
0011110001111000
0011110001111000
0001111111110000
0000111111100000
0000011111000000
0000000000000000
0000000000000000
}

' Спрайт предупреждения (! в треугольнике)  
sprite $warning [16x16/4] {
0000000000000000
0000000110000000
0000001111000000
0000011111100000
0000111111110000
0001111001111000
0011110000111100
0111100000011110
1111100000011111
1111000110001111
1110001111000111
1110000110000111
0111000000001110
0011111111111100
0001111111111000
0000000000000000
}

' Спрайт ошибки (X в круге)
sprite $error [16x16/4] {
0000011111000000
0001111111110000
0011111111111000
0111100000111100
1111001100011110
1110011110001110
1100111111000110
1001111111100010
1001111111100010
1100111111000110
1110011110001110
1111001100011110
0111100000111100
0011111111111000
0001111111110000
0000011111000000
}

' Спрайт успеха (галочка в круге)
sprite $success [16x16/4] {
0000011111000000
0001111111110000
0011111111111000
0111111111111100
1111111111111110
1111111110011110
1111111100001110
1111111000001110
1100011000011110
1110000000111110
1111000001111110
0111100011111100
0011111111111000
0001111111110000
0000011111000000
0000000000000000
}
"#;
