/// Лог - хранит в себе сумму лог файлов [crate::logfile]
pub trait Log {
    /// Ошибки
    type Err;

    /// Идентификатор записи
    type RecordId;

    /// Запись
    type Record : LogRecord<RecordOption = Self::RecordOption>;

    /// Опции записи
    type RecordOption;

    /// Выполняет добавление записи
    type LogAppend: LogRecordAppender<Err = Self::Err, RecordId = Self::RecordId>;

    /// Некий лог файл
    type LogFile;

    /// Получение списка лог файлов
    fn get_log_files( &self ) -> Result<Vec<Self::LogFile>, Self::Err>;

    /// Переключение активного лог файла
    fn switch_log_file( &mut self ) -> Result<(), Self::Err>;

    /// Добавление записи в лог
    fn append( &mut self, data: &[u8] ) -> Result<Self::LogAppend, Self::Err>;

    /// Чтение записи из лога
    fn read_record( &self, record_id: Self::RecordId ) -> Result<Self::Record, Self::Err>;

    /// Чтение заголовка
    fn read_header( &self, record_id: Self::RecordId ) -> Result<Self::RecordOption, Self::Err>;

    /// Подсчет кол-ва записей
    fn get_records_count( &self ) -> Result<u64, Self::Err>;

    /// Получение id последней записи
    fn get_last_record( &self ) -> Result<Option<Self::RecordId>, Self::Err>;

    /// Получение id первой записи
    fn get_first_record( &self ) -> Result<Option<Self::RecordId>, Self::Err>;

    /// Получение id следующей записи
    fn get_next_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,Self::Err>;

    /// Получение id предыдущей записи
    fn get_previous_record( &self, record_id: Self::RecordId ) -> Result<Option<Self::RecordId>,Self::Err>;    
}

/// Выполняет добавление записи
pub trait LogRecordAppender {
    /// Ошибки
    type Err;

    /// Идентификатор записи
    type RecordId;

    /// добавление записи
    fn run() -> Result<Self::RecordId, Self::Err>;
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

