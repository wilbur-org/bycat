use std::path::Path;

use async_process::Command;

use crate::{GeenieError, Item};
use spurgt::{Asger, Spinner, Spurgt};

pub struct Process {
    cmd: String,
    args: Vec<String>,
    output: bool,
}

impl Process {
    pub fn arg(mut self, arg: impl ToString) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn output(mut self, output: bool) -> Self {
        self.output = output;
        self
    }
}

impl<E: Asger> crate::command::Command<E> for Process {
    fn run<'a>(
        &'a self,
        env: &'a mut Spurgt<E>,
        path: &'a Path,
    ) -> impl std::future::Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            let cmd = format!("{} {}", self.cmd, self.args.join(" "));

            let mut spinner = Spinner::new(env);

            spinner.start(format!("Executing {}", cmd));

            let ret = Command::new(&self.cmd)
                .args(&self.args)
                .current_dir(path)
                .output()
                .await;

            let ret = match ret {
                Ok(ret) => {
                    spinner.stop(format!("Executed {}", cmd));
                    ret
                }
                Err(err) => {
                    spinner.error(err.to_string());
                    return Err(GeenieError::backend(err));
                }
            };

            if self.output {
                env.info(&*String::from_utf8_lossy(&ret.stdout))
                    .await
                    .map_err(GeenieError::backend)?;
            }

            if !ret.status.success() {
                return Err(GeenieError::command(
                    String::from_utf8_lossy(&ret.stderr).to_string(),
                ));
            }

            Ok(())
        }
    }
}

impl<E: Asger, C> Item<E, C> for Process {
    fn process<'a>(
        self,
        mut ctx: crate::Context<'a, E, C>,
        _env: &'a mut Spurgt<E>,
    ) -> impl std::future::Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            ctx.command(self);
            Ok(())
        }
    }
}

pub fn process(cmd: impl ToString) -> Process {
    Process {
        cmd: cmd.to_string(),
        args: Vec::new(),
        output: false,
    }
}
