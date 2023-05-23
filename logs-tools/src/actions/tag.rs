//! Указывает опции для записи в лог файле
//! 
//! Для конкретной записи можно указать опции ([BlockOptions]) 

use std::{path::Path, fs::{File, Metadata}};

use chrono::{DateTime, Utc};
use logs::block::{BlockOptions, String16, String32};

use crate::err::LogToolErr;

/// Указывает какие пары ключ/значение указать в логе
/// 
/// Применяются к тому или иному контексту ( [ApplyTag], [CommonContext], [FileContext] )
#[derive(Debug,Clone)]
pub enum TagAction {
    Clear,
    AddTag{ key: String16, value:String32 },
    AddLogWriteTime{ key: String16, format:String },
    AddFileModifyTime{ key: String16, format:String },
    AddFileName{ key:String16 },
    AddEnvVariable{ key:String16, env_var_name:String }
}

/// Результат применения
#[derive(Debug,Clone,Copy)]
pub enum TagApplyResult {
    Applied,
    Skipped
}

pub trait ApplyTag {
    fn apply(&self, options: &mut BlockOptions, tag:&TagAction ) -> Result<TagApplyResult,LogToolErr>;
}

/// Общий контекст
#[derive(Debug,Clone)]
pub struct CommonContext;

impl ApplyTag for CommonContext {
    fn apply(&self, options: &mut BlockOptions, tag:&TagAction ) -> Result<TagApplyResult,LogToolErr> {
        match &tag {
            TagAction::Clear => {
              options.clear();
              Ok(TagApplyResult::Applied)
            },
            TagAction::AddTag { key, value } => {
              options.set(key, value)?;
              Ok(TagApplyResult::Applied)
            },
            TagAction::AddEnvVariable { key, env_var_name } => {
              match std::env::var(env_var_name) {
                Ok(value) => {
                  options.set(key, value)?;
                  Ok(TagApplyResult::Applied)
                },
                Err(err) => {
                  Err(LogToolErr::ApplyTagFail { message: format!("can't read os var: {env_var_name}: {e}", e=err.to_string()), tag: tag.clone() })
                }
              }
            },
            _ => {
                Ok(TagApplyResult::Skipped)
            }
        }
    }
}

#[derive(Debug,Clone)]
pub struct FileContext<'a, P> 
where P: AsRef<Path>
{
    pub file_name: P,

    pub file: &'a File,
    
    pub metadata: &'a Metadata,

    pub data: &'a Vec<u8>,    
}

impl<'a, P> ApplyTag for FileContext<'a,P>  
where P: AsRef<Path>
{
    fn apply(&self, options: &mut BlockOptions, tag:&TagAction ) -> Result<TagApplyResult,LogToolErr> {
        match &tag {
            TagAction::AddFileModifyTime { key, format } => {
                let mod_time = self.metadata.modified()?;
                let mod_time: DateTime<Utc> = mod_time.into();
                let mod_time = mod_time.format(format).to_string();
                options.set(key, mod_time)?;
                Ok(TagApplyResult::Applied)
            },
            TagAction::AddFileName { key } => {
                match self.file_name.as_ref().to_str() {
                    Some(file_name) => {
                        options.set(key, file_name.to_string())?;
                        Ok(TagApplyResult::Applied)
                    },
                    None => {
                        Err(LogToolErr::ApplyTagFail { message: format!("can't read file name"), tag: tag.clone() })
                    }
                }
            }
            _ => Ok(TagApplyResult::Skipped)
        }
    }
}
