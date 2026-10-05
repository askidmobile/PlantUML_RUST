//! Жёсткое ограничение памяти приложения.
//!
//! # Зачем
//!
//! Препроцессор исполняет макросы PlantUML, и ошибка в них способна
//! привести к неограниченному росту. Реальный случай: в стандартной
//! библиотеке C4 значение переменной удваивалось на каждом вызове и
//! доходило до мегабайт, а разбор — до гигабайтов. Процесс выедал всю
//! память машины, и его приходилось убивать вручную.
//!
//! # Почему аллокатор, а не `ulimit`
//!
//! На macOS штатные механизмы НЕ РАБОТАЮТ. Проверено на этой машине:
//!
//! * `ulimit -v` и `ulimit -d` — ядро не поддерживает, команда молча
//!   игнорируется;
//! * `taskpolicy -m 200` — лимит игнорируется: процесс выделил 3.9 ГБ
//!   при заявленных 200 МиБ;
//! * `launchctl limit` — задаёт только системные значения, не на процесс;
//! * cgroups — механизм Linux, на macOS отсутствует.
//!
//! Работает единственный способ: считать память внутри процесса и
//! ОТКАЗЫВАТЬ в выделении сверх предела. Это и делает [`LimitedAllocator`].
//!
//! # Как это действует
//!
//! Аллокатор ведёт учёт запрошенных байтов. Когда суммарный объём
//! превышает предел, выделение возвращает null. Rust в этом случае
//! вызывает `handle_alloc_error`, то есть процесс аварийно завершается
//! с понятной диагностикой — вместо того чтобы исчерпать память машины.
//!
//! Для кода, который хочет обработать нехватку мягко, подходит
//! `Vec::try_reserve`: он возвращает `Err`, а не паникует.
//!
//! # Предел
//!
//! По умолчанию — 1 ГиБ. Изменить: [`set_memory_limit`].
//! Отключить: [`set_memory_limit`] с `usize::MAX`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Предел памяти по умолчанию: 1 ГиБ.
pub const DEFAULT_MEMORY_LIMIT: usize = 1024 * 1024 * 1024;

/// Сколько байтов сейчас выдано приложению.
static USED: AtomicUsize = AtomicUsize::new(0);

/// Предел, выше которого выделение не выполняется.
static LIMIT: AtomicUsize = AtomicUsize::new(DEFAULT_MEMORY_LIMIT);

/// Аллокатор с жёстким пределом памяти.
///
/// Устанавливается как `#[global_allocator]`, поэтому действует на всё
/// приложение целиком, включая зависимости.
pub struct LimitedAllocator;

/// Пытается зарезервировать `size` байтов.
///
/// Возвращает `false`, если резерв превысил бы предел. Сравнение и
/// запись идут в цикле CAS: при многопоточной сборке два потока не
/// должны проскочить проверку одновременно.
fn reserve(size: usize) -> bool {
    let limit = LIMIT.load(Ordering::Relaxed);
    let mut current = USED.load(Ordering::Relaxed);

    loop {
        if current.saturating_add(size) > limit {
            return false;
        }

        match USED.compare_exchange_weak(
            current,
            current + size,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return true,
            Err(actual) => current = actual,
        }
    }
}

/// Возвращает ранее зарезервированные байты.
fn release(size: usize) {
    // Насыщающее вычитание: разбалансировка учёта не должна приводить
    // к панике в аллокаторе — это сделало бы отладку невозможной.
    let _ = USED.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_sub(size))
    });
}

unsafe impl GlobalAlloc for LimitedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !reserve(layout.size()) {
            return std::ptr::null_mut();
        }

        let pointer = unsafe { System.alloc(layout) };
        if pointer.is_null() {
            release(layout.size());
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if !reserve(layout.size()) {
            return std::ptr::null_mut();
        }

        let pointer = unsafe { System.alloc_zeroed(layout) };
        if pointer.is_null() {
            release(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        release(layout.size());
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_size = layout.size();

        if new_size > old_size {
            if !reserve(new_size - old_size) {
                return std::ptr::null_mut();
            }
        } else {
            release(old_size - new_size);
        }

        let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
        if new_pointer.is_null() {
            // Перераспределение не удалось — возвращаем учёт в исходное
            // состояние, иначе память «утечёт» в счётчике.
            if new_size > old_size {
                release(new_size - old_size);
            } else {
                let _ = reserve(old_size - new_size);
            }
        }
        new_pointer
    }
}

/// Текущий предел памяти в байтах.
pub fn memory_limit() -> usize {
    LIMIT.load(Ordering::Relaxed)
}

/// Задаёт предел памяти в байтах.
///
/// `usize::MAX` отключает ограничение. Значение ниже уже занятого
/// объёма не освобождает память, но запрещает дальнейшие выделения.
pub fn set_memory_limit(bytes: usize) {
    LIMIT.store(bytes, Ordering::Relaxed);
}

/// Сколько байтов сейчас занято приложением.
pub fn memory_used() -> usize {
    USED.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Превышение предела даёт ошибку выделения, а не рост памяти.
    ///
    /// `try_reserve` выбран намеренно: он возвращает `Err` вместо
    /// аварийного завершения, поэтому проверка безопасна для тестов.
    #[test]
    fn test_limit_rejects_oversized_allocation() {
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
        let before = memory_used();

        {
            let buffer = vec![0u8; 4 * 1024 * 1024];
            assert!(memory_used() > before, "выделение не учтено");
            drop(buffer);
        }

        // Освобождение асинхронно по отношению к другим потокам, поэтому
        // допускаем небольшой остаток, но не рост на объём буфера.
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
}
