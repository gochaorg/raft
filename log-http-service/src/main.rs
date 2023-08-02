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

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, guard};
use config::AppConfig;
use logs::{logqueue::{find_logs::FsLogFind, LogQueueConf, LogQueueFileNumID, LogQueueFileNumIDOpen, ValidateStub, LogFileQueue}, bbuff::absbuff::FileBuff, logfile::LogFile};
use logs::logqueue::path_template2;
use path_template::PathTemplateParser;
use std::{env, path::PathBuf, sync::{Arc, Mutex}, marker::PhantomData};
use log::{info, debug};
use actix_web::middleware::Logger;
use env_logger::Env;

use crate::{state::AppState, config::CmdLineParams};


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

    let app_conf = 
        CmdLineParams::from_cmd_line().apply(
            AppConfig::find_or_default()
        );
    let app_conf = Arc::new(app_conf);

    info!("starting server on {}:{}", &app_conf.web_server.host, app_conf.web_server.port);
    let conf_t = app_conf.clone();

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
        }));

        let app = app
            .service(web::resource("/{name}.{ext:html|css|js|png|jpg}").route(web::route().guard(guard::Get()).to(static_api::get_static)));
        let app = app.service(static_api::hello);
        let app = app.service(web::scope("/queue").configure(queue_api::queue_api_route));
        app
    })
    .bind((app_conf.clone().web_server.host.clone(), app_conf.web_server.port))?
    .run()
    .await
}