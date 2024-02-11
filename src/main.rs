use clap::*;
use rand::*;
use rhai::{Dynamic, Engine, FnPtr, ImmutableString, Scope, AST};
use std::process::Command;
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};

mod command;
mod minimizer;
mod tmp;
#[derive(Parser)]
struct MinmizeCommands {
    #[arg(short, long)]
    source_path: PathBuf,
    #[arg(short, long)]
    minimizer_path: PathBuf,
}

struct Evaluator {
    engine: Arc<Engine>,
    validators: Vec<Vaildator>,
}
impl Evaluator {
    fn new(engine: Arc<Engine>, validators: Vec<Vaildator>) -> Self {
        Self { engine, validators }
    }
    fn evaluate(&self, file: &str) -> Result<(), String> {
        for vaildator in &self.validators {
            let res = vaildator
                .ptr
                .call::<Result<(), String>>(&self.engine, &AST::empty(), (file.to_owned(),))
                .unwrap();
            res?;
        }
        Ok(())
    }
}
#[derive(Clone)]
struct Vaildator {
    pub ptr: FnPtr,
    pub name: String,
}
fn new_minimizer() -> Vec<Vaildator> {
    Vec::new()
}

fn not_equivalent(msg: &str) -> Result<(), String> {
    Err(msg.into())
}
fn equivalent() -> Result<(), String> {
    Ok(())
}
fn add_validator(validators: &mut Vec<Vaildator>, fn_ptr: FnPtr, name: String) {
    validators.push(Vaildator { ptr: fn_ptr, name })
}

fn main() {
    let args = MinmizeCommands::parse();
    // Set up the engine
    let mut engine = Engine::new();
    engine.register_fn("new_minimizer", new_minimizer);
    engine.register_fn("add", add_validator);
    engine.register_fn("not_equivalent", not_equivalent);
    engine.register_fn("equivalent", equivalent);
    tmp::TMPFile::register_rhai_fns(&mut engine);
    tmp::TMPCrate::register_rhai_fns(&mut engine);
    command::CommandBuilder::register_rhai_fns(&mut engine);
    command::CommandResults::register_rhai_fns(&mut engine);
    // Register predefined functions
    let ast = engine.compile(include_str!("predefined.rhai")).unwrap();
    let module = rhai::Module::eval_ast_as_new(Scope::new(), &ast, &engine).unwrap();
    println!("module:{module:?}");
    engine.register_global_module(module.into());
    //engine.register_fn("launch_command", launch_command);

    let mut source_file = File::open(args.minimizer_path).unwrap();
    let mut source = Vec::new();
    source_file.read_to_end(&mut source).unwrap();
    let source = String::from_utf8(source).unwrap();
    let ast = engine.compile(source).unwrap();
    let engine = Arc::new(engine);
    let result = engine
        .call_fn::<Vec<Vaildator>>(&mut Scope::new(), &ast, "init", ())
        .unwrap();
    let eval = Evaluator::new(engine, result);

    let source_path = args.source_path;
    let mut last_ok_path = source_path.clone();
    last_ok_path.set_file_name("last_ok");
    if let Some(ext) = source_path.extension() {
        last_ok_path.set_extension(ext);
    }

    let mut min = minimizer::RustSourceFile::from_file(std::io::BufReader::new(
        std::fs::File::open(source_path).unwrap(),
    ))
    .unwrap();

    min.try_remove_lines(&|f| eval.evaluate(&f.to_string()), &last_ok_path);
}
