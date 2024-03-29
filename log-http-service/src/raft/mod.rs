//! Алгоритм консунсуса RAFT
//! 
//! Ссылки
//! - https://habr.com/ru/companies/dododev/articles/469999/
//! - https://raft.github.io/
//! 
//! Основные термины
//! ==================
//! 
//! - Узел - участник кластера (id), 
//!   самостоятельный (http/tcp/...) сервер входящий в группу таких же кластеров.
//!   
//!   Состояния узлов (enum)
//!     - Лидер (Leader)
//!     - Сторонник (Follower)
//!     - Кандидат (Candidate)
//! - Коммуникация между серверами
//!   Сообщения деляться на (enum)
//!     - RequestVote
//!     - AppendEntries
//! - Срок (электоральный)
//!     - Сроки нумеруются последовательно (number)
//!     - В течения одного срока выделются 2 фазы (enum)
//!         - Выборы
//!         - Обычная работа
//! 
//! Выбор лидера
//! =================
//! 
//! Для определения момента, когда пора начинать новые выборы, Raft полагается на heartbeat. 
//! Фоловер остаётся фоловером до тех пор, пока он получает сообщения от действующего лидера или кандидата. 
//! Лидер периодически рассылает всем остальным серверам heartbeat.
//! 
//! Если фоловер не будет получать никаких сообщений некоторое время, он вполне закономерно предположит, 
//! что лидер умер, а значит самое время брать инициативу в свои руки. В этот момент бывший фоловер инициирует выборы.
//! 
//! Для инициации выборов фоловер инкрементит свой номер срока, переходит в состояние «кандидат», 
//! голосует сам за себя и затем рассылает запрос «RequestVote» всем остальным серверам. 
//! После этого кандидат ждёт одного из трёх событий:
//! 
//! 1. **Кандидат получает большинство голосов (включая свой) и побеждает в выборах.** <br>
//!   Каждый сервер голосует в каждом сроке лишь единожды, за первого достучавшегося кандидата (с некоторым исключением, рассмотренным далее), 
//!   поэтому набрать в конкретном сроке большинство голосов может только один кандидат. 
//!   Победивший сервер становится лидером, начинает рассылать heartbeat и обслуживать запросы клиентов к кластеру
//! 
//! 2. **Кандидат получает сообщение от уже действующего лидера текущего срока или от любого сервера более старшего срока.** <br>
//!   В этом случае кандидат понимает, что выборы, в которых он участвует, уже не актуальны. 
//!   Ему не остаётся ничего, кроме как признать нового лидера/новый срок и перейти в состояние фоловер.
//! 
//! 3. **Кандидат не получает за некоторый таймаут большинство голосов.** <br>
//!   Такое может произойти в случае, когда несколько фоловеров становятся кандидатами, 
//! и голоса разделяются среди них так, что ни один не получает большинства. 
//! В этом случае срок заканчивается без лидера, а кандидат сразу же начинает новые выборы на следующий срок.
//! 
//!  Реплицируем логи
//! =======================
//! 
//! Когда лидер выбран, на него ложится ответственность за управление распределённым логом. 
//! Лидер принимает от клиентов запросы, содержащие некоторые команды. 
//! Лидер кладёт в свой лог новую запись, содержащую команду, 
//! а затем отсылает «AppendEntries» всем фоловерам, для того чтобы отреплицировать запись с новой записью.
//! 
//! Когда запись будет успешно разреплицирована на большинстве серверов, 
//! лидер начинает считать запись закоммиченой и отвечает клиенту. 
//! Лидер следит за тем, какая запись является последней. 
//! Он отправляет номер этой записи в AppendEntries (включая heartbeat), 
//! чтобы фоловеры могли закоммитить запись у себя.
//! 
//! В случае, если лидер не может достучаться до некоторых фоловеров, 
//! он будет ретраить AppendEntries до бесконечности.
//! 
//! Гарантируем надёжность алгоритма
//! ==================================
//! 
//! Пока из того, что мы рассмотрели, непонятно, каким образом Raft может давать хоть какие-то гарантии. 
//! Однако алгоритм предоставляет набор свойств, которые вместе гарантируют надёжность его исполнения:
//! 
//! - **Election Safety**: в рамках одного срока может быть выбрано не более одного лидера. 
//! Это свойство следует из того, что каждый сервер голосует в рамках каждого срока лишь единожды, 
//! а для становления лидера необходимо большинство голосов
//! 
//! - **Leader Append-Only**: лидер никогда не перезаписывает и не стирает, 
//! не двигает записи в своем логе, только дописывает новые записи. 
//! Это свойство следует напрямую из описания алгоритма – единственная операция, которую лидер может совершать со своим логом – дописывать записи в конец. И всё.
//! 
//! - **Log Matching**: если логи двух серверов содержат запись с одинаковым индексом и номером срока, 
//! то оба лога идентичны вплоть до данной записи включительно. 
//! 
//! - **Leader Completeness**: если запись в логе закоммичена в данный срок, 
//! то логи лидеров всех последующий сроков будут включать эту запись. Это свойство предоставляет нам гарантии durability.
//! 
//! - **State Machine Safety**: это свойство в оригинале описывается в терминах распределённых машин состояний, 
//! в терминах нашей статьи это свойство можно описать так – когда сервер коммитит запись с некоторым индексом, 
//! ни один другой сервер не закоммитит по данному индексу другую запись.
//! <br><br>
//! Это свойство следует из прошлого. Если фоловер коммитит некоторую запись по индексу N, 
//! значит его лог идентичен логу лидера вплоть до N включительно. Leader completeness property гарантирует нам, 
//! что все последующие лидеры, будут также содержать эту закоммиченную запись по индексу N, 
//! а значит фоловеры, коммитящие в последующих сроках запись по индексу N, будут коммитить то же самое значение.

