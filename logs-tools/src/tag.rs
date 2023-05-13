use std::{fs::File, path::Path};

use logs::block::{BlockOptions, String16, String32};

use crate::err::LogToolErr;

#[derive(Debug, Clone)]
pub enum TagAction {
    Clear,
    AddTag { key: String16, value: String32 },
    AddLogWriteTime { key: String16, format: String },
    AddFileModifyTime { key: String16, format: String },
    AddFileName { key: String16 },
    AddEnvVariable { key: String16, env_var_name: String },
}

#[derive(Debug, Clone, Copy)]
pub enum TagApplyResult {
    Applied,
    Skipped,
}

pub trait ApplyTag {
    fn apply(
        &self,
        options: &mut BlockOptions,
        tag: &TagAction,
    ) -> Result<TagApplyResult, LogToolErr>;
}

pub struct CommonTag;

impl ApplyTag for CommonTag {
    fn apply(
        &self,
        options: &mut BlockOptions,
        tag: &TagAction,
    ) -> Result<TagApplyResult, LogToolErr> {
        match &tag {
            TagAction::Clear => {
                options.clear();
                Ok(TagApplyResult::Applied)
            }
            TagAction::AddTag { key, value } => {
                options.set(key, value)?;
                Ok(TagApplyResult::Applied)
            }
            TagAction::AddEnvVariable { key, env_var_name } => match std::env::var(env_var_name) {
                Ok(value) => {
                    options.set(key, value)?;
                    Ok(TagApplyResult::Applied)
                }
                Err(err) => Err(LogToolErr::ApplyTagFail {
                    message: format!("can't read os var: {env_var_name}"),
                    tag: tag.clone(),
                }),
            },
            _ => Ok(TagApplyResult::Skipped),
        }
    }
}

pub struct FileContext<'a, P>
where
    P: AsRef<Path>,
{
    file_name: P,
    file: &'a File,
    data: &'a Vec<u8>,
}
