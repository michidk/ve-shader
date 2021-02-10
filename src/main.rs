#![feature(once_cell)]

use log::{debug, error, info, warn};
use regex::Regex;
use std::{
    fs,
    fs::File,
    io::{BufRead, BufReader},
    lazy::SyncLazy,
    path::{Path, PathBuf},
    str::FromStr,
};
use structopt::StructOpt;

// Cli arguments
#[derive(StructOpt, Debug)]
#[structopt(name = "veshader")]
struct CliArgs {
    /// Specify the shader files to compile using glob
    glob: String,
    /// Enable debug
    #[structopt(short = "d", long = "debug")]
    debug: Option<bool>,
    /// Shader version: vulkan, vulkan1_0, vulkan1_1, vulkan1_2
    #[structopt(short = "s", long = "target-version")]
    shader_version: Option<TargetVersion>,
    /// Optimization level: zero, size, performance
    #[structopt(short = "O", long = "optimization", parse(try_from_str=parse_optimization_level))]
    optimization: Option<shaderc::OptimizationLevel>,
    /// Specify the target
    #[structopt(short = "t", long = "target")]
    target: Option<u32>,
    // Also compile files without the .glsl file extension
    #[structopt(long = "ignore-extension")]
    ignore_extension: bool,
    /// Output directory, to place the compiled shader in
    #[structopt(short = "o", long = "output")]
    output: String,
    /// Output debug info
    #[structopt(long = "verbose")]
    verbose: bool,
    /// ???
    #[structopt(short = "r", long = "rick")]
    rick: bool,
}

// Vulkan target version
#[derive(Debug)]
enum TargetVersion {
    Vulkan1_0,
    Vulkan1_1,
    Vulkan1_2,
}

impl TargetVersion {
    fn into_bitmask(self) -> u32 {
        match self {
            TargetVersion::Vulkan1_0 => 1 << 22,
            TargetVersion::Vulkan1_1 => 1 << 22 | 1 << 12,
            TargetVersion::Vulkan1_2 => 1 << 22 | 2 << 12,
        }
    }
}

impl FromStr for TargetVersion {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vulkan" | "vulkan1_0" => Ok(TargetVersion::Vulkan1_0),
            "vulkan1_1" => Ok(TargetVersion::Vulkan1_1),
            "vulkan1_2" => Ok(TargetVersion::Vulkan1_2),
            _ => Err(CliError::InvalidTarget(String::from(s))),
        }
    }
}

impl Default for TargetVersion {
    fn default() -> Self {
        TargetVersion::Vulkan1_0
    }
}

