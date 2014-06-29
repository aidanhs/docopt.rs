#![crate_id = "docopt#0.1.0"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![license = "UNLICENSE"]
#![doc(html_root_url = "http://burntsushi.net/rustdoc/docopt")]

#![allow(dead_code, unused_variable, unused_imports)]
#![allow(visible_private_types)]

//! Docopt for Rust.

#![feature(macro_rules, phase)]

extern crate debug;
extern crate libc;
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

use std::collections::HashMap;
use std::fmt;
use parse::Parser;
use synonym::SynonymMap;

pub type Error = String;

#[deriving(Show)]
pub struct Docopt {
    p: Parser,
    conf: Config,
}

#[deriving(Show)]
pub struct Config {
    pub options_first: bool,
    pub help: bool,
    pub version: Option<String>,
}

pub fn docopt(doc: &str) -> ValueMap {
    use std::io;
    use std::os;

    let conf = Config { options_first: false, help: true, version: None };
    let dopt =
        match Docopt::new(doc, conf) {
            Ok(dopt) => dopt,
            Err(err) => fail!("{}", err),
        };
    let os_argv = os::args();
    let argv: Vec<&str> = os_argv.iter().skip(1).map(|s|s.as_slice()).collect();
    dopt.argv(argv.as_slice()).unwrap_or_else(|err| {
        if !err.is_empty() {
            let _ = writeln!(io::stderr(), "{}\n", err);
        }
        let _ = writeln!(io::stderr(), "{}", dopt.p.usage.as_slice().trim());
        unsafe { libc::exit(1 as libc::c_int); }
    })
}

impl Docopt {
    pub fn new(doc: &str, conf: Config) -> Result<Docopt, Error> {
        Ok(Docopt {
            p: try!(Parser::new(doc, conf.options_first)),
            conf: conf,
        })
    }

    pub fn argv(&self, args: &[&str]) -> Result<ValueMap, Error> {
        let argv = try!(self.p.parse_argv(args));
        self.p.matches(&argv)
            .map(|m| Ok(ValueMap { map: m }))
            .unwrap_or(Err("".to_string()))
    }
}

#[deriving(Clone)]
pub struct ValueMap {
    map: SynonymMap<String, Value>,
}

#[deriving(Clone, PartialEq, Show)]
pub enum Value {
    Switch(bool),
    Counted(uint),
    Plain(Option<String>),
    List(Vec<String>),
}

impl ValueMap {
    pub fn get_bool(&self, key: &str) -> bool {
        self.find(&key).map(|v| v.as_bool()).unwrap_or(false)
    }
    pub fn get_count(&self, key: &str) -> uint {
        self.find(&key).map(|v| v.as_count()).unwrap_or(0)
    }
    pub fn get_str<'a>(&'a self, key: &str) -> &'a str {
        self.find(&key).map(|v| v.as_str()).unwrap_or("")
    }
    pub fn get_vec<'a>(&'a self, key: &str) -> Vec<&'a str> {
        self.find(&key).map(|v| v.as_vec()).unwrap_or(vec!())
    }
}

impl Value {
    pub fn as_bool(&self) -> bool {
        match self {
            &Switch(b) => b,
            &Counted(n) => n > 0,
            &Plain(None) => false,
            &Plain(Some(_)) => true,
            &List(ref vs) => !vs.is_empty(),
        }
    }
    pub fn as_count(&self) -> uint {
        match self {
            &Switch(b) => if b { 1 } else { 0 },
            &Counted(n) => n,
            &Plain(None) => 0,
            &Plain(Some(_)) => 1,
            &List(ref vs) => vs.len(),
        }
    }
    pub fn as_str<'a>(&'a self) -> &'a str {
        match self {
            &Switch(_) | &Counted(_) | &Plain(None) | &List(_) => "",
            &Plain(Some(ref s)) => s.as_slice(),
        }
    }
    pub fn as_vec<'a>(&'a self) -> Vec<&'a str> {
        match self {
            &Switch(_) | &Counted(_) | &Plain(None) => vec!(),
            &Plain(Some(ref s)) => vec!(s.as_slice()),
            &List(ref vs) => vs.iter().map(|s| s.as_slice()).collect(),
        }
    }
}

impl Collection for ValueMap {
    fn len(&self) -> uint { self.map.len() }
}

impl<'k> Map<&'k str, Value> for ValueMap {
    fn find<'a>(&'a self, key: & &'k str) -> Option<&'a Value> {
        self.map.find(&key.to_string())
    }
}

impl fmt::Show for ValueMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // This is a little crazy, but we want to group synonyms with
        // their keys and sort them for predictable output.
        let reverse: HashMap<&String, &String> =
            self.map.synonyms().map(|(from, to)| (to, from)).collect();
        let mut keys: Vec<&String> = self.map.keys().collect();
        keys.sort();
        let mut first = true;
        for &k in keys.iter() {
            if !first { try!(write!(f, "\n")); } else { first = false; }
            match reverse.find(&k) {
                None => {
                    try!(write!(f, "{} => {}", k, self.map.get(k)))
                }
                Some(s) => {
                    try!(write!(f, "{}, {} => {}", s, k, self.map.get(k)))
                }
            }
        }
        Ok(())
    }
}

mod parse;
mod synonym;
#[cfg(test)]
mod test;
