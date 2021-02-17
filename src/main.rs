#![allow(clippy::print_literal)]
#[macro_use]
extern crate log;
extern crate strum;
extern crate strum_macros;
use crate::enums::IOSchedClassRepr;
use crate::enums::IOSCHED_CLASS_FROM_REPR;
use crate::enums::IOSCHED_CLASS_TO_REPR;
use crate::enums::SCHED_POLICY_FROM_REPR;
use crate::enums::SCHED_POLICY_TO_REPR;
use procfs::process::Process;
use subprocess::NullFile;
use subprocess::{Exec, Redirection};
mod config;
mod enums;
use regex::Regex;
extern crate procfs;
extern crate serde;
extern crate simplelog;
extern crate subprocess;
use clap::*;
use simplelog::*;
use std::{thread, time};

#[derive(Clap)]
#[clap()]
struct Opts {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap()]
    Run(Run),
    #[clap()]
    ImportAnanicy(ImportAnanicy),
}

#[derive(Clap)]
struct Run {}

#[derive(Clap)]
struct ImportAnanicy {}

fn call_renice(process: &Process, niceness: i8) {
    let pid = process.pid();
    let current_niceness = process.stat.nice as i8;
    if current_niceness == niceness {
        return;
    }

    info!("renice {}: {} -> {}", pid, current_niceness, niceness);
    let command = Exec::cmd("renice")
        .arg(niceness.to_string())
        .arg("-p")
        .arg(pid.to_string());
    command.stdout(NullFile).join().unwrap();
}

fn call_ionice(
    process: &Process,
    iosched_class_repr: Option<&IOSchedClassRepr>,
    iosched_priority: Option<i8>,
) {
    let pid = process.pid();

    let out = Exec::cmd("ionice")
        .arg("-p")
        .arg(pid.to_string())
        .stdout(Redirection::Pipe)
        .capture()
        .unwrap()
        .stdout_str();
    let out: Vec<&str> = out.trim().split(": prio ").collect::<Vec<&str>>();
    let current_iosched_class_repr = out[0];
    let current_iosched_class = IOSCHED_CLASS_FROM_REPR[current_iosched_class_repr];
    let mut current_iosched_priority = 0;
    if let 2 = out.len() {
        current_iosched_priority = out[1].as_bytes().as_ptr() as i8;
    };

    let mut command = Exec::cmd("ionice").arg("-p").arg(pid.to_string());

    if let Some(iosched_class_repr) = iosched_class_repr {
        let iosched_class = IOSCHED_CLASS_FROM_REPR[iosched_class_repr.as_str()];
        if current_iosched_class != iosched_class {
            let current_iosched_class_repr = IOSCHED_CLASS_TO_REPR[current_iosched_class];
            info!(
                "ionice {}: {} -> {}",
                pid, current_iosched_class_repr, iosched_class_repr
            );
            command = command.arg("-c").arg(iosched_class.to_string());
        }
    }
    if let Some(iosched_priority) = iosched_priority {
        if current_iosched_priority != iosched_priority {
            info!(
                "ionice {}: {} -> {}",
                pid, current_iosched_priority, iosched_priority
            );
            command = command.arg("-n").arg(iosched_priority.to_string());
        }
    }
    command.stdout(NullFile).join().unwrap();
}

fn call_schedtool(
    process: &Process,
    sched_policy_repr: Option<&enums::SchedPolicyRepr>,
    sched_priority: Option<u32>,
) {
    let pid = process.pid();

    let current_sched_policy = process.stat().unwrap().policy.unwrap().to_string();
    let current_sched_priority = process.stat().unwrap().rt_priority.unwrap().to_string();

    let mut command = Exec::cmd("schedtool");

    if let Some(sched_policy_repr) = sched_policy_repr {
        let sched_policy = SCHED_POLICY_FROM_REPR[sched_policy_repr.as_str()];
        if current_sched_policy != sched_policy {
            let current_sched_policy_repr = SCHED_POLICY_TO_REPR[current_sched_policy.as_str()];
            info!(
                "schedtool {}: {} -> {}",
                pid, current_sched_policy_repr, sched_policy_repr
            );
            command = command.arg("-M").arg(sched_policy);
        }
    }

    if let Some(sched_priority) = sched_priority {
        let sched_priority = &sched_priority.to_string();
        if &current_sched_priority != sched_priority {
            info!(
                "schedtool {}: {} -> {}",
                pid, current_sched_priority, sched_priority
            );
            command = command.arg("-p").arg(&sched_priority);
        }
    }

    command = command.arg(pid.to_string());
    command.stdout(NullFile).join().unwrap();
}