/// реализация алгоритма выбора
mod election;

mod rand_duration;
pub use rand_duration::*;

mod domain;
pub use domain::*;

mod api_spec;
pub use api_spec::*;

/// Фоновые задачи
pub mod bg_tasks;

/// API для RAFT
pub mod rest_api;

#[cfg(test)]
mod test {
    use std::sync::{Arc, RwLock};
    use std::{time::Duration, fmt::Debug};
    use actix_rt::{System, spawn};
    use actix_rt::time::sleep;
    use futures::Future;
    use futures::future::join_all;
    
    #[test]
    fn sequence_test() {
        async fn try_n_times<F,R>(n:u32, f:F) 
        where 
            F: Fn() -> R,
            R: Future,
            R::Output : Debug,
        {
            for _i in 0..n {
                sleep(Duration::from_millis(1000) ).await;
                let res = f().await;
                println!("{res:?}");
            }
        }

        System::new().block_on( async {
            println!("try 10");
            try_n_times(3, 
                || { async {
                    println!("be be");
                    1
                }}
            ).await
        })
    }

    #[test]
    fn parallel_test() {
        async fn collect<R>( itms:Vec<R> ) -> i32 
        where
            R: Future<Output = i32>,
        {
            let res = join_all(itms).await;
            res.iter().fold(0, |acc,it| acc + it)
        }

        fn ret_f(n:i32) -> impl Future<Output = i32> {
            async move {
                println!("compute {}",n);
                sleep(Duration::from_millis(500)).await;
                n
            }
        }

        System::new().block_on(async {
            let s = collect( vec![
                ret_f(1), ret_f(2), ret_f(3), ret_f(4),
                ret_f(5), ret_f(6), ret_f(7), ret_f(8)
            ]).await;
            println!("sum {s}");
        })
    }

    #[test]
    fn bg_task_test() {
        let counter = Arc::new(RwLock::new(0));

        let _h = spawn( async move {
            loop {
                sleep(Duration::from_millis(500)).await;
                let mut c = counter.write().unwrap();
                *c += 1;
            }
        });
    }
}