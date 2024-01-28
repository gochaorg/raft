use std::collections::HashMap;
use std::time::Duration;

use actix_web::{get, post, web, HttpResponse, Responder};
use actix_web::Result;
use date_format::*;
use date_format::Format;
use log_http_client::QueueClient;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, Utc};

use crate::queue_api::ApiErr;
use crate::raft::{Node, Heartbeat};
use crate::state::AppState;
use crate::raft::RaftError;

/*
Create - POST
Read - GET
Update - PUT
Delete - DELETE
*/

/// Настройка маршрутов
pub fn route( cfg: &mut web::ServiceConfig ) {
    cfg
        .service(status)
        .service(bg_job_stop)
        .service(bg_job_start)
        .service(node_add)
        .service(node_list)
        .service(node_status)
        ;
}

#[get("/status")]
async fn status( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Status {
        status: String,
        bg_job: BgJobStatus,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct BgJobStatus {
        running: bool,
        timeout: Duration
    }

    let state = state.raft.lock().unwrap();

    let (bg_running,bg_timeout) = state.bg_job.as_ref().map(|b| 
        ( b.is_running(), b.get_timeout().clone() )
    ).unwrap_or((false,Duration::ZERO));    

    Ok(web::Json(
        Status {
            status: "ok".to_string(),
            bg_job: BgJobStatus {
                running: bg_running,
                timeout: bg_timeout
            }
        }
    ))
}

#[post("/bg/stop")]
async fn bg_job_stop( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    let mut state = state.raft.lock().unwrap();
    state.bg_job.as_mut().map(|j| j.stop());
    Ok(web::Json("try stop"))
}

#[post("/bg/start")]
async fn bg_job_start( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    let mut state = state.raft.lock().unwrap();
    state.bg_job.as_mut().map(|j| j.start());
    Ok(web::Json("try stop"))
}

#[derive(Debug,Deserialize)]
#[serde(rename_all = "camelCase")]
struct NodeAddBody {
    pub base_address: String,
}

#[derive(Debug,Deserialize)]
struct NodeAddPath {
    pub node_id: String,
}

#[post("/node/{node_id}")]
async fn node_add( state: web::Data<AppState>, body:web::Json<NodeAddBody>, path: web::Path<NodeAddPath> ) -> 
   Result<impl Responder,ApiErr> 
{
    let path = path.into_inner();    
    let body = body.into_inner();

    let id = &path.node_id;
    let base_addr = &body.base_address;

    let mut raft = state.raft.lock()?;
    let id_matched = raft.nodes.iter().find_map(|a| if a.id.eq(id){ Some(()) }else{ None } ).is_some();
    let addr_matched = raft.nodes.iter().find_map(|a| if a.base_address.eq(base_addr){ Some(()) }else{ None } ).is_some();

    if id_matched { 
        return Err(ApiErr::BadRequest(format!("node {id} already registerd")));
    }

    if addr_matched { 
        return Err(ApiErr::BadRequest(format!("base address {base_addr} already registerd")));
    }

    let mut client = QueueClient::new(base_addr.to_string()).map_err(|e| RaftError::CantCreateClient(e))?;
    client.version_timeout = Some(Duration::from_secs(3));

    raft.nodes.push(Node {
        id: id.to_string(),
        base_address: base_addr.to_string(),
        hearbeat: vec![],
        client: client,
    });

    Ok(web::Json(""))
}

#[get("/node")]
async fn node_list( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    let raft = state.raft.lock()?;

    #[derive(Serialize)]
    struct NodeView {
        pub base_address:String,
    }
    let mut nodes: HashMap<String, NodeView> = HashMap::new();

    for node in raft.nodes.iter() {
        nodes.insert(node.id.to_string(), NodeView { base_address: node.base_address.to_string() });
    }

    Ok(web::Json(nodes))
}

#[get("/node/{node_id}")]
async fn node_status( state: web::Data<AppState>, path: web::Path<NodeAddPath> ) -> Result<impl Responder,ApiErr> {
    let path = path.into_inner();
    let node_id = path.node_id;

    let raft = state.raft.lock()?;
    let node = raft.nodes.iter().find(|n| n.id == node_id);
    if node.is_none() {
        return Err(ApiErr::Raft(RaftError::NotFound));
    }

    let node = node.unwrap();

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct NodeStatus {
        base_address: String,
        hearbeat: Vec<HBeat>,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    enum HBeat {
        Succ { latancy: String, started: String },
        Connect { started: String },
        Timeout { started: String },
    }

    let df = DateFormat::parse("utc:yyyy-mm-ddThh:mi:ss.s3zhm");
    
    impl HBeat {
        pub fn from( h:Heartbeat, df:&DateFormat ) -> Self {
            match h {
                Heartbeat::Timeout { started } => Self::Timeout { started: started.format(df) },
                Heartbeat::ConnectFail { started } => Self::Connect { started: started.format(df) },
                Heartbeat::Succ { started, latency } => Self::Succ { started: started.format(df), latancy: latency.as_millis().to_string() },
            }
        }
    }

    Ok( web::Json(NodeStatus {
        base_address: node.base_address.to_string(),
        hearbeat: node.hearbeat.iter().map(|h| HBeat::from(h.clone(), &df)).collect()
    }))
}