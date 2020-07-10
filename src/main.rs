use gumdrop::Options;
use std::path::PathBuf;
use xmltree::{Element, ElementPredicate, XMLNode, EmitterConfig};
use std::collections::hash_map::Entry;

/// Injects vars from environment to intelliJ workspace file's configuration section
#[derive(Options, Debug)]
struct InjectorOptions {
    #[options(help = "pick the environment variables that start with this string", short="s", required)]
    env_vars_start_with: String,
    #[options(help = "workspace file", short="f", required)]
    workspace_file: PathBuf,
    #[options(help = "configuration name in workspace file", short="c", required)]
    configuration_name: String,
    #[options(help = "prints this help message")]
    help: bool,
}

fn main() -> Result<(), std::io::Error> {
    let opts: InjectorOptions = InjectorOptions::parse_args_default_or_exit();
    let mut vars = Vec::new();
    for (k, v) in std::env::vars() {
        if k.starts_with(&opts.env_vars_start_with) {
            vars.push((k, v));
        }
    }

    println!("Env vars:");
    for &(ref k, ref v) in vars.iter() {
        println!("{}={}", k, v);
    }

    let path_to_file = if opts.workspace_file.is_absolute() {
        opts.workspace_file
    } else {
        std::env::current_dir().expect("current dir").join(opts.workspace_file)
    };

    let contents = match std::fs::read_to_string(&path_to_file) {
        Err(e) => panic!("Could not read workspace file at {:?}, {:?}", path_to_file, e),
        Ok(contents) => contents,
    };

    println!("workspace file found at {:?}", path_to_file);

    let mut project: Element = Element::parse(contents.as_bytes()).expect("failed to parse workspace.xml file");
    if let Some(run_manager) = project.get_mut_child(MatchTagWithName::new("component", "RunManager")) {
        if let Some(configuration) = run_manager.get_mut_child(MatchTagWithName::new("configuration", &opts.configuration_name)) {
            let envs = if let Some(envs) = configuration.get_mut_child("envs") {
                envs
            } else {
                configuration.children.insert(0, XMLNode::Element(Element::new("envs")));
                configuration.get_mut_child("envs").unwrap()
            };
            for (k, v) in vars {
                let env = if let Some(env) = envs.get_mut_child(MatchTagWithName::new("env", &k)) {
                    env
                } else {
                    let mut new_env = Element::new("env");
                    new_env.attributes.insert("name".to_owned(), k.clone());

                    envs.children.insert(0, XMLNode::Element(new_env));
                    let env = envs.get_mut_child(MatchTagWithName::new("env", &k)).unwrap();

                    env
                };

                match env.attributes.entry("value".to_owned()) {
                    Entry::Occupied(value) => { *value.into_mut() = v },
                    Entry::Vacant(empty) => { empty.insert(v); },
                }
            }
        } else {
            panic!("Failed to find \"configuration\" tag with name {:?} \
            in <component name=\"RunManager\"> tag, in {:?} file", opts.configuration_name, path_to_file);
        }
    } else {
        panic!("Failed to find <component name=\"RunManager\"> tag in {:?} file", path_to_file);
    }
    match project.write_with_config(
        std::fs::OpenOptions::new()
            .write(true)
            .create(false)
            .append(false)
            .truncate(true)
            .open(&path_to_file)?,
        EmitterConfig {
            perform_indent: true,
            .. EmitterConfig::default()
        }
    ) {
        Ok(_) => {
            println!("file updated")
        },
        Err(e) => {
            println!("failed to write to workspace file: {:?}", e);
        },
    }

    Ok(())
}

struct MatchTagWithName {
    tag: String,
    name_value: String,
}

impl MatchTagWithName {
    pub fn new(tag: &str, name: &str) -> MatchTagWithName {
        MatchTagWithName {
            tag: tag.into(),
            name_value: name.into(),
        }
    }
}

impl ElementPredicate for MatchTagWithName {
    fn match_element(&self, e: &Element) -> bool {
        e.name == self.tag && e.attributes.get("name") == Some(&self.name_value)
    }
}
