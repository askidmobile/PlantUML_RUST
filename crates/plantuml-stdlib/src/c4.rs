//! C4 Model — библиотека для архитектурных диаграмм
//!
//! Реализация C4 Model (Context, Container, Component, Code) для PlantUML.
//! Основано на <https://github.com/plantuml-stdlib/C4-PlantUML>

use std::collections::HashMap;

/// Регистрирует все C4 включения в реестре
pub fn register(registry: &mut HashMap<&'static str, &'static str>) {
    // C4_Context — диаграммы контекста системы
    registry.insert("C4/C4_Context", C4_CONTEXT);
    registry.insert("C4/C4_Context.puml", C4_CONTEXT);

    // C4_Container — диаграммы контейнеров
    registry.insert("C4/C4_Container", C4_CONTAINER);
    registry.insert("C4/C4_Container.puml", C4_CONTAINER);

    // C4_Component — диаграммы компонентов
    registry.insert("C4/C4_Component", C4_COMPONENT);
    registry.insert("C4/C4_Component.puml", C4_COMPONENT);

    // C4_Dynamic — динамические диаграммы
    registry.insert("C4/C4_Dynamic", C4_DYNAMIC);
    registry.insert("C4/C4_Dynamic.puml", C4_DYNAMIC);

    // C4_Deployment — диаграммы развёртывания
    registry.insert("C4/C4_Deployment", C4_DEPLOYMENT);
    registry.insert("C4/C4_Deployment.puml", C4_DEPLOYMENT);
}

/// C4_Context.puml — макросы для контекстных диаграмм
const C4_CONTEXT: &str = r#"' C4_Context.puml
' C4 Model - System Context diagram macros
' Based on https://c4model.com/

!define C4_CONTEXT

' Цвета по умолчанию
!define PERSON_BG_COLOR #08427B
!define PERSON_BORDER_COLOR #073B6F
!define SYSTEM_BG_COLOR #1168BD
!define SYSTEM_BORDER_COLOR #0B4884
!define EXTERNAL_SYSTEM_BG_COLOR #999999
!define EXTERNAL_SYSTEM_BORDER_COLOR #8A8A8A
!define EXTERNAL_PERSON_BG_COLOR #999999
!define EXTERNAL_PERSON_BORDER_COLOR #8A8A8A

' Настройки внешнего вида
skinparam rectangle {
    StereotypeFontColor #FFFFFF
    FontColor #FFFFFF
    BackgroundColor SYSTEM_BG_COLOR
    BorderColor SYSTEM_BORDER_COLOR
    roundCorner 8
}

' ===== МАКРОСЫ ЭЛЕМЕНТОВ =====

' Person - пользователь системы
!define Person(e_alias, e_label) rectangle "==e_label\n<size:12>[Person]</size>" <<person>> as e_alias
!define Person(e_alias, e_label, e_descr) rectangle "==e_label\n<size:12>[Person]</size>\n\ne_descr" <<person>> as e_alias

' Person_Ext - внешний пользователь
!define Person_Ext(e_alias, e_label) rectangle "==e_label\n<size:12>[External Person]</size>" <<external_person>> as e_alias
!define Person_Ext(e_alias, e_label, e_descr) rectangle "==e_label\n<size:12>[External Person]</size>\n\ne_descr" <<external_person>> as e_alias

' System - внутренняя система
!define System(e_alias, e_label) rectangle "==e_label\n<size:12>[Software System]</size>" <<system>> as e_alias
!define System(e_alias, e_label, e_descr) rectangle "==e_label\n<size:12>[Software System]</size>\n\ne_descr" <<system>> as e_alias

' System_Ext - внешняя система
!define System_Ext(e_alias, e_label) rectangle "==e_label\n<size:12>[External System]</size>" <<external_system>> as e_alias
!define System_Ext(e_alias, e_label, e_descr) rectangle "==e_label\n<size:12>[External System]</size>\n\ne_descr" <<external_system>> as e_alias

