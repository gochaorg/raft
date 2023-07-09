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

/// Запись в лог
pub trait LogWriting<ERR,RecordId,Record> {
    fn write( &self, record:Record ) -> Result<RecordId,ERR>;
}
