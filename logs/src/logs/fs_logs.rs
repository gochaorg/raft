/// Описывает где искать логи
struct FsLogFind {
    /// Шаблон искомого файла
    wildcard: String,

    /// Корень поиска
    root: String,

    /// Рекурсивный поиск
    recursive: bool,
}

