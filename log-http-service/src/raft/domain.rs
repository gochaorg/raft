/// Роль
#[derive(Clone,Debug)]
#[allow(unused)]
pub enum Role {
    Follower,
    Candidate,
    Leader
}

/// Ошибки
#[allow(dead_code)]
pub enum RErr {
    /// Нет ответа
    ReponseTimeout,

    /// Номер эпохи не совпаддает с ожидаемым
    EpochNotMatch {
        expect: u32,
        actual: u32,
    },

    /// Уже проголосовал
    AlreadVoted {
        nominant: String
    }
}