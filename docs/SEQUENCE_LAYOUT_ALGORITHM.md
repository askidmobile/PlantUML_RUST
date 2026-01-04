# Алгоритм расчёта горизонтального расположения участников в Sequence Diagram

> **Статус**: Реализовано в `crates/plantuml-layout/src/sequence/engine.rs`

## Проблема

При расчёте расстояний между участниками (spacing) нужно учитывать ширину текста сообщений. Но сообщения могут идти:
1. Между **соседними** участниками (A → B)
2. Между **несоседними** участниками (A → C, где между ними есть B)

Для случая 2 текст располагается на всю длину стрелки и НЕ должен влиять на расстояние между промежуточными участниками.

**Критически важно**: Для корректного расчёта необходимо знать РЕАЛЬНЫЕ ширины участников, а не приблизительные значения. Ширины вычисляются ДО расчёта spacing.

## Модель данных

```
Участники (слева направо): P0, P1, P2, P3, ...
Spacing между соседними: S[0] = расстояние P0-P1, S[1] = P1-P2, ...

Сообщение от Pi к Pj (i < j):
- Если j = i+1: сообщение между СОСЕДНИМИ
- Если j > i+1: сообщение через (j-i-1) промежуточных участников
```

## Ключевые понятия

### Длина стрелки (arrow_length)

Длина стрелки от Pi до Pj (i < j):
```
arrow_length = sum(S[k] for k in range(i, j)) + sum(participant_widths[k] for k in range(i, j))
```

То есть: сумма всех spacing между участниками + сумма ширин промежуточных участников.

### Требуемая длина для текста (required_length)

```
required_length = text_width + left_padding + right_padding
```

Где:
- `text_width` — ширина текста сообщения (включая autonumber если есть)
- `left_padding` — отступ от начала стрелки до текста (~5px)
- `right_padding` — отступ от конца текста до наконечника стрелки (~15px)

## Алгоритм расчёта spacing

### Шаг 1: Инициализация

```
min_spacing = 50.0  # минимальное расстояние между соседними участниками
S[i] = min_spacing for all i
```

### Шаг 2: Сбор требований от сообщений

Для каждого сообщения от Pi до Pj:

**Случай A: Соседние участники (j = i + 1)**
```
required_length = text_width + 20  # padding
S[i] = max(S[i], required_length - avg_participant_width)
```

Здесь `avg_participant_width` — приблизительная ширина участника, т.к. стрелка идёт от центра до центра.

**Случай B: Несоседние участники (j > i + 1)**

Для стрелок через несколько участников текст использует ВСЮ длину стрелки.
Эти сообщения НЕ увеличивают spacing между промежуточными участниками!

Вместо этого, мы проверяем что СУММАРНАЯ длина достаточна:
```
current_arrow_length = sum(S[k] for k in range(i, j)) + intermediate_widths
if current_arrow_length < required_length:
    # Нужно увеличить ОБЩУЮ длину, но КАК?
```

### Шаг 3: Решение для несоседних участников

**Вариант 1: Увеличить только крайние сегменты**
```
deficit = required_length - current_arrow_length
# Распределяем deficit между S[i] и S[j-1] (первый и последний сегмент)
S[i] += deficit / 2
S[j-1] += deficit / 2
```

**Вариант 2: Увеличить равномерно все сегменты**
```
deficit = required_length - current_arrow_length
segments_count = j - i
per_segment = deficit / segments_count
for k in range(i, j):
    S[k] += per_segment
```

**Вариант 3 (PlantUML поведение): Не увеличивать промежуточные**

PlantUML НЕ увеличивает расстояния между промежуточными участниками.
Текст просто располагается на имеющемся пространстве.

Если текст не помещается — PlantUML увеличивает расстояние только между ПЕРВОЙ парой участников (от источника):
```
required_for_first_segment = required_length - (current_arrow_length - S[i])
S[i] = max(S[i], required_for_first_segment)
```

## Порядок обработки

**КРИТИЧЕСКИ ВАЖНО:** Сначала обрабатываются сообщения между СОСЕДНИМИ участниками, затем через 1 промежуточный, затем через 2, и т.д.

Это гарантирует что:
1. Базовые расстояния устанавливаются от прямых сообщений
2. Длинные сообщения используют уже установленные расстояния

