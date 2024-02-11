use std::{fs::File, io::Write, path::PathBuf, sync::Arc};

use rhai::Engine;
#[derive(Clone)]
pub struct TMPFile {
    path: PathBuf,
}
impl TMPFile {
    pub fn new(extension: &str, contents: &str) -> Arc<Self> {
        let mut tmp = std::env::temp_dir();
        let fname = format!("tmp{}.{extension}", rand::random::<u32>());
        // /println!("fname:{fname:?}");
        tmp.push(fname);
        File::create(&tmp)
            .unwrap()
            .write(contents.as_bytes())
            .unwrap();
        Arc::new(Self { path: tmp })
    }
    fn empty(extension: &str) -> Arc<Self> {
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("tmp{}.{extension}", rand::random::<u32>()));
        Arc::new(Self { path: tmp })
    }
    fn path(file: TMPFile) -> String {
        file.path.display().to_string()
    }

    pub fn register_rhai_fns(engine: &mut Engine) {
        engine.register_fn("new_tmpfile", Self::new);
        engine.register_fn("empty_tmpfile", Self::empty);
        engine.register_fn("path", Self::path);
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
}
impl Drop for TMPFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.path).inspect_err(|err| println!("err:{err:?}"));
    }
}

pub struct TMPCrate {
    crate_path: PathBuf,
    main_path: PathBuf,
}
impl TMPCrate {
    pub fn new(contents: &str) -> Arc<Self> {
        let tmp = std::env::temp_dir();
        let crate_name = format!("tmp{}", rand::random::<u32>());
        std::process::Command::new("cargo")
            .current_dir(&tmp)
            .arg("new")
            .arg(&crate_name)
            .output();
        let mut crate_path = tmp.clone();
        crate_path.push(crate_name);
        println!("crate_path:{crate_path:?}");
        let mut main_path = crate_path.clone();
        main_path.push("src");
        main_path.push("main");
        main_path.set_extension("rs");
        File::create(&main_path)
            .unwrap()
            .write(contents.as_bytes())
            .unwrap();
        Arc::new(Self {
            crate_path,
            main_path,
        })
    }
    pub fn cargo_command(self: Arc<Self>, command: &str) -> crate::command::CommandBuilder {
        let mut cmd = crate::command::CommandBuilder::new("cargo".into());
        cmd.set_dir(self.crate_path.display().to_string());
        cmd.arg(command.into());
        cmd
    }
    pub fn register_rhai_fns(engine: &mut Engine) {
        engine.register_fn("new_tmp_crate", Self::new);
        engine.register_fn("cargo_command", Self::cargo_command);
    }
}

impl Drop for TMPCrate {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.crate_path).inspect_err(|err| println!("err:{err:?}"));
    }
}
