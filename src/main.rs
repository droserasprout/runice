#![allow(clippy::print_literal)]
#[macro_use]
extern crate log;
extern crate strum;
#[macro_use]
extern crate strum_macros;
use crate::enums::IOSchedClassRepr;
use crate::enums::iosched_class_to_repr;
use crate::enums::iosched_class_from_repr;
use procfs::process::Process;
use subprocess::{Exec, Redirection};
mod config;
mod enums;
use regex::Regex;
extern crate procfs;
extern crate serde;
extern crate simplelog;
extern crate subprocess;
use simplelog::*;
use clap::*;


#[derive(Clap)]
#[clap()]
struct Opts {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn call_renice(process: &Process, niceness: i64) {
    let pid = process.pid();
    let current_niceness = process.stat.nice as i64;
    if current_niceness == niceness {
        ()
    }

    info!("renice {}: {} -> {}", pid, current_niceness, niceness);
    let command = Exec::cmd("renice")
        .arg(niceness.to_string())
        .arg("-p")
        .arg(pid.to_string());
    let _exit_status = command.join();
}

fn call_ionice(
    process: &Process,
    iosched_class_repr: Option<&IOSchedClassRepr>,
    iosched_priority: Option<i8>,
) {
    let pid = process.pid();

    let out = Exec::cmd("ionice").arg("-p").arg(pid.to_string())
        .stdout(Redirection::Pipe)
        .capture().unwrap().stdout_str();
    let out: Vec<&str> = out.trim().split(": prio ").collect::<Vec<&str>>();
    let current_iosched_class_repr = out[0];
    dbg!(current_iosched_class_repr);
    let current_iosched_class = iosched_class_from_repr[current_iosched_class_repr];
    let current_iosched_priority = out[1].as_bytes().as_ptr() as i8;

    let mut command = Exec::cmd("ionice").arg("-p").arg(pid.to_string());

    if let Some(iosched_class_repr) = iosched_class_repr {
        let iosched_class = iosched_class_from_repr[iosched_class_repr.as_str()];
        dbg!(current_iosched_class, iosched_class);
        if current_iosched_class != iosched_class {
            let current_iosched_class_repr = iosched_class_to_repr[current_iosched_class];
            info!("ionice {}: {} -> {}", pid, current_iosched_class_repr, iosched_class_repr);
            command = command.arg("-c").arg(iosched_class.to_string());
        }
    }
    if let Some(iosched_priority) = iosched_priority {
        if current_iosched_priority != iosched_priority {
            info!("ionice {}: {} -> {}", pid, current_iosched_priority, iosched_priority);
            command = command.arg("-n").arg(iosched_priority.to_string());
        }
    }
    let _exit_status = command.join();
}

fn call_schedtool(
    _process: &Process,
    _sched_policy: Option<&enums::SchedPolicyRepr>,
    _sched_priority: Option<i8>,
) {
}

fn match_process<'a>(
    process: &Process,
    rules: &config::RulesMapping,
    classes: &'a config::ClassesMapping,
) -> Option<&'a config::ProcessClassConfig> {
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
        debug!("{}", rule);

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

fn apply_class(process: &Process, class: &config::ProcessClassConfig) {
    if let Some(class_niceness) = class.niceness {
        call_renice(process, class_niceness);
    }
    if class.iosched_class.is_some() | class.iosched_priority.is_some() {
        call_ionice(process, class.iosched_class.as_ref(), class.iosched_priority);
    }
    if class.sched_policy.is_some() | class.sched_priority.is_some() {
        call_schedtool(process, class.sched_policy.as_ref(), class.sched_priority);
    }
}

fn main() {

    let opts: Opts = Opts::parse();

    let mut level_filter = LevelFilter::Info;
    match opts.verbose {
        1 => { level_filter = LevelFilter:: Debug; },
        _ => (),
    }

    SimpleLogger::init(level_filter, Config::default()).unwrap();


    info!("Initializing runice");

    let config = config::load_config();

    let rules = &config.get::<config::RulesMapping>("rules").unwrap();
    let classes = &config.get::<config::ClassesMapping>("classes").unwrap();
    let _cgroups = &config.get::<config::CgroupsMapping>("cgroups").unwrap();

    for process in procfs::process::all_processes().unwrap() {
        let matched_class = match_process(&process, rules, classes);
        if matched_class.is_some() {
            apply_class(&process, &matched_class.unwrap());
        }
    }
}
