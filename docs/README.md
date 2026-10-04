# Документация plantuml-rs

| Документ | Назначение |
|----------|------------|
| [AUDIT.md](AUDIT.md) | **Аудит проекта и план изменений.** Полный разбор состояния: критические дефекты, измеренные расхождения с PlantUML, план по 7 фазам |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Архитектура: крейты, pipeline, layout-движки, feature-флаги |
| [PLAN.md](PLAN.md) | Исходный план разработки проекта |
| [SEQUENCE_LAYOUT_ALGORITHM.md](SEQUENCE_LAYOUT_ALGORITHM.md) | Алгоритм расчёта горизонтального расположения участников sequence-диаграмм |
| [SYNTAX.md](SYNTAX.md) | Справочник по поддерживаемому синтаксису |
| [PLANTUML_LANGUAGE_REFERENCE.md](PLANTUML_LANGUAGE_REFERENCE.md) | Справочник по языку PlantUML |

## Сверка с оригинальным PlantUML

Методика и текущий уровень расхождений: [../tests/golden/README.md](../tests/golden/README.md).

## PlantUML.pdf

Полный справочник PlantUML в PDF (10.5 МБ, 610 страниц) **не хранится в
репозитории**: он составлял 88% трекаемого контента. Скачать при
необходимости:

```bash
curl -o docs/PlantUML.pdf \
  https://plantuml.com/PlantUML_Language_Reference_Guide.pdf
```

Онлайн-версия: <https://plantuml.com/guide>
