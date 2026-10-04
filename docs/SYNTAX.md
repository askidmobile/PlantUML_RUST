# Синтаксис PlantUML — Справочник

Этот документ описывает синтаксис PlantUML, который должен поддерживаться библиотекой plantuml-rs.

---

## Общая структура

Каждая диаграмма начинается с `@startuml` и заканчивается `@enduml`:

```plantuml
@startuml
' Здесь содержимое диаграммы
@enduml
```

### Комментарии

```plantuml
' Однострочный комментарий

/' Многострочный
   комментарий '/
```

---

## 1. Sequence Diagram

### Участники

```plantuml
@startuml
participant Alice
actor Bob
boundary Controller
control Logic
entity Database
database MySQL
collections Queue
queue MessageQueue
@enduml
```

### Сообщения

```plantuml
@startuml
Alice -> Bob: Синхронное сообщение
Alice --> Bob: Пунктирная линия
Alice ->> Bob: Тонкая стрелка
Alice -->> Bob: Пунктирная тонкая
Alice -\ Bob: Половина стрелки
Alice -/ Bob: Половина стрелки (нижняя)
Alice ->x Bob: Крест (потерянное)
Alice -[#red]> Bob: Цветная стрелка
@enduml
```

### Self-сообщения

```plantuml
@startuml
Alice -> Alice: Сообщение себе
@enduml
```

### Активации

```plantuml
@startuml
Alice -> Bob: Запрос
activate Bob
Bob -> Charlie: Подзапрос
activate Charlie
Charlie --> Bob: Ответ
deactivate Charlie
Bob --> Alice: Ответ
deactivate Bob
@enduml
```

Короткая форма:
```plantuml
@startuml
Alice -> Bob++: Запрос
Bob -> Charlie++: Подзапрос
Charlie --> Bob--: Ответ
Bob --> Alice--: Ответ
@enduml
```

### Фрагменты

```plantuml
@startuml
Alice -> Bob: Запрос

alt Успех
    Bob --> Alice: OK
else Ошибка
    Bob --> Alice: Error
end

opt Опционально
    Alice -> Bob: Дополнительно
end

loop 1000 раз
    Alice -> Bob: Ping
end

par Параллельно
    Alice -> Bob: Msg1
else
    Alice -> Charlie: Msg2
end

break Прервать
    Alice -> Bob: Stop
end

critical Критическая секция
    Alice -> Bob: Lock
end

group Группа [описание]
    Alice -> Bob: Grouped
end
@enduml
```

### Заметки

```plantuml
@startuml
Alice -> Bob: Hello
note left: Заметка слева
note right: Заметка справа
note over Alice: Заметка над Alice
note over Alice, Bob: Заметка над обоими
@enduml
```

### Разделители

```plantuml
@startuml
== Инициализация ==
Alice -> Bob: Init
== Основная логика ==
Alice -> Bob: Process
@enduml
```

---

## 2. Class Diagram

### Классы

```plantuml
@startuml
class User {
    - id: Long
    - name: String
    + getId(): Long
    + setName(name: String): void
}

abstract class AbstractUser {
    {abstract} + validate(): boolean
}

interface Comparable<T> {
    + compareTo(other: T): int
}

enum Status {
    ACTIVE
    INACTIVE
    DELETED
}
@enduml
```

### Модификаторы видимости

- `-` private
- `#` protected
- `~` package private
- `+` public

### Статические и абстрактные члены

```plantuml
@startuml
class Example {
    {static} + instance: Example
    {abstract} + process(): void
}
@enduml
```

### Отношения

```plantuml
@startuml
Class01 <|-- Class02 : Наследование
Class03 *-- Class04 : Композиция
Class05 o-- Class06 : Агрегация
Class07 <-- Class08 : Ассоциация
Class09 -- Class10 : Связь
Class11 <.. Class12 : Зависимость
Class13 ..|> Class14 : Реализация
Class15 .. Class16 : Пунктир
@enduml
```

### Множественность

```plantuml
@startuml
Company "1" *-- "many" Employee
@enduml
```

### Пакеты

```plantuml
@startuml
package "Domain" {
    class User
    class Order
}

package "Infrastructure" {
    class UserRepository
}

User <-- UserRepository
@enduml
```

---

## 3. Activity Diagram (новый синтаксис)

### Основные элементы

```plantuml
@startuml
start
:Первый шаг;
:Второй шаг;
stop
@enduml
```

### Условия

```plantuml
@startuml
start
if (Условие?) then (да)
    :Действие A;
else (нет)
    :Действие B;
endif
stop
@enduml
```

### Циклы

```plantuml
@startuml
start
while (Условие?)
    :Действие;
endwhile
stop
@enduml

@startuml
start
repeat
    :Действие;
repeat while (Продолжить?)
stop
@enduml
```

