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
extern crate procfs;
extern crate serde;
extern crate simplelog;
extern crate subprocess;
use simplelog::*;

fn call_renice(pid: i32, niceness: i64) {
    let command = Exec::cmd("renice")
        .arg("-p")
        .arg(pid.to_string())
        .arg("-n")
        .arg(niceness.to_string());
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

    let _cmdline = match process.cmdline() {
        Ok(cmdline) => cmdline.join(" "),
        Err(_) => "".into(),
    };

    let exe = match process.exe() {
        Ok(exe) => exe,
        Err(_) => "".into(),
    };

    let process_exe = String::from(exe.to_str().unwrap());
    let _process_stat = process.stat().unwrap();

    for (name, rule) in rules {
        debug!("{}", format!("Processing rule `{}`", name));
        let rule_exe = rule
            .exe
            .clone()
            .unwrap_or_else(|| -> String { "".to_string() });
        let rule_class = rule.class.clone();
        debug!("{}", format!("rule_exe={}, process_exe={}", rule_exe, process_exe));
        if rule_exe == process_exe {
            debug!("{}", format!("Matched by `exe`: {}", pid));
            return Some(&classes[&rule_class]);
        }
    }
    None
}

fn apply_class(pid: i32, class: &config::ProcessClassConfig) {
    if class.niceness.is_some() {
        let new_niceness = class.niceness.unwrap();
        call_renice(pid, new_niceness);
    }
    if class.iosched_class.is_some() | class.iosched_priority.is_some() {
        call_ionice(pid, class.iosched_class.as_ref(), class.iosched_priority);
    }
    if class.sched_policy.is_some() | class.sched_priority.is_some() {
        call_schedtool(pid, class.sched_policy.as_ref(), class.sched_priority);
    }
}

fn main() {
    let _ = SimpleLogger::init(LevelFilter::Debug, Config::default());

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
