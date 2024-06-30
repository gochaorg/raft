use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use actix_web::{delete, get, post, web, HttpResponse, Responder};
use actix_web::Result;
use date_format::*;
use date_format::Format;
use log_http_client::QueueClient;
use serde::{Deserialize, Serialize};

use crate::queue_api::ApiErr;
use crate::raft::*;
use crate::state::AppState;
use crate::raft::RaftError;

#[get("/status")]
async fn status( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Status {
        id: String,
        status: String,
        bg_job: BgJobStatus,
        master: Option<MasterNode>,
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
            id: state.id.clone(),
            status: "ok".to_string(),
            bg_job: BgJobStatus {
                running: bg_running,
                timeout: bg_timeout
            },
            master: state.master.clone(),
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
        log_shipping: Arc::new(Mutex::new(HashMap::new())),
        log_shipping_idseq: Arc::new(AtomicU32::new(0)),
    });

    Ok(web::Json(""))
}

#[derive(Debug,Deserialize)]
struct NodeDelPath {
    pub node_id: String,
}

#[delete("/node/{node_id}")]
async fn node_del( state: web::Data<AppState>, path: web::Path<NodeDelPath> ) -> Result<impl Responder,ApiErr> {
    let mut state = state.raft.lock()?;
    let path = path.into_inner();

    match state.nodes.iter().enumerate().find_map(|(idx,n)| if n.id == path.node_id { Some(idx) } else { None } ) {
        Some(idx) => { state.nodes.remove(idx); Ok(()) },
        _ => { Err(ApiErr::BadRequest(format!("node {} not found", path.node_id))) }
    }?;

    Ok(web::Json(""))
}

#[get("/node")]
async fn node_list( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    let raft = state.raft.lock()?;

    #[derive(Serialize)]
    struct NodeView {
        pub base_address:String,
        pub heartbeat_timeout: Option<Duration>,
    }
    let mut nodes: HashMap<String, NodeView> = HashMap::new();

    for node in raft.nodes.iter() {
        nodes.insert(
            node.id.to_string(), 
            NodeView { 
                base_address: node.base_address.to_string(),
                heartbeat_timeout: node.client.version_timeout.clone(),
            });
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

#[derive(Debug,Deserialize)]
struct MasterSet {
    id: String,
    base_address: String,
}

#[post("/master/set")]
async fn master_set( state: web::Data<AppState>, body:web::Json<MasterSet> ) -> Result<impl Responder,ApiErr> {
    let mut raft = state.raft.lock()?;
    let body = body.into_inner();

    raft.master = Some(MasterNode { id: body.id.clone(), base_address: body.base_address.clone() });

    Ok(web::Json(""))
}

#[post("/master/reset")]
async fn master_reset( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    let mut raft = state.raft.lock()?;
    raft.master = None;
    Ok(web::Json(""))
}

#[get("/id")]
async fn id_get( state: web::Data<AppState> ) -> Result<impl Responder,ApiErr> {
    let raft = state.raft.lock()?;
    Ok(HttpResponse::Ok().json(raft.id.clone()))
}

#[post("/id")]
async fn id_set( state: web::Data<AppState>, body:web::Json<String> ) -> Result<impl Responder,ApiErr> {
    let mut raft = state.raft.lock()?;
    let new_id = body.into_inner();
    let old_id = raft.id.clone();
    raft.id = new_id.clone();

    #[derive(Serialize)]
    struct IdChange { old_id: String, new_id: String }

    Ok(HttpResponse::Ok().json( IdChange { old_id: old_id, new_id: new_id.clone() } ))
}
