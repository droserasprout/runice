use super::enums;
use crate::enums::IOSchedClassRepr;
use crate::enums::SchedPolicyRepr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use walkdir::WalkDir;
extern crate simplelog;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufReader;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnanicyRuleConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnanicyTypeConfig {
    #[serde(rename = "type")]
    pub type_field: String,
    pub nice: Option<String>,
    pub ioclass: Option<String>,
    pub ionice: Option<String>,
    pub cgroup: Option<String>,
    pub sched: Option<String>,
    pub oom_score_adj: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnanicyCgroupConfig {
    pub cgroup: String,
    pub CPUQuota: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RuniceRuleConfig {
    pub class: String,
    pub name: Option<String>,
    pub exe: Option<String>,
    pub cmdline: Option<String>,
    pub user: Option<String>,
}

impl std::fmt::Display for RuniceRuleConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(
            f,
            "class={}, name={}, exe={}, cmdline={}, user={}",
            self.class,
            self.name.as_ref().unwrap_or(&String::from("")),
            &self.exe.as_ref().unwrap_or(&String::from("")),
            &self.cmdline.as_ref().unwrap_or(&String::from("")),
            &self.user.as_ref().unwrap_or(&String::from("")),
        )
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RuniceClassConfig {
    pub niceness: Option<i64>,
    pub sched_policy: Option<SchedPolicyRepr>,
    pub sched_priority: Option<i8>,
    pub iosched_class: Option<IOSchedClassRepr>,
    pub iosched_priority: Option<i8>,
    pub oom_score: Option<i8>,
    pub cgroup: Option<String>,
    pub affinity: Option<String>,
}

trait Validate {
    fn validate(&self);
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RuniceCgroupConfig {
    pub cpu_quota: Option<i8>,
    pub memory_limit: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RuniceConfig {
    pub rules: Option<HashMap<String, RuniceRuleConfig>>,
    pub classes: Option<HashMap<String, RuniceClassConfig>>,
    pub cgroups: Option<HashMap<String, RuniceCgroupConfig>>,
}

pub type RulesMapping = HashMap<String, RuniceRuleConfig>;
pub type ClassesMapping = HashMap<String, RuniceClassConfig>;
pub type CgroupsMapping = HashMap<String, RuniceCgroupConfig>;

pub fn load_config() -> config::Config {
    let config_path = "/etc/runice/";
    info!("Loading config from {}", config_path);

    let mut config = config::Config::new();
    let walkdir = WalkDir::new(config_path);

    for config_path in walkdir {
        let config_path_unwrapped = config_path.unwrap();
        let path = config_path_unwrapped.path();
        if !path.is_file() {
            continue;
        }
        let path = path.to_str().unwrap();

        info!("Merging file {}", String::from(path));
        config.merge(config::File::with_name(path)).unwrap();
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

pub fn import_ananicy_config() {
    let runice_config_directory = "/etc/runice/";
    let ananicy_config_directory = "/etc/ananicy.d/";

    info!("Importing Ananicy config from {}", ananicy_config_directory);

    let ananicy_config_walkdir = WalkDir::new(ananicy_config_directory);

    for ananicy_config_path_obj in ananicy_config_walkdir {
        let ananicy_config_path_obj = ananicy_config_path_obj.unwrap();
        let ananicy_config_path_obj = ananicy_config_path_obj.path();
        if !ananicy_config_path_obj.is_file() {
            continue;
        }

        let ananicy_filename = ananicy_config_path_obj.file_name().unwrap();

        let out: Vec<&str> = ananicy_filename
            .to_str()
            .unwrap()
            .split(".")
            .collect::<Vec<&str>>();
        let ananicy_name = out[0];
        let anaicy_extension = out[1];

        // FIXME: ensure path exists
        let runice_config_path = String::from(format!(
            "{}00-ananicy/{}.yml",
            runice_config_directory, ananicy_name
        ));
        File::create(&runice_config_path).unwrap();
        let runice_file = File::open(&runice_config_path).unwrap();

        let ananicy_file = File::open(ananicy_config_path_obj).unwrap();
        let ananicy_reader = BufReader::new(ananicy_file);

        let ananicy_config = serde_json::from_reader(ananicy_reader);

        let mut runice_config: RuniceConfig = RuniceConfig {
            rules: None,
            classes: None,
            cgroups: None,
        };

        match anaicy_extension {
            "rules" => {
                let ananicy_config: Vec<AnanicyRuleConfig> = ananicy_config.unwrap();

                let mut rules_hashmap: HashMap<String, RuniceRuleConfig> = HashMap::new();
                runice_config.rules = Some(rules_hashmap.clone());

                for ananicy_config_item in ananicy_config {
                    let rule_config = RuniceRuleConfig {
                        class: ananicy_config_item.type_field,
                        name: Some(ananicy_config_item.name),
                        exe: None,
                        cmdline: None,
                        user: None,
                    };
                    rules_hashmap.insert(String::from(ananicy_name), rule_config);
                }
                serde_json::to_writer(&runice_file, &runice_config).unwrap();
            }
            "types" => {}
            "cgroups" => {}
            _ => {
                continue;
            }
        }
    }
}