' System_Boundary - граница системы
!define System_Boundary(e_alias, e_label) rectangle "e_label" <<boundary>> as e_alias {

' Enterprise_Boundary - граница предприятия
!define Enterprise_Boundary(e_alias, e_label) rectangle "e_label" <<enterprise>> as e_alias {

' ===== СВЯЗИ =====

' Rel - связь между элементами
!define Rel(e_from, e_to, e_label) e_from --> e_to : e_label
!define Rel(e_from, e_to, e_label, e_techn) e_from --> e_to : e_label\n<size:10>[e_techn]</size>

' Rel_Back - обратная связь
!define Rel_Back(e_from, e_to, e_label) e_from <-- e_to : e_label

' Rel_Neighbor - связь рядом
!define Rel_Neighbor(e_from, e_to, e_label) e_from -> e_to : e_label

' BiRel - двунаправленная связь
!define BiRel(e_from, e_to, e_label) e_from <--> e_to : e_label
!define BiRel(e_from, e_to, e_label, e_techn) e_from <--> e_to : e_label\n<size:10>[e_techn]</size>

' ===== НАПРАВЛЕННЫЕ СВЯЗИ =====
'
' Версия C4-PlantUML умеет задавать направление стрелки: Rel_D (вниз),
' Rel_U (вверх), Rel_L (влево), Rel_R (вправо) и их длинные синонимы.
' В нашей библиотеке их не было — не хватало 17 макросов.
' Синтаксис направлений `-down->` поддерживается грамматикой component.

!define Rel_D(e_from, e_to, e_label) e_from -down-> e_to : e_label
!define Rel_D(e_from, e_to, e_label, e_techn) e_from -down-> e_to : e_label\n<size:10>[e_techn]</size>
!define Rel_Down(e_from, e_to, e_label) Rel_D(e_from, e_to, e_label)
!define Rel_Down(e_from, e_to, e_label, e_techn) Rel_D(e_from, e_to, e_label, e_techn)

!define Rel_U(e_from, e_to, e_label) e_from -up-> e_to : e_label
!define Rel_U(e_from, e_to, e_label, e_techn) e_from -up-> e_to : e_label\n<size:10>[e_techn]</size>
!define Rel_Up(e_from, e_to, e_label) Rel_U(e_from, e_to, e_label)
!define Rel_Up(e_from, e_to, e_label, e_techn) Rel_U(e_from, e_to, e_label, e_techn)

!define Rel_L(e_from, e_to, e_label) e_from -left-> e_to : e_label
!define Rel_L(e_from, e_to, e_label, e_techn) e_from -left-> e_to : e_label\n<size:10>[e_techn]</size>
!define Rel_Left(e_from, e_to, e_label) Rel_L(e_from, e_to, e_label)
!define Rel_Left(e_from, e_to, e_label, e_techn) Rel_L(e_from, e_to, e_label, e_techn)

!define Rel_R(e_from, e_to, e_label) e_from -right-> e_to : e_label
!define Rel_R(e_from, e_to, e_label, e_techn) e_from -right-> e_to : e_label\n<size:10>[e_techn]</size>
!define Rel_Right(e_from, e_to, e_label) Rel_R(e_from, e_to, e_label)
!define Rel_Right(e_from, e_to, e_label, e_techn) Rel_R(e_from, e_to, e_label, e_techn)

!define Rel_Back_Neighbor(e_from, e_to, e_label) e_from <- e_to : e_label

!define BiRel_D(e_from, e_to, e_label) e_from <->down-> e_to : e_label
!define BiRel_U(e_from, e_to, e_label) e_from <->up-> e_to : e_label
!define BiRel_L(e_from, e_to, e_label) e_from <->left-> e_to : e_label
!define BiRel_R(e_from, e_to, e_label) e_from <->right-> e_to : e_label
!define BiRel_Neighbor(e_from, e_to, e_label) e_from <-> e_to : e_label
!define BiRel_Back_Neighbor(e_from, e_to, e_label) e_from <-> e_to : e_label

' Boundary - граница с произвольным типом
!define Boundary(e_alias, e_label, e_type) rectangle "e_label" <<e_type>> as e_alias {

' Lay_* — псевдонимы для указания направления раскладки
!define Lay_D(e_from, e_to) e_from -down-> e_to
!define Lay_U(e_from, e_to) e_from -up-> e_to
!define Lay_L(e_from, e_to) e_from -left-> e_to
!define Lay_R(e_from, e_to) e_from -right-> e_to

' Теги элементов и связей
!define AddElementTag(e_tag, e_bgColor, e_fontColor, e_borderColor) skinparam rectangle<<e_tag>> { \n BackgroundColor e_bgColor \n FontColor e_fontColor \n BorderColor e_borderColor \n }
!define AddRelTag(e_tag, e_color, e_lineStyle, e_textColor) skinparam arrow<<e_tag>> { \n Color e_color \n }

' ===== СТИЛИ =====

skinparam rectangle<<person>> {
    BackgroundColor PERSON_BG_COLOR
    BorderColor PERSON_BORDER_COLOR
}

skinparam rectangle<<external_person>> {
    BackgroundColor EXTERNAL_PERSON_BG_COLOR
    BorderColor EXTERNAL_PERSON_BORDER_COLOR
}

skinparam rectangle<<system>> {
    BackgroundColor SYSTEM_BG_COLOR
    BorderColor SYSTEM_BORDER_COLOR
}

skinparam rectangle<<external_system>> {
    BackgroundColor EXTERNAL_SYSTEM_BG_COLOR
    BorderColor EXTERNAL_SYSTEM_BORDER_COLOR
}

skinparam rectangle<<boundary>> {
    BackgroundColor #FFFFFF
    BorderColor #444444
    BorderStyle dashed
    FontColor #444444
}

skinparam rectangle<<enterprise>> {
    BackgroundColor #FFFFFF
    BorderColor #444444
    BorderStyle dashed
    FontColor #444444
}

' Layout helpers
!define LAYOUT_TOP_DOWN top to bottom direction
!define LAYOUT_LEFT_RIGHT left to right direction
!define LAYOUT_AS_SKETCH skinparam handwritten true
"#;

/// C4_Container.puml — макросы для диаграмм контейнеров
const C4_CONTAINER: &str = r#"' C4_Container.puml
' C4 Model - Container diagram macros

!define C4_CONTAINER
!include <C4/C4_Context>

' Дополнительные цвета
!define CONTAINER_BG_COLOR #438DD5
!define CONTAINER_BORDER_COLOR #3C7FC0
!define DATABASE_BG_COLOR #438DD5
!define DATABASE_BORDER_COLOR #3C7FC0

' Container - контейнер (приложение, сервис, БД)
!define Container(e_alias, e_label, e_techn) rectangle "==e_label\n<size:12>[Container: e_techn]</size>" <<container>> as e_alias
!define Container(e_alias, e_label, e_techn, e_descr) rectangle "==e_label\n<size:12>[Container: e_techn]</size>\n\ne_descr" <<container>> as e_alias

' ContainerDb - база данных
!define ContainerDb(e_alias, e_label, e_techn) database "==e_label\n<size:12>[Database: e_techn]</size>" <<database>> as e_alias
!define ContainerDb(e_alias, e_label, e_techn, e_descr) database "==e_label\n<size:12>[Database: e_techn]</size>\n\ne_descr" <<database>> as e_alias

' ContainerQueue - очередь сообщений
!define ContainerQueue(e_alias, e_label, e_techn) queue "==e_label\n<size:12>[Queue: e_techn]</size>" <<queue>> as e_alias
!define ContainerQueue(e_alias, e_label, e_techn, e_descr) queue "==e_label\n<size:12>[Queue: e_techn]</size>\n\ne_descr" <<queue>> as e_alias

' Container_Ext - внешний контейнер
!define Container_Ext(e_alias, e_label, e_techn) rectangle "==e_label\n<size:12>[External Container: e_techn]</size>" <<external_container>> as e_alias

' Container_Boundary - граница контейнера
!define Container_Boundary(e_alias, e_label) rectangle "e_label" <<container_boundary>> as e_alias {

skinparam rectangle<<container>> {
    BackgroundColor CONTAINER_BG_COLOR
    BorderColor CONTAINER_BORDER_COLOR
    FontColor #FFFFFF
}

skinparam database<<database>> {
    BackgroundColor DATABASE_BG_COLOR
    BorderColor DATABASE_BORDER_COLOR
    FontColor #FFFFFF
}

skinparam rectangle<<external_container>> {
    BackgroundColor EXTERNAL_SYSTEM_BG_COLOR
    BorderColor EXTERNAL_SYSTEM_BORDER_COLOR
    FontColor #FFFFFF
}

skinparam rectangle<<container_boundary>> {
    BackgroundColor #FFFFFF
    BorderColor #444444
    BorderStyle dashed
    FontColor #444444
}
"#;

/// C4_Component.puml — макросы для диаграмм компонентов
const C4_COMPONENT: &str = r#"' C4_Component.puml
' C4 Model - Component diagram macros

!define C4_COMPONENT
!include <C4/C4_Container>

' Цвета компонентов
!define COMPONENT_BG_COLOR #85BBF0
!define COMPONENT_BORDER_COLOR #78A8D8

' Component - компонент внутри контейнера
!define Component(e_alias, e_label, e_techn) rectangle "==e_label\n<size:12>[Component: e_techn]</size>" <<component>> as e_alias
!define Component(e_alias, e_label, e_techn, e_descr) rectangle "==e_label\n<size:12>[Component: e_techn]</size>\n\ne_descr" <<component>> as e_alias

' Component_Ext - внешний компонент
!define Component_Ext(e_alias, e_label, e_techn) rectangle "==e_label\n<size:12>[External Component: e_techn]</size>" <<external_component>> as e_alias

' ComponentDb - компонент базы данных
!define ComponentDb(e_alias, e_label, e_techn) database "==e_label\n<size:12>[Component: e_techn]</size>" <<component_db>> as e_alias

' ComponentQueue - компонент очереди
!define ComponentQueue(e_alias, e_label, e_techn) queue "==e_label\n<size:12>[Component: e_techn]</size>" <<component_queue>> as e_alias

skinparam rectangle<<component>> {
    BackgroundColor COMPONENT_BG_COLOR
    BorderColor COMPONENT_BORDER_COLOR
    FontColor #000000
}

skinparam rectangle<<external_component>> {
    BackgroundColor #CCCCCC
    BorderColor #AAAAAA
    FontColor #000000
}

skinparam database<<component_db>> {
    BackgroundColor COMPONENT_BG_COLOR
    BorderColor COMPONENT_BORDER_COLOR
    FontColor #000000
}
"#;

/// C4_Dynamic.puml — макросы для динамических диаграмм
const C4_DYNAMIC: &str = r#"' C4_Dynamic.puml
' C4 Model - Dynamic diagram macros (sequence-like)

!define C4_DYNAMIC
!include <C4/C4_Component>

' Для динамических диаграмм используем sequence diagram
' с C4 стилизацией

' RelIndex - связь с номером шага
!define RelIndex(e_index, e_from, e_to, e_label) e_from -> e_to : e_index. e_label
!define RelIndex(e_index, e_from, e_to, e_label, e_techn) e_from -> e_to : e_index. e_label\n<size:10>[e_techn]</size>

' Increment - вспомогательная функция для нумерации
!define INCREMENT(e_counter) !$e_counter = $e_counter + 1
"#;

/// C4_Deployment.puml — макросы для диаграмм развёртывания
const C4_DEPLOYMENT: &str = r#"' C4_Deployment.puml
' C4 Model - Deployment diagram macros

!define C4_DEPLOYMENT
!include <C4/C4_Container>

' Цвета узлов развёртывания
!define NODE_BG_COLOR #FFFFFF
!define NODE_BORDER_COLOR #444444

' Deployment_Node - узел развёртывания (сервер, VM, контейнер)
!define Deployment_Node(e_alias, e_label) node "e_label" <<deployment_node>> as e_alias {
!define Deployment_Node(e_alias, e_label, e_type) node "e_label\n<size:10>[e_type]</size>" <<deployment_node>> as e_alias {
!define Deployment_Node(e_alias, e_label, e_type, e_descr) node "e_label\n<size:10>[e_type]</size>\n\ne_descr" <<deployment_node>> as e_alias {

' Deployment_Node_L - узел развёртывания (левый)
!define Deployment_Node_L(e_alias, e_label, e_type) node "e_label\n<size:10>[e_type]</size>" <<deployment_node>> as e_alias {

' Deployment_Node_R - узел развёртывания (правый)  
!define Deployment_Node_R(e_alias, e_label, e_type) node "e_label\n<size:10>[e_type]</size>" <<deployment_node>> as e_alias {

' Node - альтернативное имя для Deployment_Node
!define Node(e_alias, e_label) Deployment_Node(e_alias, e_label)
!define Node(e_alias, e_label, e_type) Deployment_Node(e_alias, e_label, e_type)

skinparam node<<deployment_node>> {
    BackgroundColor NODE_BG_COLOR
    BorderColor NODE_BORDER_COLOR
    FontColor #000000
}
"#;
