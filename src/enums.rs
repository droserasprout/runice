#![allow(non_camel_case_types)]
use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use serde_repr::{Serialize_repr};
// use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Display, Serialize_repr)]
#[repr(u8)]
pub enum IOSchedClass {
    none = 0,
    realtime = 1,
    best_effort = 2,
    idle = 3,
}

struct IOSchedClassVisitor;

impl<'de> Visitor<'de> for IOSchedClassVisitor {
    type Value = IOSchedClass;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("expecting one of `none`, `realtime`, `best_effort`, `idle`")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        use IOSchedClass::*;
        let s = match v.to_lowercase().as_str() {
            "none" => none,
            "realtime" => realtime,
            "best_effort" => best_effort,
            "idle" => idle,
            a => return Err(de::Error::custom(format!("{} not supported", a))),
        };
        Ok(s)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(v.as_str())
    }
}

impl<'de> Deserialize<'de> for IOSchedClass {
    fn deserialize<D>(deserializer: D) -> Result<IOSchedClass, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(IOSchedClassVisitor)
    }
}

#[derive(Debug, Serialize_repr, Display)]
#[repr(u8)]
pub enum SchedPolicy {
    normal = 0,
    fifo = 1,
    rr = 2,
    batch = 3,
    iso = 4,
    idle = 5,
    deadline = 6,
    other = 99,
}

struct SchedPolicyVisitor;

impl<'de> Visitor<'de> for SchedPolicyVisitor {
    type Value = SchedPolicy;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("expecting one of `none`, `realtime`, `best_effort`, `idle`")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        use SchedPolicy::*;
        let s = match v.to_lowercase().as_str() {
            "normal" => normal,
            "fifo" => fifo,
            "rr" => rr,
            "batch" => batch,
            "iso" => iso,
            "idle" => idle,
            "deadline" => deadline,
            "other" => other,
            a => return Err(de::Error::custom(format!("{} not supported", a))),
        };
        Ok(s)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(v.as_str())
    }
}

impl<'de> Deserialize<'de> for SchedPolicy {
    fn deserialize<D>(deserializer: D) -> Result<SchedPolicy, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SchedPolicyVisitor)
    }
}
