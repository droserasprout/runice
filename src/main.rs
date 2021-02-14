#![allow(clippy::print_literal)]
#[macro_use]
extern crate log;
extern crate strum;
#[macro_use]
extern crate strum_macros;
use procfs::process::Process;
use subprocess::Exec;
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


fn call_renice(pid: i32, niceness: i64) {
    let command = Exec::cmd("renice")
        .arg(niceness.to_string())
        .arg("-p")
        .arg(pid.to_string());
    let _exit_status = command.join();
}

fn call_ionice(
    pid: i32,
    iosched_class: Option<&enums::IOSchedClass>,
    iosched_priority: Option<i8>,
) {
    let mut command = Exec::cmd("ionice").arg("-p").arg(pid.to_string());
    if iosched_class.is_some() {
        let iosched_class_value = iosched_class.unwrap().to_string();
        command = command.arg("-c").arg(iosched_class_value);
    }
    if iosched_priority.is_some() {
        let ionice_value = iosched_priority.unwrap().to_string();
        command = command.arg("-n").arg(ionice_value);
    }
    let _exit_status = command.join();
}

fn call_schedtool(
    _pid: i32,
    _sched_policy: Option<&enums::SchedPolicy>,
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

fn apply_class(pid: i32, class: &config::ProcessClassConfig) {
    if let Some(class_niceness) = class.niceness {
        call_renice(pid, class_niceness);
    }
    if class.iosched_class.is_some() | class.iosched_priority.is_some() {
        call_ionice(pid, class.iosched_class.as_ref(), class.iosched_priority);
    }
    if class.sched_policy.is_some() | class.sched_priority.is_some() {
        call_schedtool(pid, class.sched_policy.as_ref(), class.sched_priority);
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

    for prc in procfs::process::all_processes().unwrap() {
        let matched_class = match_process(&prc, rules, classes);
        if matched_class.is_some() {
            apply_class(prc.pid(), &matched_class.unwrap());
        }
    }
}
