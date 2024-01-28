/// Конфигурация
mod config;

/// Состояние
mod state;

/// Rest api для работы с очередью
mod queue_api;

/// Статические ресурсы
mod static_api;

/// Raft
mod raft;

use std::{env, marker::PhantomData, path::PathBuf, sync::{Arc, Mutex}, time::Duration};
use actix_cors::Cors;
use env_logger::Env;
use log::{debug, info, warn};
use actix_web::{web, App, HttpServer, guard};
use actix_web::middleware::Logger;

use config::AppConfig;
use logs::{logqueue::{find_logs::FsLogFind, LogQueueConf, LogQueueFileNumID, LogQueueFileNumIDOpen, ValidateStub, LogFileQueue}, bbuff::absbuff::FileBuff, logfile::LogFile};
use logs::logqueue::path_template2;
use path_template::PathTemplateParser;

use crate::{config::CmdLineParams, raft::RaftState, state::AppState};
use crate::raft::bg_tasks::*;

/// Очередь
static mut QUEUE_GLOBAL: Option<Arc<Mutex<dyn LogFileQueue<LogQueueFileNumID,PathBuf,LogFile<FileBuff>>  >>> = None;

/// Работа с очередю
/// 
/// Аргументы
/// 
/// - `work` - функция получающая ссылку на очередь
pub fn queue<F,R>( work:F ) -> R 
where
    F: Fn( Arc<Mutex<dyn LogFileQueue<LogQueueFileNumID,PathBuf,LogFile<FileBuff>>>> ) -> R
{
    let q = unsafe{ QUEUE_GLOBAL.clone().unwrap() };
    work(q)
}

/// Входная точка программы
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //env_logger::init();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let cmd_line = CmdLineParams::from_cmd_line();

    let app_conf = cmd_line.clone().conf_file.map(|f| AppConfig::load(PathBuf::from(f)));
    let app_conf = move || {
        match app_conf {
            None => AppConfig::find_or_default(),
            Some(conf) => match conf {
                Ok(conf) => conf,
                Err(err) => {           
                    println!("config not load! {}", err.to_string());                    
                    panic!()
                }
            }
        }
    };

    let app_conf = 
        cmd_line.apply(
            app_conf()
        );
    let app_conf = Arc::new(app_conf);

    info!("starting server on {}:{}", &app_conf.web_server.host, app_conf.web_server.port);
    let conf_t = app_conf.clone();

    /// Задает переменные в шаблон:
    /// 
    /// - `exe.dir` Каталог exe файла, определяется из env
    /// - `work.dir` Рабочий каталог, может быть переопределен параметрами командной строки
    fn template_vars<'a>( tp: PathTemplateParser<'a>, conf:Arc<AppConfig> ) -> PathTemplateParser<'a> {
        tp.with_variable("exe.dir", env::current_exe().map(|f| f.parent().unwrap().to_str().unwrap().to_string() ).unwrap() )
          .with_variable("work.dir", conf.work_dir.to_string())
    }

    let template_parser = move || {
        template_vars(PathTemplateParser::default(),conf_t.clone())
    };

    // static files ..........
    debug!("configure static files");
    let static_files_opt = app_conf.web_server.static_files.clone().map(|tmpl|{
        PathBuf::from( template_parser().parse(&tmpl).unwrap().generate() )
    });
    let static_files_opt = Arc::new(Mutex::new(static_files_opt));

    // queue ..........
    debug!("openning queue");
    let fs_log_find = FsLogFind::new( 
        &template_parser().parse(&app_conf.queue.find.root).unwrap().generate(), 
        "*.binlog", 
        true ).unwrap();

    let log_queue_conf: LogQueueConf<LogQueueFileNumID, PathBuf, FileBuff, _, _, _, _> = {
        let conf = app_conf.clone();
        LogQueueConf {
            find_files: fs_log_find,
            open_log_file: LogQueueFileNumIDOpen,
            validate: ValidateStub,
            new_file: path_template2( &app_conf.queue.new_file.template, move |tp| template_vars(tp, conf.clone())).unwrap(),
            _p: PhantomData.clone(),
        }
    };    

    let queue = log_queue_conf.open().unwrap();
    let queue: Arc<Mutex<dyn LogFileQueue<LogQueueFileNumID,PathBuf,LogFile<FileBuff>>  >> = Arc::new(Mutex::new(queue));

    unsafe {
        QUEUE_GLOBAL = Some(queue);
    }

    info!("queue openned");

    ///////////////////////////////////////////////////////
    let raft_state = RaftState::default();
    let raft_state = Arc::new(Mutex::new(raft_state));
    let raft_state_0 = raft_state.clone();

    let mut bg = raft::bg_tasks::bg_job_async(move || {
        let raft_state_00 = raft_state_0.clone();
        async move {            
            match raft_state_00.try_lock() {
                Ok(mut state) => {
                    state.on_timer().await;
                }
                Err(lock_err) => {
                    warn!("can't lock RaftState {}", lock_err.to_string());
                }
            };
        }
    });

    bg.set_timeout(Duration::from_secs(2));
    bg.set_name("raft bg job");
    let _ = bg.start();

    let m_bg : Box<dyn raft::bg_tasks::job::Job + Send + Sync> = Box::new(bg);
    {
        let mut r = raft_state.lock().unwrap();
        r.bg_job = Some(m_bg);
    }
    ///////////////////////////////////////////////////////
    
    let debug = Arc::new(
        Mutex::new(
            crate::state::Debug::default()
        )
    );

    // configure atix ...........
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        let app = App::new();
        let app = 
            app
                .wrap(cors)
                .wrap(Logger::new("%a %r %s, time=%T"));

        let app = app.app_data(web::Data::new(AppState {
            static_files: static_files_opt.clone(),
            raft: raft_state.clone(),
            debug: debug.clone(),
        }));

        // https://peterevans.dev/posts/how-to-host-swagger-docs-with-github-pages/

        let app = app
            .service(web::resource("/{name}.{ext:html|css|js|png|jpg}").route(web::route().guard(guard::Get()).to(static_api::get_static)));
        let app = app.service(static_api::hello);
        let app = app
            .service(web::scope("/queue").configure(queue_api::queue_api_route))
            .service(web::scope("/raft").configure(raft::rest_api::route))
            ;
        app
    })
    .bind((app_conf.clone().web_server.host.clone(), app_conf.web_server.port))?
    .run()
    .await
}