```python
# Группируем сообщения по "расстоянию" (количеству сегментов)
messages_by_span = defaultdict(list)
for msg in messages:
    span = abs(msg.to_idx - msg.from_idx)
    messages_by_span[span].append(msg)

# Обрабатываем от коротких к длинным
for span in sorted(messages_by_span.keys()):
    for msg in messages_by_span[span]:
        process_message(msg)
```

## Пример

```
Участники: A, B, C, D
Сообщения:
  1. A → B: "Hello" (ширина 35px)
  2. A → C: "Long message text" (ширина 120px)
  3. B → C: "Hi" (ширина 14px)
  4. A → D: "Very long message across all" (ширина 200px)
```

**Шаг 1:** min_spacing = 50, S = [50, 50, 50]

**Шаг 2:** Обрабатываем span=1 (соседние):
- A → B: required = 35 + 20 = 55, S[0] = max(50, 55 - 40) = 50 (не меняется, т.к. 15 < 50)
- B → C: required = 14 + 20 = 34, S[1] = max(50, 34 - 40) = 50 (не меняется)

**Шаг 3:** Обрабатываем span=2 (через 1 участника):
- A → C: 
  - current_length = S[0] + S[1] + width(B) = 50 + 50 + 40 = 140px
  - required = 120 + 20 = 140px
  - 140 >= 140, OK — ничего не меняем

**Шаг 4:** Обрабатываем span=3 (через 2 участника):
- A → D:
  - current_length = S[0] + S[1] + S[2] + width(B) + width(C) = 50 + 50 + 50 + 40 + 40 = 230px
  - required = 200 + 20 = 220px
  - 230 >= 220, OK — ничего не меняем

## Итоговая формула

```rust
fn calculate_spacing(messages: &[Message], participants: &[Participant]) -> Vec<f64> {
    let n = participants.len();
    let mut spacing = vec![MIN_SPACING; n - 1];
    
    // Группируем по span
    let mut by_span: BTreeMap<usize, Vec<&Message>> = BTreeMap::new();
    for msg in messages {
        let span = (msg.to_idx as i32 - msg.from_idx as i32).abs() as usize;
        by_span.entry(span).or_default().push(msg);
    }
    
    // Обрабатываем от коротких к длинным
    for (span, msgs) in by_span {
        for msg in msgs {
            let (start, end) = if msg.from_idx < msg.to_idx {
                (msg.from_idx, msg.to_idx)
            } else {
                (msg.to_idx, msg.from_idx)
            };
            
            let required = msg.text_width + PADDING;
            
            if span == 1 {
                // Соседние: напрямую устанавливаем spacing
                // Вычитаем половины ширин участников (стрелка от центра до центра)
                let adjustment = (participants[start].width + participants[end].width) / 2.0;
                spacing[start] = spacing[start].max(required - adjustment);
            } else {
                // Несоседние: проверяем суммарную длину
                let current_length: f64 = (start..end).map(|i| spacing[i]).sum::<f64>()
                    + (start+1..end).map(|i| participants[i].width).sum::<f64>();
                
                if current_length < required {
                    // Нужно увеличить. Увеличиваем ПЕРВЫЙ сегмент.
                    let deficit = required - current_length;
                    spacing[start] += deficit;
                }
            }
        }
    }
    
    spacing
}
```

## Визуализация

```
      50px      50px      50px
   |--------|--------|--------|
   A   40   B   40   C   40   D
   |   px   |   px   |   px   |
   
Сообщение A → C:
   [====== текст ======]
   |------ 140px ------|
   A        B          C
   
Сообщение A → D:
   [======== очень длинный текст ========]
   |------------ 230px -----------------|
   A        B          C          D
```

## Отличие от текущей реализации

**Текущая (неправильная):**
```rust
let spacing_for_text = (total_width - 20.0) / pairs_count as f64;
for i in start..end {
    spacing_map[i] = max(spacing_map[i], spacing_for_text);
}
```
Проблема: делит ширину текста между ВСЕМИ сегментами, увеличивая промежуточные.

**Правильная:**
1. Для соседних (span=1): устанавливаем spacing напрямую
2. Для несоседних (span>1): проверяем суммарную длину, увеличиваем только если нужно, и только первый сегмент
