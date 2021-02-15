use super::enums;
use crate::enums::IOSchedClassRepr;
use crate::enums::SchedPolicyRepr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use walkdir::WalkDir;
extern crate simplelog;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
#[macro_use]
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnanicyRuleConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnanicyTypeConfig {
    #[serde(rename = "type")]
    pub type_field: String,
    pub nice: Option<i8>,
    pub ioclass: Option<String>,
    pub ionice: Option<i8>,
    pub cgroup: Option<String>,
    pub sched: Option<String>,
    pub oom_score_adj: Option<i16>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnanicyCgroupConfig {
    pub cgroup: String,
    pub CPUQuota: String,
}

#[skip_serializing_none]
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

#[skip_serializing_none]
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RuniceClassConfig {
    pub niceness: Option<i8>,
    pub sched_policy: Option<SchedPolicyRepr>,
    pub sched_priority: Option<u32>,
    pub iosched_class: Option<IOSchedClassRepr>,
    pub iosched_priority: Option<i8>,
    pub oom_score_adj: Option<i16>,
    pub cgroup: Option<String>,
    pub affinity: Option<String>,
}

trait Validate {
    fn validate(&self);
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RuniceCgroupConfig {
    pub cpu_quota: Option<i8>,
    pub memory_limit: Option<String>,
}

#[skip_serializing_none]
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

    // let cgroups: CgroupsMapping = config.get("cgroups").unwrap();
    // let total_cgroups: usize = cgroups.len();
    let total_cgroups: usize = 0;

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
        match File::create(&runice_config_path) {
            _ => (),
        }
        let runice_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&runice_config_path)
            .unwrap();

        let ananicy_file = File::open(ananicy_config_path_obj).unwrap();
        let mut ananicy_reader = BufReader::new(ananicy_file);

        let mut runice_config: RuniceConfig = RuniceConfig {
            rules: None,
            classes: None,
            cgroups: None,
        };

        let mut ananicy_config_items = String::new();
        ananicy_reader
            .read_to_string(&mut ananicy_config_items)
            .unwrap();
        let ananicy_config_items: Vec<String> = ananicy_config_items
            .lines()
            .map(|line| line.split_whitespace().collect())
            .collect();
        let ananicy_config_items: Vec<&String> = ananicy_config_items
            .iter()
            .filter(|line| !line.starts_with("#") & (line.len() != 0))
            .collect();

        match anaicy_extension {
            "rules" => {
                let ananicy_config_items: Vec<AnanicyRuleConfig> = ananicy_config_items
                    .iter()
                    .map(|item| serde_json::from_str(item))
                    .filter_map(|item| {
                        item.unwrap_or({
                            warn!("skipped invalid item");
                            None
                        })
                    })
                    .collect();
                let mut rules_hashmap: HashMap<String, RuniceRuleConfig> = HashMap::new();

                for ananicy_config_item in ananicy_config_items {
                    let rule_config = RuniceRuleConfig {
                        class: ananicy_config_item.type_field,
                        name: Some(ananicy_config_item.name.clone()),
                        exe: None,
                        cmdline: None,
                        user: None,
                    };
                    rules_hashmap.insert(ananicy_config_item.name.clone(), rule_config);
                }
                runice_config.rules = Some(rules_hashmap.clone());
                serde_yaml::to_writer(&runice_file, &runice_config).unwrap();
            }
            "types" => {
                let ananicy_config_items: Vec<AnanicyTypeConfig> = ananicy_config_items
                    .iter()
                    .map(|item| serde_json::from_str(item))
                    .filter_map(|item| {
                        item.unwrap_or({
                            warn!("skipped invalid item");
                            None
                        })
                    })
                    .collect();
                let mut classes_hashmap: HashMap<String, RuniceClassConfig> = HashMap::new();

                for ananicy_config_item in ananicy_config_items {
                    let class_config = RuniceClassConfig {
                        niceness: ananicy_config_item.nice,
                        sched_policy: ananicy_config_item.sched,
                        sched_priority: None,
                        iosched_class: ananicy_config_item.ioclass,
                        iosched_priority: ananicy_config_item.ionice,
                        oom_score_adj: ananicy_config_item.oom_score_adj,
                        cgroup: ananicy_config_item.cgroup,
                        affinity: None,
                    };
                    classes_hashmap.insert(ananicy_config_item.type_field.clone(), class_config);
                }
                runice_config.classes = Some(classes_hashmap.clone());
                serde_yaml::to_writer(&runice_file, &runice_config).unwrap();
            }
            "cgroups" => {}
            _ => {
                continue;
            }
        }
    }
}