### Параллельные ветки

```plantuml
@startuml
start
fork
    :Ветка 1;
fork again
    :Ветка 2;
end fork
stop
@enduml
```

### Swim lanes

```plantuml
@startuml
|Пользователь|
start
:Заполнить форму;
|Система|
:Валидировать;
|База данных|
:Сохранить;
stop
@enduml
```

---

## 4. State Diagram

### Состояния

```plantuml
@startuml
[*] --> State1
State1 --> State2 : Событие
State2 --> [*]
@enduml
```

### Вложенные состояния

```plantuml
@startuml
state Active {
    [*] --> SubState1
    SubState1 --> SubState2
    SubState2 --> [*]
}

[*] --> Active
Active --> [*]
@enduml
```

### Параллельные состояния

```plantuml
@startuml
state Parallel {
    state A {
        [*] --> A1
    }
    --
    state B {
        [*] --> B1
    }
}
@enduml
```

---

## 5. Component Diagram

```plantuml
@startuml
package "Frontend" {
    [Web App]
    [Mobile App]
}

package "Backend" {
    [API Gateway]
    [User Service]
    [Order Service]
}

database "PostgreSQL" {
}

[Web App] --> [API Gateway]
[Mobile App] --> [API Gateway]
[API Gateway] --> [User Service]
[API Gateway] --> [Order Service]
[User Service] --> [PostgreSQL]
[Order Service] --> [PostgreSQL]
@enduml
```

---

## 6. Use Case Diagram

```plantuml
@startuml
left to right direction

actor User
actor Admin

rectangle System {
    usecase "Войти" as UC1
    usecase "Просмотр" as UC2
    usecase "Редактировать" as UC3
}

User --> UC1
User --> UC2
Admin --> UC3
Admin --> UC1
@enduml
```

---

## 7. Препроцессор

### Переменные

```plantuml
@startuml
!$name = "Alice"
!$color = "#FF0000"

participant $name
@enduml
```

### Условия

```plantuml
@startuml
!$debug = %true()

!if $debug
    note: Debug mode
!endif
@enduml
```

### Include

```plantuml
@startuml
!include common.puml
!include_once styles.puml
!include <stdlib/aws>
@enduml
```

### Функции

```plantuml
@startuml
!function $double($x)
    !return $x * 2
!endfunction

!$result = $double(5)
@enduml
```

### Builtin функции

| Функция | Описание |
|---------|----------|
| `%date()` | Текущая дата |
| `%time()` | Текущее время |
| `%version()` | Версия PlantUML |
| `%filename()` | Имя файла |
| `%dirpath()` | Путь к директории |
| `%true()` | Логическое true |
| `%false()` | Логическое false |
| `%not($x)` | Логическое NOT |
| `%strlen($s)` | Длина строки |
| `%substr($s, $start, $len)` | Подстрока |
| `%upper($s)` | В верхний регистр |
| `%lower($s)` | В нижний регистр |

---

## 8. Skinparam

```plantuml
@startuml
skinparam backgroundColor #EEEEEE
skinparam handwritten true
skinparam sequenceArrowThickness 2
skinparam roundcorner 20
skinparam maxmessagesize 60

skinparam sequence {
    ArrowColor DeepSkyBlue
    ActorBorderColor DeepSkyBlue
    LifeLineBorderColor blue
    LifeLineBackgroundColor #A9DCDF
    
    ParticipantBorderColor DeepSkyBlue
    ParticipantBackgroundColor DodgerBlue
    ParticipantFontName Impact
    ParticipantFontSize 17
    ParticipantFontColor #A9DCDF
}

Alice -> Bob: Hello
@enduml
```

---

## 9. Темы

```plantuml
@startuml
!theme cerulean
@enduml
```

Встроенные темы (`plantuml-themes`):
- `default` — тема по умолчанию
- `classic` (алиас `plantuml`) — классическая палитра PlantUML
- `minimal`
- `dark`
- `sketchy` (алиас `sketchy-outline`)
- `cerulean`

Полный список доступен программно: `plantuml_core::available_themes()`.

> **Ограничение:** `!theme` и `skinparam` разбираются препроцессором, но пока
> не доходят до рендерера — тема не влияет на вывод. См. `docs/AUDIT.md`, §3.2.

---

## Ссылки

- [PlantUML Language Reference](https://plantuml.com/guide)
- [PlantUML Sequence Diagram](https://plantuml.com/sequence-diagram)
- [PlantUML Class Diagram](https://plantuml.com/class-diagram)
- [PlantUML Activity Diagram](https://plantuml.com/activity-diagram-beta)
- [PlantUML State Diagram](https://plantuml.com/state-diagram)