/// Happens during setup
#[derive(thiserror::Error, Debug)]
enum CliError {
    #[error("Invalid target: {0}")]
    InvalidTarget(String),
    #[error("Unknown error")]
    CompilerCreation,
    #[error("")]
    PatternError(#[from] glob::PatternError),
    #[error("")]
    GlobError(#[from] glob::GlobError),
    #[error("")]
    CompileOptionsError,
}

/// Happens during shader compilation; prints the error and continues
#[derive(thiserror::Error, Debug)]
enum CompilerError {
    #[error("Error reading the file")]
    FileRead(#[from] std::io::Error),
    #[error("Error compiling the shader: {0}")]
    Compilation(String),
    #[error("Unknown shader type: {0}")]
    UnknownShaderType(String),
}

const GLOB_OPTIONS: glob::MatchOptions = glob::MatchOptions {
    case_sensitive: false,
    require_literal_separator: false,
    require_literal_leading_dot: false,
};

fn main() -> Result<(), CliError> {
    let args = CliArgs::from_args();

    if !args.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    } else {
        env_logger::Builder::new()
            .filter(None, log::LevelFilter::Debug)
            .init();
    }

    if args.rick {
        info!("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        return Ok(());
    }

    let mut options = shaderc::CompileOptions::new().ok_or(CliError::CompilerCreation)?;

    // debug
    let mut debug = false;
    if let Some(b) = args.debug {
        debug = b;
    }
    if debug {
        options.set_generate_debug_info();
    }

    // optimization
    options.set_optimization_level(
        args.optimization
            .unwrap_or(shaderc::OptimizationLevel::Performance),
    );

    // target version
    options.set_target_env(
        shaderc::TargetEnv::Vulkan,
        args.shader_version.unwrap_or_default().into_bitmask(),
    );

    // target environment
    if let Some(target) = args.target {
        options.set_forced_version_profile(target, shaderc::GlslProfile::None);
    }

    if args.ignore_extension {
        debug!("Compiling files with all file extensions.")
    }

    let output_path = Path::new(&args.output);

    let glob = glob::glob_with(&args.glob, GLOB_OPTIONS)?;
    for path in glob {
        let path = path?;

        // check extension
        if let Some(Some(extension)) = path.extension().map(|x| x.to_str()) {
            if extension.to_ascii_lowercase() != "glsl" && !args.ignore_extension {
                warn!("Skipped {} because it does not have the .glsl file extension. Ignore with --ignore-extension.", path.display());
            } else {
                let options = options.clone().ok_or(CliError::CompileOptionsError)?;

                info!("Compiling shader at path: {}", path.display());
                if let Err(err) = parse(path, options, &output_path) {
                    error!("{}", err); // handles CompilerError
                }
            }
        } else {
            warn!(
                "Ignored file \"{}\", because no file extension was found.",
                path.display()
            );
        }
    }

    Ok(())
}

static REG: SyncLazy<Regex> = SyncLazy::new(|| Regex::new(r":([0-9]*):").unwrap());

/// Parses a shader file in the custom format
fn parse(
    path: PathBuf,
    mut options: shaderc::CompileOptions,
    output_path: &Path,
) -> Result<(), CompilerError> {
    let include_path = path.clone();
    options.set_include_callback(move |name, ty, src, _depth| {
        let path = match ty {
            shaderc::IncludeType::Relative => Path::new(src).parent().unwrap().join(name),
            shaderc::IncludeType::Standard => include_path.parent().unwrap().join(name),
        };
        let path_str = path.to_str().ok_or("Non-unicode path")?.to_owned();
        Ok(shaderc::ResolvedInclude {
            resolved_name: path_str,
            content: fs::read_to_string(path).map_err(|x| x.to_string())?,
        })
    });

    let mut curr_shader = String::new();
    let mut shader_type: Option<shaderc::ShaderKind> = None;
    let mut line_mapping: Vec<usize> = Vec::new();
    let mut version: Option<String> = None;

    if let Ok(file) = File::open(&path) {
        // read line-by-line
        for (idx, line) in BufReader::new(file).lines().enumerate() {
            if let Ok(line) = line {
                // custom format intsruction
                if line.contains("//#") {
                    let split: Vec<_> = line.split(' ').collect();
                    // parse custom instructions
                    if let Some(&instruction) = split.get(1) {
                        // handle TYPE instruction
                        if instruction.contains("TYPE") {
                            // parse instruction arguments
                            if let Some(&token) = split.get(2) {
                                let new_kind = parse_shader_kind(token).ok_or_else(|| {
                                    CompilerError::UnknownShaderType(String::from(token))
                                })?;
                                if let Some(kind) = shader_type {
                                    compile_shader(
                                        &curr_shader,
                                        &path,
                                        &options,
                                        kind,
                                        line_mapping,
                                        &output_path,
                                        &version,
                                    )?;

                                    curr_shader = String::new();
                                    line_mapping = Vec::new();
                                }
                                shader_type = Some(new_kind);
                            }
                        } else if instruction.contains("VERSION") && split.len() >= 3 {
                            version = Some(String::from(split[2]));
                        }
                    }
                } else if curr_shader.is_empty() {
                    curr_shader = line;
                } else {
                    // ignore empty lines and comments
                    if !line.is_empty() && !line.starts_with("//") {
                        curr_shader = format!("{}\n{}", &curr_shader, &line);
                        line_mapping.push(idx);
                    }
                }
            }
        }
    }

    // compile last shader
    if let Some(kind) = shader_type {
        compile_shader(
            &curr_shader,
            &path,
            &options,
            kind,
            line_mapping,
            &output_path,
            &version,
        )?;
    }
    Ok(())
}

/// Compiles a single shader
fn compile_shader(
    curr_shader: &str,
    path: &Path,
    options: &shaderc::CompileOptions,
    kind: shaderc::ShaderKind,
    line_mapping: Vec<usize>,
    output_path: &Path,
    version: &Option<String>,
) -> Result<(), CompilerError> {
    // add version to curr_shader
    let curr_shader: String = if let Some(version) = version {
        format!("#version {}\n{}", version, curr_shader)
    } else {
        String::from(curr_shader)
    };

    debug!("Compiling:\n{}", &curr_shader);

    // compile
    let mut compiler = shaderc::Compiler::new().unwrap();
    let out = compiler
        .compile_into_spirv(
            &curr_shader,
            kind,
            &path.to_str().unwrap(),
            "main",
            Some(&options),
        )
        .map_err(|e| {
            // replaces error lines from what the parser saw to what is actually used in the input file
            let err = e.to_string();
            let captures = REG
                .captures(&err)
                .expect("Failed error translation: regex failed");
            let first_capture = captures
                .get(1)
                .expect("Failed error translation: no capture found")
                .as_str();

            let old_line: usize = first_capture.parse().unwrap_or_else(|_| {
                panic!(
                    "Failed error translation: capture not usize: {}",
                    first_capture
                )
            });

            CompilerError::Compilation(str::replace(
                &e.to_string(),
                &format!(":{}:", old_line),
                &format!(
                    ":{}:",
                    line_mapping.get(old_line - 1).unwrap_or_else(|| panic!(
                        "Failed error translation: couldn't find line mapping: {}",
                        old_line
                    ))
                ),
            ))
        })?;

    if out.get_num_warnings() != 0 {
        warn!("{}", out.get_warning_messages());
    }

    // save CompliationArtifact
    let output_folder = path.file_stem().expect("Invalid path").to_str().unwrap();
    let output_extension = get_shader_kind_extension(kind).expect("Invalid output file extension");
    let p = output_path.join(format!("{}-{}.spv", output_folder, output_extension));
    std::fs::write(p, out.as_binary_u8()).expect("Unable to write file");
    Ok(())
}

/// Converts a &str to shaderc::ShaderKind
pub fn parse_shader_kind(identifier: &str) -> Option<shaderc::ShaderKind> {
    use shaderc::ShaderKind::*;
    Some(match identifier {
        "VERTEX" => Vertex,
        "FRAGMENT" => Fragment,
        "GEOMETRY" => Geometry,
        _ => {
            return None;
        }
    })
}

/// Converts a &str to shaderc::OptimizationLevel
fn parse_optimization_level(level: &str) -> Result<shaderc::OptimizationLevel, String> {
    use shaderc::OptimizationLevel::*;
    match level {
        "zero" => Ok(Zero),
        "size" => Ok(Size),
        "performance" => Ok(Performance),
        _ => Err(format!("Failed to parse optimization level: {}", level)),
    }
}

/// Converts a &str to shaderc::ShaderKind
pub fn get_shader_kind_extension(kind: shaderc::ShaderKind) -> Option<String> {
    use shaderc::ShaderKind::*;
    Some(match kind {
        Vertex => String::from("vert"),
        Fragment => String::from("frag"),
        Geometry => String::from("geo"),
        _ => {
            return None;
        }
    })
}
