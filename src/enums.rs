#![allow(non_camel_case_types)]

use phf::phf_map;

// pub type IOSchedClass = String;
pub type IOSchedClassRepr = String;
// pub type SchedPolicy = String;
pub type SchedPolicyRepr = String;

pub static IOSCHED_CLASS_FROM_REPR: phf::Map<&'static str, &'static str> = phf_map! {
    "none" => "0",
    "realtime" => "1",
    "best-effort" => "2",
    "idle" => "3",
};

pub static IOSCHED_CLASS_TO_REPR: phf::Map<&'static str, &'static str> = phf_map! {
    "0" => "none",
    "1" => "realtime",
    "2" => "best-effort",
    "3" => "idle",
};

pub static SCHED_POLICY_FROM_REPR: phf::Map<&'static str, &'static str> = phf_map! {
    "normal" => "0",
    "fifo" => "1",
    "rr" => "2",
    "batch" => "3",
    "iso" => "4",
    "idle" => "5",
    "deadline" => "6",
};

pub static SCHED_POLICY_TO_REPR: phf::Map<&'static str, &'static str> = phf_map! {
    "0" => "normal",
    "1" => "fifo",
    "2" => "rr",
    "3" => "batch",
    "4" => "iso",
    "5" => "idle",
    "6" => "deadline",
};
