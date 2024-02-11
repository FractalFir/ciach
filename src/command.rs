use std::process::{Command, ExitStatus};

use crate::tmp::TMPFile;
#[derive(Clone)]
pub struct CommandResults {
    pub has_launch_failed: bool,
    pub has_timedout: bool,
    pub stdout: String,
    pub stderr: String,
}
impl CommandResults {
    /*
    fn merge(self,other:Self){
        let mut stdout = self.stdout;
        stdout.push_str(&other.stdout);
    }*/
    fn has_launch_failed(self) -> bool {
        self.has_launch_failed
    }

    fn has_timedout(&self) -> bool {
        self.has_timedout
    }
    fn is_ok(self) -> bool {
        !self.has_launch_failed && !self.has_timedout
    }
    fn stdout(self) -> String {
        self.stdout
    }

    fn stderr(self) -> String {
        self.stderr
    }
    pub fn register_rhai_fns(engine: &mut rhai::Engine) {
        engine.register_fn("is_ok", Self::is_ok);
        engine.register_fn("has_launch_failed", Self::has_launch_failed);
        engine.register_fn("stdout", Self::stdout);
        engine.register_fn("stderr", Self::stderr);
    }
}
#[derive(Clone)]
pub struct CommandBuilder {
    exec_path: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    dir: Option<String>,
}
impl CommandBuilder {
    pub fn new(exec_path: String) -> Self {
        Self {
            exec_path,
            args: vec![],
            env: vec![],
            dir: None,
        }
    }
    pub fn new_with_timeout(exec_path: String, sec: f64) -> Self {
        #[cfg(not(target_os = "linux"))]
        {
            todo!("Can't launch commands with timeout on OSs different than Linux.")
        }
        let mut cmd = Self::new("timeout".into());
        cmd.arg(format!("{sec}"));
        cmd.arg(exec_path);
        cmd
    }
    pub fn arg(&mut self, arg: String) {
        self.args.push(arg)
    }
    fn env(&mut self, key: String, value: String) {
        self.env.push((key, value));
    }
    pub fn set_dir(&mut self, dir: String) {
        self.dir = Some(dir);
    }
    fn to_command(&self) -> Command {
        let mut command = Command::new(&self.exec_path);
        if let Some(dir) = &self.dir {
            command.current_dir(dir);
        }
        for arg in &self.args {
            command.arg(arg);
        }
        for (key, value) in &self.env {
            command.env(key, value);
        }
        command
    }
    fn display(self) -> String {
        format!("{:?}", self.to_command())
    }
    fn launch(self) -> CommandResults {
        let mut command = self.to_command();
        let out = if let Ok(out) = command.output() {
            out
        } else {
            return CommandResults {
                stdout: "".to_string(),
                stderr: "".to_string(),
                has_launch_failed: true,
                has_timedout: false,
            };
        };
        match out.status.code() {
            Some(124) => {
                return CommandResults {
                    stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                    has_launch_failed: false,
                    has_timedout: true,
                };
            }
            None => {
                return CommandResults {
                    stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                    has_launch_failed: true,
                    has_timedout: false,
                };
            }
            _ => (),
        }
        CommandResults {
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            has_launch_failed: false,
            has_timedout: false,
        }
    }
    pub fn register_rhai_fns(engine: &mut rhai::Engine) {
        engine.register_fn("launch", Self::launch);
        engine.register_fn("new_command", Self::new);
        engine.register_fn("new_command_with_timeout", Self::new_with_timeout);
        engine.register_fn("arg", Self::arg);
        engine.register_fn("env", Self::env);
        engine.register_fn("set_dir", Self::set_dir);
        engine.register_fn("launch", Self::launch);
        engine.register_fn("display", Self::display);
        engine.register_fn("__impl_is_valid_rust", Self::check_if_valid_rust);
        CompileCommand::register_rhai_fns(engine);
    }
    fn check_if_valid_rust(file: &str) -> Result<(), String> {
        let tmp = crate::tmp::TMPFile::new("rs", file);
        let mut cmd = Command::new("rustc");
        cmd.arg("+nightly")
            .arg("-Z")
            .arg("no-codegen")
            .arg(&tmp.get_path().display().to_string())
            .arg("--edition")
            .arg("2021");
        let out = cmd.output();
        let out = match out {
            Ok(out) => out,
            Err(_) => return Err("Could not run rustc".into()),
        };
        let stderr = String::from_utf8(out.stderr).unwrap();
        // Ensures file is not removed early
        let _ = tmp;
        if !stderr.contains("error") {
            Ok(())
        } else {
            Err("rustc error".into())
        }
    }
}
#[derive(Clone)]
struct CompileCommand {
    cmd: CommandBuilder,
    exec: String,
    src: std::sync::Arc<TMPFile>,
}
impl CompileCommand {
    fn compile_cmd(file: &str) -> CompileCommand {
        Self::compile_cmd_ext(file, "a")
    }
    fn compile_cmd_ext(file: &str, ext: &str) -> CompileCommand {
        let tmp = crate::tmp::TMPFile::new("rs", file);
        let mut exec = tmp.get_path().to_owned();
        exec.set_extension(ext);
        let mut cmd = CommandBuilder::new("rustc".into());
        cmd.arg(tmp.get_path().display().to_string());
        cmd.arg("--edition".into());
        cmd.arg("2021".into());
        cmd.arg("-o".into());
        cmd.arg(exec.display().to_string());
        CompileCommand {
            cmd,
            exec: exec.display().to_string(),
            src: tmp,
        }
    }
    pub fn register_rhai_fns(engine: &mut rhai::Engine) {
        engine.register_fn("compile_command", Self::compile_cmd);
        engine.register_fn("compile_command", Self::compile_cmd_ext);
        engine.register_fn("launch", Self::launch);
        engine.register_fn("arg", Self::arg);
        engine.register_fn("env", Self::env);
        engine.register_fn("set_dir", Self::set_dir);
        engine.register_fn("launch", Self::launch);
        engine.register_fn("display", Self::display);
        engine.register_fn("exec_file", Self::exec_file);
    }
    fn arg(&mut self, arg: String) {
        self.cmd.arg(arg)
    }
    fn display(self) -> String {
        self.cmd.display()
    }

    fn env(&mut self, key: String, value: String) {
        self.cmd.env(key, value)
    }

    fn launch(self) -> CommandResults {
        self.cmd.launch()
    }

    fn set_dir(&mut self, dir: String) {
        self.cmd.set_dir(dir)
    }

    fn exec_file(self) -> String {
        self.exec
    }
}
