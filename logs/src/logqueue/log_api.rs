use super::log_id::LogQueueFileId;

/// Навигация по смеженным записям
/// 
/// Типы
/// - `Err` - Ошибки
/// - `RecordId` - Идентификатор записи
pub trait LogNavigationNear<ERR,RecordId> {
    /// Получение id следующей записи
    fn next_record( &self, record_id: RecordId ) -> Result<Option<RecordId>,ERR>;

    /// Получение id предыдущей записи
    fn previous_record( &self, record_id: RecordId ) -> Result<Option<RecordId>,ERR>;    
}

/// Навигация в конец
pub trait LogNavigateLast<ERR, RecordId> {
    /// Получение последней записи в log queue
    fn last_record( &self ) -> Result<Option<RecordId>,ERR>;
}

/// Чтение отдельных записей
pub trait LogReading<ERR,RecordId,Record,RecordOptions> {
    /// Чтение записи и ее опций
    fn read_record( &self, record_id: RecordId ) -> Result<(Record,RecordOptions), ERR>;

    /// Чтение опций записи
    fn read_options( &self, record_id: RecordId ) -> Result<RecordOptions, ERR>;
}

/// Лог - хранит в себе сумму лог файлов [crate::logfile]
/// 
/// Типы
/// - `ERR` - Тип ошибки
/// - `RecordId` - Идентификатор записи
pub trait LogQueue<ERR,RecordId,LogId>: 
    LogNavigationNear<ERR,RecordId> + 
    LogNavigateLast<ERR,RecordId>
{
    /// Запись
    type Record : LogRecord<RecordOption = Self::RecordOption>;

    /// Опции записи
    type RecordOption;

    /// Выполняет добавление записи
    type LogAppend: LogRecordAppender<ERR = ERR, RecordId = RecordId>;

    /// Некий лог файл
    type LogFile;

    /// Получение списка лог файлов
    fn get_log_files( &self ) -> Result<Vec<Self::LogFile>, ERR>;

    /// Переключение активного лог файла
    fn switch_log_file( &mut self ) -> Result<(), ERR>;

    /// Добавление записи в лог
    fn append( &mut self, data: &[u8] ) -> Result<Self::LogAppend, ERR>;

    /// Чтение записи из лога
    fn read_record( &self, record_id: RecordId ) -> Result<Self::Record, ERR>;

    /// Чтение заголовка
    fn read_header( &self, record_id: RecordId ) -> Result<Self::RecordOption, ERR>;

    /// Подсчет кол-ва записей
    fn get_records_count( &self ) -> Result<u64, ERR>;
}

/// Выполняет добавление записи
pub trait LogRecordAppender {
    /// Ошибки
    type ERR;

    /// Идентификатор записи
    type RecordId;

    /// добавление записи
    fn run() -> Result<Self::RecordId, Self::ERR>;
}

/// Запись лог файла
pub trait LogRecord {
    /// Ошибки
    type Err;

    /// Опции записи
    type RecordOption;

    /// Получение размера данных в байтах
    fn get_data_size( &self ) -> Result<u32, Self::Err>;

    /// Получение данных
    /// 
    /// Возвращает размер записанных данных,
    /// может вернуть 0, если:
    /// 
    /// - буффер имеет размер 0
    /// - исходные данны нулевого размера
    fn read_data( &self, buffer: &mut [u8] ) -> Result<u32, Self::Err>;
}

