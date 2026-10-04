//! Файловый FileResolver для поддержки !include
//!
//! Реализует загрузку файлов с файловой системы с поддержкой:
//! - Абсолютных путей
//! - Относительных путей (относительно базовой директории)
//! - Путей поиска (search paths)
//! - Стандартной библиотеки PlantUML (`<stdlib/...>`)

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{FileResolver, PreprocessError, Result};

/// FileResolver для файловой системы
///
/// Поддерживает загрузку файлов для директив `!include` и `!include_once`.
///
/// # Пример
///
/// ```rust,ignore
/// use plantuml_preprocessor::{Preprocessor, FsFileResolver};
///
/// let resolver = FsFileResolver::new("/path/to/project");
/// let preprocessor = Preprocessor::with_resolver(resolver);
/// let result = preprocessor.process(source)?;
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FsFileResolver {
    /// Базовая директория для относительных путей
    base_dir: PathBuf,
    /// Дополнительные пути поиска
    search_paths: Vec<PathBuf>,
    /// Максимальная глубина включений (защита от рекурсии)
    max_depth: usize,
    /// Текущая глубина включений
    current_depth: usize,
    /// Уже включённые файлы (для обнаружения циклов)
    included_files: HashSet<PathBuf>,
}

impl FsFileResolver {
    /// Создаёт новый FsFileResolver с указанной базовой директорией
    ///
    /// # Аргументы
    ///
    /// * `base_dir` - базовая директория для относительных путей
    ///
    /// # Пример
    ///
    /// ```rust,ignore
    /// let resolver = FsFileResolver::new("/home/user/project");
    /// ```
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            search_paths: Vec::new(),
            max_depth: 10,
            current_depth: 0,
            included_files: HashSet::new(),
        }
    }

    /// Создаёт FsFileResolver с текущей директорией как базовой
    pub fn current_dir() -> std::io::Result<Self> {
        Ok(Self::new(std::env::current_dir()?))
    }

    /// Добавляет путь поиска
    ///
    /// Пути поиска проверяются после базовой директории
    /// при разрешении относительных путей.
    pub fn with_search_path(mut self, path: impl AsRef<Path>) -> Self {
        self.search_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Добавляет несколько путей поиска
    pub fn with_search_paths(mut self, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        self.search_paths.extend(paths);
        self
    }

    /// Устанавливает максимальную глубину включений
    ///
    /// По умолчанию: 10
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Устанавливает базовую директорию
    pub fn with_base_dir(mut self, base_dir: impl AsRef<Path>) -> Self {
        self.base_dir = base_dir.as_ref().to_path_buf();
        self
    }

    /// Разрешает путь к файлу
    ///
    /// Порядок поиска:
    /// 1. Абсолютный путь (если путь абсолютный)
    /// 2. Относительно base_dir
    /// 3. В каждом из search_paths
    /// 4. В стандартной библиотеке (для путей вида `<...>`)
    fn resolve_path(&self, path: &str) -> Option<PathBuf> {
        let path_str = path.trim();

        // Обработка стандартной библиотеки: <stdlib/aws/...>
        if path_str.starts_with('<') && path_str.ends_with('>') {
            return self.resolve_stdlib_path(&path_str[1..path_str.len() - 1]);
        }

        // Убираем кавычки если есть
        let path_str = path_str.trim_matches('"');
        let file_path = Path::new(path_str);

        // 1. Абсолютный путь
        if file_path.is_absolute() && file_path.exists() {
            return Some(file_path.to_path_buf());
        }

        // 2. Относительно base_dir
        let relative_path = self.base_dir.join(file_path);
        if relative_path.exists() {
            return Some(relative_path);
        }

        // 3. В search_paths
        for search_path in &self.search_paths {
            let candidate = search_path.join(file_path);
            if candidate.exists() {
                return Some(candidate);
            }
        }

        None
    }

    /// Разрешает путь к стандартной библиотеке
    fn resolve_stdlib_path(&self, stdlib_path: &str) -> Option<PathBuf> {
        // Сначала проверяем встроенную стандартную библиотеку
        if plantuml_stdlib::exists(stdlib_path) {
            // Возвращаем специальный маркер для stdlib
            // (будет обработан в read_file)
            return Some(PathBuf::from(format!("__stdlib__:{}", stdlib_path)));
        }

        // Fallback: ищем в search_paths
        for search_path in &self.search_paths {
            let candidate = search_path.join(stdlib_path);
            if candidate.exists() {
                return Some(candidate);
            }

            // Попробуем с расширением .puml
            let with_ext = search_path.join(format!("{}.puml", stdlib_path));
            if with_ext.exists() {
                return Some(with_ext);
            }

            // Попробуем с расширением .iuml
            let with_ext = search_path.join(format!("{}.iuml", stdlib_path));
            if with_ext.exists() {
                return Some(with_ext);
            }
        }

        None
    }

    /// Проверяет на циклическое включение
    #[allow(dead_code)]
    fn check_recursion(&mut self, path: &Path) -> Result<()> {
        // Проверка глубины
        if self.current_depth >= self.max_depth {
            return Err(PreprocessError::RecursiveInclude(format!(
                "превышена максимальная глубина включений ({})",
                self.max_depth
            )));
        }

        // Канонический путь для сравнения
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // Проверка на уже включённый файл (цикл)
        if self.included_files.contains(&canonical) {
            return Err(PreprocessError::RecursiveInclude(format!(
                "{}",
                path.display()
            )));
        }

        self.included_files.insert(canonical);
        self.current_depth += 1;

        Ok(())
    }

    /// Сбрасывает состояние после включения файла
    #[allow(dead_code)]
    fn finish_include(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
    }
}

