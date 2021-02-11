use super::enums;
use config::{Config, File};
use serde::{Deserialize};
#[allow(non_camel_case_types)]
use std::collections::HashMap;
use walkdir::WalkDir;
extern crate simplelog;

#[derive(Debug, Deserialize)]
pub struct ProcessRuleConfig {
    pub class: String,
    pub exe: Option<String>,
    pub user: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProcessClassConfig {
    pub niceness: Option<i64>,
    pub sched_policy: Option<enums::SchedPolicy>,
    pub sched_priority: Option<i8>,
    pub iosched_class: Option<enums::IOSchedClass>,
    pub iosched_priority: Option<i8>,
    pub oom_score: Option<i8>,
    pub cgroup: Option<String>,
    pub affinity: Option<String>,
}

trait Validate {
    fn validate(&self);
}

#[derive(Debug, Deserialize)]
pub struct CgroupConfig {
    pub cpu_quota: Option<i8>,
    pub memory_limit: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RuniceConfig {
    pub rules: Option<HashMap<String, ProcessRuleConfig>>,
    pub classes: Option<HashMap<String, ProcessClassConfig>>,
    pub cgroups: Option<HashMap<String, CgroupConfig>>,
}

pub type RulesMapping = HashMap<String, ProcessRuleConfig>;
pub type ClassesMapping = HashMap<String, ProcessClassConfig>;
pub type CgroupsMapping = HashMap<String, CgroupConfig>;

pub fn load_config() -> Config {
    let config_path = "/etc/runice/";
    info!("Loading config from {}", config_path);

    let mut config = Config::new();
    let walkdir = WalkDir::new(config_path);

    for config_path in walkdir {
        let config_path_unwrapped = config_path.unwrap();
        let path = config_path_unwrapped.path();
        if !path.is_file() {
            continue;
        }
        let path = path.to_str().unwrap();

        info!("Merging file {}", String::from(path));
        config.merge(File::with_name(path)).unwrap();
    }

    let rules: RulesMapping = config.get("rules").unwrap();
    let total_rules: usize = rules.len();

    let classes: ClassesMapping = config.get("classes").unwrap();
    let total_classes: usize = classes.len();

    let cgroups: CgroupsMapping = config.get("cgroups").unwrap();
    let total_cgroups: usize = cgroups.len();

    info!(
        "Config has been loaded successfully: {} rules, {} classes, {} cgroups",
        total_rules, total_classes, total_cgroups
    );

    config
}