fn match_process<'a>(
    process: &Process,
    rules: &config::RulesMapping,
    classes: &'a config::ClassesMapping,
) -> Option<&'a config::RuniceClassConfig> {
    let pid = process.pid;
    debug!("Trying to match PID {}", pid);

    let process_stat = match process.stat() {
        Ok(process_stat) => process_stat,
        Err(_) => return None,
    };

    let process_name = process_stat.comm;

    let process_exe = match process.exe() {
        Ok(exe) => String::from(exe.to_str().unwrap()),
        Err(_) => "".into(),
    };

    let process_cmdline = match process.cmdline() {
        Ok(cmdline) => cmdline.join(" "),
        Err(_) => "".into(),
    };

    debug!(
        "name={}, exe={}, cmdline={}",
        process_name, process_exe, process_cmdline
    );

    let mut matched_rule = None;

    for (name, rule) in rules {
        debug!("Trying rule `{}`", name);
        debug!("{:?}", rule);

        if let Some(rule_name) = &rule.name {
            if process_name == *rule_name {
                debug!("matched by name");
                matched_rule = Some(rule);
                break;
            }
        }

        if let Some(rule_exe) = &rule.exe {
            if process_exe == *rule_exe {
                debug!("matched by exe");
                matched_rule = Some(rule);
                break;
            }
        }

        if let Some(rule_cmdline) = &rule.cmdline {
            let re = Regex::new(rule_cmdline).unwrap();
            if re.is_match(process_cmdline.as_ref()) {
                debug!("matched by cmdline");
                matched_rule = Some(rule);
                break;
            }
        }
    }

    match matched_rule {
        Some(matched_rule) => Some(&classes[&matched_rule.class]),
        None => None,
    }
}

fn apply_class(process: &Process, class: &config::RuniceClassConfig) {
    if let Some(class_niceness) = class.niceness {
        call_renice(process, class_niceness);
    }
    if class.iosched_class.is_some() | class.iosched_priority.is_some() {
        call_ionice(
            process,
            class.iosched_class.as_ref(),
            class.iosched_priority,
        );
    }
    if class.sched_policy.is_some() | class.sched_priority.is_some() {
        call_schedtool(process, class.sched_policy.as_ref(), class.sched_priority);
    }
}

fn run() {
    info!("Initializing runice");

    let config = config::load_config();

    let rules = &config.get::<config::RulesMapping>("rules").unwrap();
    let classes = &config.get::<config::ClassesMapping>("classes").unwrap();
    // let _cgroups = &config.get::<config::CgroupsMapping>("cgroups").unwrap();
    let mut processed_pids: Vec<i32> = Vec::new();

    loop {
        for process in procfs::process::all_processes().unwrap() {
            let pid = process.pid;
            if processed_pids.contains(&pid) {
                continue;
            }
            processed_pids.push(pid);

            let matched_class = match_process(&process, rules, classes);
            if let Some(matched_class) = matched_class {
                apply_class(&process, &matched_class);
            }
        }
        thread::sleep(time::Duration::from_millis(1000));
    }
}

fn import_ananicy() {
    config::import_ananicy_config()
}

fn main() {
    let opts: Opts = Opts::parse();

    let mut level_filter = LevelFilter::Info;
    if opts.verbose == 1 {
        level_filter = LevelFilter::Debug;
    }

    SimpleLogger::init(level_filter, Config::default()).unwrap();

    match opts.subcmd {
        SubCommand::Run(_) => {
            run();
        }
        SubCommand::ImportAnanicy(_) => {
            import_ananicy();
        }
    }
}
