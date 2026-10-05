//! Тесты жёсткого предела памяти приложения.
//!
//! Вынесены в отдельный интеграционный бинарник намеренно: счётчики
//! аллокатора глобальные, а `cargo test` запускает тесты одного бинарника
//! параллельно в потоках. Внутри `plantuml-core` проверка «память
//! вернулась в счётчик» падала из-за аллокаций соседних тестов, а не
//! из-за ошибки в аллокаторе. Здесь же, кроме этих четырёх тестов, нет
//! никого, а сами они сериализованы мьютексом.

use std::sync::{Mutex, MutexGuard, OnceLock};

use plantuml_core::{memory_limit, memory_used, set_memory_limit, DEFAULT_MEMORY_LIMIT};

/// Сериализует тесты: они меняют общий предел и читают общий счётчик.
fn lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Превышение предела даёт ошибку выделения, а не рост памяти.
///
/// `try_reserve` выбран намеренно: он возвращает `Err` вместо
/// аварийного завершения, поэтому проверка безопасна для тестов.
#[test]
fn test_limit_rejects_oversized_allocation() {
    let _guard = lock();
    let saved = memory_limit();
    set_memory_limit(64 * 1024 * 1024);

    let mut buffer: Vec<u8> = Vec::new();
    let result = buffer.try_reserve(512 * 1024 * 1024);

    set_memory_limit(saved);

    assert!(
        result.is_err(),
        "выделение сверх предела должно быть отклонено"
    );
}

/// В пределах лимита память выделяется как обычно.
#[test]
fn test_allocation_within_limit_succeeds() {
    let _guard = lock();
    let saved = memory_limit();
    set_memory_limit(64 * 1024 * 1024);

    let mut buffer: Vec<u8> = Vec::new();
    let result = buffer.try_reserve(1024 * 1024);

    set_memory_limit(saved);

    assert!(
        result.is_ok(),
        "выделение в пределах лимита должно проходить"
    );
}

/// После освобождения память возвращается в счётчик.
#[test]
fn test_freed_memory_is_accounted() {
    let _guard = lock();
    let before = memory_used();

    {
        let buffer = vec![0u8; 4 * 1024 * 1024];
        assert!(memory_used() > before, "выделение не учтено");
        drop(buffer);
    }

    assert!(
        memory_used() < before + 1024 * 1024,
        "память не вернулась в счётчик: {} против {}",
        memory_used(),
        before
    );
}

/// Предел по умолчанию — 1 ГиБ.
#[test]
fn test_default_limit_is_one_gib() {
    assert_eq!(DEFAULT_MEMORY_LIMIT, 1024 * 1024 * 1024);
}
