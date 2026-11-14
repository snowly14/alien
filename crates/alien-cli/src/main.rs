use std::{env, error::Error, fs, path::PathBuf, process::ExitCode};

use alien_compiler::{compile_globals, transform_with_config};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("alien: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args().skip(1);
    let mut input = None;
    let mut globals = None;
    let mut module_id = None;
    let mut check = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                println!(
                    "Usage: alien <file.ts|file.tsx> [--globals alien.globals.ts] [--module-id project/path.ts] [--check]\nEmits CSS to stdout; --check validates without emitting. Type generation is provided by the Bun CLI: alien prepare."
                );
                return Ok(());
            }
            "--globals" => {
                globals = Some(PathBuf::from(
                    arguments.next().ok_or("--globals requires a path")?,
                ));
            }
            "--module-id" => {
                module_id = Some(
                    arguments
                        .next()
                        .ok_or("--module-id requires an identifier")?,
                );
            }
            "--check" => check = true,
            option if option.starts_with('-') => {
                return Err(format!("unknown option {option}").into());
            }
            path if input.is_none() => input = Some(PathBuf::from(path)),
            _ => return Err("only one input file is supported".into()),
        }
    }
    let input = input.ok_or("missing input file; run alien --help")?;
    let source = fs::read_to_string(&input)?;
    let config = globals
        .map(|path| {
            let source = fs::read_to_string(&path)?;
            Ok::<_, Box<dyn Error>>(compile_globals(&source, &path)?)
        })
        .transpose()?
        .unwrap_or_default();
    let module_id = module_id.unwrap_or_else(|| input.to_string_lossy().replace('\\', "/"));
    if let Some(output) = transform_with_config(&source, &input, &module_id, &config)?
        && !check
    {
        println!("{}", output.css);
    }
    Ok(())
}