impl FileResolver for FsFileResolver {
    fn read_file(&self, path: &str) -> Result<String> {
        // Разрешаем путь
        let resolved_path = self.resolve_path(path).ok_or_else(|| {
            PreprocessError::FileNotFound(format!(
                "{} (base_dir: {})",
                path,
                self.base_dir.display()
            ))
        })?;

        // Проверяем, является ли это встроенным stdlib
        let path_str = resolved_path.to_string_lossy();
        if let Some(stdlib_path) = path_str.strip_prefix("__stdlib__:") {
            return plantuml_stdlib::get_include(stdlib_path)
                .map(|s| s.to_string())
                .ok_or_else(|| PreprocessError::FileNotFound(format!("stdlib: {}", stdlib_path)));
        }

        // Читаем файл с файловой системы
        fs::read_to_string(&resolved_path).map_err(|e| {
            PreprocessError::FileReadError(format!("{}: {}", resolved_path.display(), e))
        })
    }

    fn file_exists(&self, path: &str) -> bool {
        self.resolve_path(path).is_some()
    }
}

/// Создаёт FsFileResolver из пути к файлу (использует родительскую директорию)
impl From<&Path> for FsFileResolver {
    fn from(file_path: &Path) -> Self {
        let base_dir = file_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        Self::new(base_dir)
    }
}

impl From<PathBuf> for FsFileResolver {
    fn from(file_path: PathBuf) -> Self {
        Self::from(file_path.as_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_resolve_relative_path() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(temp_dir.path(), "common.puml", "' common definitions");

        let resolver = FsFileResolver::new(temp_dir.path());

        assert!(resolver.file_exists("common.puml"));
        assert!(!resolver.file_exists("nonexistent.puml"));
    }

    #[test]
    fn test_read_file() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(temp_dir.path(), "test.puml", "Alice -> Bob: Hello");

        let resolver = FsFileResolver::new(temp_dir.path());
        let content = resolver.read_file("test.puml").unwrap();

        assert_eq!(content, "Alice -> Bob: Hello");
    }

    #[test]
    fn test_search_paths() {
        let temp_dir = TempDir::new().unwrap();
        let lib_dir = temp_dir.path().join("lib");
        fs::create_dir_all(&lib_dir).unwrap();
        create_test_file(&lib_dir, "shared.puml", "' shared content");

        let resolver = FsFileResolver::new(temp_dir.path()).with_search_path(&lib_dir);

        assert!(resolver.file_exists("shared.puml"));
        let content = resolver.read_file("shared.puml").unwrap();
        assert_eq!(content, "' shared content");
    }

    #[test]
    fn test_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(
            temp_dir.path(),
            "includes/common/styles.puml",
            "skinparam backgroundColor white",
        );

        let resolver = FsFileResolver::new(temp_dir.path());

        assert!(resolver.file_exists("includes/common/styles.puml"));
    }

    #[test]
    fn test_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = FsFileResolver::new(temp_dir.path());

        let result = resolver.read_file("nonexistent.puml");
        assert!(matches!(result, Err(PreprocessError::FileNotFound(_))));
    }

    #[test]
    fn test_quoted_path() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(temp_dir.path(), "my file.puml", "content");

        let resolver = FsFileResolver::new(temp_dir.path());

        assert!(resolver.file_exists("\"my file.puml\""));
    }

    #[test]
    fn test_from_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("diagrams/main.puml");
        create_test_file(temp_dir.path(), "diagrams/main.puml", "main");
        create_test_file(temp_dir.path(), "diagrams/include.puml", "include");

        let resolver = FsFileResolver::from(file_path.as_path());

        // base_dir должен быть "diagrams/"
        assert!(resolver.file_exists("include.puml"));
    }

    #[test]
    fn test_stdlib_include() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = FsFileResolver::new(temp_dir.path());

        // Проверяем, что stdlib включения доступны
        assert!(resolver.file_exists("<C4/C4_Context>"));

        // Читаем содержимое
        let content = resolver.read_file("<C4/C4_Context>").unwrap();
        assert!(content.contains("C4_CONTEXT"));
        assert!(content.contains("Person"));
    }

    #[test]
    fn test_stdlib_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = FsFileResolver::new(temp_dir.path());

        // Несуществующий stdlib путь
        assert!(!resolver.file_exists("<nonexistent/path>"));
    }
}
