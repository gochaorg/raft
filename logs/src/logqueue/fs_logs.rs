/// Описывает где искать логи
#[allow(dead_code)]
struct FsLogFind {
    /// Шаблон искомого файла
    wildcard: String,

    /// Корень поиска
    root: String,

    /// Рекурсивный поиск
    recursive: bool,
}

