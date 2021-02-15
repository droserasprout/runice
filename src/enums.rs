#![allow(non_camel_case_types)]

use phf::phf_map;

pub type IOSchedClass = String;
pub type IOSchedClassRepr = String;
pub type SchedPolicy = String;
pub type SchedPolicyRepr = String;

pub static iosched_class_from_repr: phf::Map<&'static str, &'static str> = phf_map! {
    "none" => "0",
    "realtime" => "1",
    "best-effort" => "2",
    "idle" => "3",
};

pub static iosched_class_to_repr: phf::Map<&'static str, &'static str> = phf_map! {
    "0" => "none",
    "1" => "realtime",
    "2" => "best-effort",
    "3" => "idle",
};

pub static sched_policy_from_repr: phf::Map<&'static str, &'static str> = phf_map! {
    "normal" => "0",
    "fifo" => "1",
    "rr" => "2",
    "batch" => "3",
    "iso" => "4",
    "idle" => "5",
    "deadline" => "6",
    "other" => "99",
};

pub static sched_policy_to_repr: phf::Map<&'static str, &'static str> = phf_map! {
    "0" => "normal",
    "1" => "fifo",
    "2" => "rr",
    "3" => "batch",
    "4" => "iso",
    "5" => "idle",
    "6" => "deadline",
    "99" => "other",
};
