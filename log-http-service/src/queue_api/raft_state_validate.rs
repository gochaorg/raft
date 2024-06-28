use actix_web::{http::Uri, web, HttpRequest, HttpResponse};
use crate::state::AppState;
use super::ApiErr;

enum RaftMasterMatch {
    Matched,
    NotMatched {
        base_address: String
    }
}

/// Проверка наличия состояния raft.master и наличие заголовка Raft-Master-Id = raft.master.id
fn match_raft_master( state: &web::Data<AppState>, req: &HttpRequest ) -> Result<RaftMasterMatch, ApiErr>
{
    let raft = state.raft.lock()?;
    Ok(match &raft.master {
        Some(master_node) => {
            let matched = req.headers().get("Raft-Master-Id").and_then(|req_m_id| req_m_id.to_str().ok() ).map( |req_m_id| req_m_id == master_node.id ).unwrap_or(false);
            match matched {
                true => RaftMasterMatch::Matched,
                false => RaftMasterMatch::NotMatched { base_address: master_node.base_address.clone() }
            }
        }
        None => { 
            RaftMasterMatch::Matched
        }
    })
}

/// Валидация запроса на наличие заголовка Raft-Master-Id = raft.master.id.
/// 
/// Выполняет обработчик `f`, когда:
/// - Если raft.master установлен и совпадает заголовком `Raft-Master-Id` запроса
/// - Если raft.master не установлен
/// 
/// Иначе, запрос отклоняется
pub fn validate_raft_master<F: FnOnce() -> Result<HttpResponse,ApiErr>>( state: &web::Data<AppState>, req: &HttpRequest, f:F ) -> Result<HttpResponse,ApiErr> {
    match match_raft_master(&state, &req)? {
        RaftMasterMatch::Matched => {
            let r = f()?;
            Ok(r)
        },
        RaftMasterMatch::NotMatched { base_address: addr }  => {
            Ok(match req.uri().path_and_query().map(|pq| format!(
                "{addr}{pq}"
            )) {
                Some(target) => {
                    HttpResponse::TemporaryRedirect().append_header(("Location",target)).body("accept only from master")
                }
                None => {
                    HttpResponse::Forbidden().body("accept only from master")
                }
            })
        }
    }
}

#[test]
fn uri_test(){
    let cur = Uri::try_from("http://localhost:8080/abc/cde?f=g&h=i").unwrap();
    println!("{}", cur.path_and_query().unwrap());
}