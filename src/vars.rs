//! Variable storage with dynamic scoping and copy-on-write scopes (cheap subshell clones).

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

pub const EXPORT: u32 = 1;
pub const READONLY: u32 = 2;
pub const INTEGER: u32 = 4;
pub const LOWER: u32 = 8;
pub const UPPER: u32 = 16;
pub const NAMEREF: u32 = 32;
pub const ARRAY: u32 = 64;
pub const ASSOC: u32 = 128;
pub const CAPITALIZE: u32 = 256;
pub const TRACE: u32 = 512;

#[derive(Clone, Debug, Default)]
pub struct AssocMap {
    keys: Vec<String>,
    map: HashMap<String, String>,
}

impl AssocMap {
    pub fn get(&self, k: &str) -> Option<&String> {
        self.map.get(k)
    }
    pub fn insert(&mut self, k: String, v: String) {
        if !self.map.contains_key(&k) {
            self.keys.push(k.clone());
        }
        self.map.insert(k, v);
    }
    pub fn remove(&mut self, k: &str) {
        if self.map.remove(k).is_some() {
            self.keys.retain(|x| x != k);
        }
    }
    pub fn len(&self) -> usize {
        self.keys.len()
    }
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.keys.iter()
    }
    pub fn values(&self) -> impl Iterator<Item = &String> {
        self.keys.iter().map(move |k| &self.map[k])
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.keys.iter().map(move |k| (k, &self.map[k]))
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Str(String),
    Arr(BTreeMap<i64, String>),
    Assoc(AssocMap),
}

impl Value {
    pub fn scalar(&self) -> String {
        match self {
            Value::Str(s) => s.clone(),
            Value::Arr(a) => a.get(&0).cloned().unwrap_or_default(),
            Value::Assoc(m) => m.get("0").cloned().unwrap_or_default(),
        }
    }
    pub fn values(&self) -> Vec<String> {
        match self {
            Value::Str(s) => vec![s.clone()],
            Value::Arr(a) => a.values().cloned().collect(),
            Value::Assoc(m) => m.values().cloned().collect(),
        }
    }
    pub fn keys(&self) -> Vec<String> {
        match self {
            Value::Str(_) => vec!["0".into()],
            Value::Arr(a) => a.keys().map(|k| k.to_string()).collect(),
            Value::Assoc(m) => m.keys().cloned().collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Var {
    pub value: Option<Value>,
    pub attrs: u32,
}

impl Var {
    pub fn new(v: Value, attrs: u32) -> Var {
        Var { value: Some(v), attrs }
    }
    pub fn str(s: impl Into<String>) -> Var {
        Var { value: Some(Value::Str(s.into())), attrs: 0 }
    }
    pub fn has(&self, a: u32) -> bool {
        self.attrs & a != 0
    }
}

pub type Scope = HashMap<String, Var>;

#[derive(Clone, Default)]
pub struct Vars {
    pub global: Arc<Scope>,
    pub frames: Vec<Arc<Scope>>,
}

impl Vars {
    pub fn get(&self, name: &str) -> Option<&Var> {
        for f in self.frames.iter().rev() {
            if let Some(v) = f.get(name) {
                return Some(v);
            }
        }
        self.global.get(name)
    }

    /// Mutable access to the variable wherever it is visible; created in global scope if absent.
    pub fn get_mut_or_create(&mut self, name: &str) -> &mut Var {
        let mut idx = None;
        for (i, f) in self.frames.iter().enumerate().rev() {
            if f.contains_key(name) {
                idx = Some(i);
                break;
            }
        }
        let scope = match idx {
            Some(i) => Arc::make_mut(&mut self.frames[i]),
            None => Arc::make_mut(&mut self.global),
        };
        scope.entry(name.to_string()).or_insert(Var { value: None, attrs: 0 })
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Var> {
        let mut idx = None;
        for (i, f) in self.frames.iter().enumerate().rev() {
            if f.contains_key(name) {
                idx = Some(i);
                break;
            }
        }
        match idx {
            Some(i) => Arc::make_mut(&mut self.frames[i]).get_mut(name),
            None => {
                if self.global.contains_key(name) {
                    Arc::make_mut(&mut self.global).get_mut(name)
                } else {
                    None
                }
            }
        }
    }

    /// Declare in the innermost scope (function-local if in a function).
    pub fn local_mut(&mut self, name: &str) -> &mut Var {
        let scope = match self.frames.last_mut() {
            Some(f) => Arc::make_mut(f),
            None => Arc::make_mut(&mut self.global),
        };
        scope.entry(name.to_string()).or_insert(Var { value: None, attrs: 0 })
    }

    pub fn is_local_here(&self, name: &str) -> bool {
        match self.frames.last() {
            Some(f) => f.contains_key(name),
            None => self.global.contains_key(name),
        }
    }

    pub fn global_mut(&mut self, name: &str) -> &mut Var {
        Arc::make_mut(&mut self.global).entry(name.to_string()).or_insert(Var { value: None, attrs: 0 })
    }

    /// Remove the innermost visible binding.
    pub fn unset(&mut self, name: &str) -> bool {
        for i in (0..self.frames.len()).rev() {
            if self.frames[i].contains_key(name) {
                Arc::make_mut(&mut self.frames[i]).remove(name);
                return true;
            }
        }
        if self.global.contains_key(name) {
            Arc::make_mut(&mut self.global).remove(name);
            return true;
        }
        false
    }

    pub fn push_frame(&mut self) {
        self.frames.push(Arc::new(HashMap::new()));
    }

    pub fn push_frame_with(&mut self, s: Scope) {
        self.frames.push(Arc::new(s));
    }

    pub fn pop_frame(&mut self) {
        self.frames.pop();
    }

    /// All visible variable names (sorted).
    pub fn names(&self) -> Vec<String> {
        let mut set: std::collections::BTreeSet<String> = self.global.keys().cloned().collect();
        for f in &self.frames {
            for k in f.keys() {
                set.insert(k.clone());
            }
        }
        set.into_iter().collect()
    }

    /// Visible (name, var) pairs, sorted by name.
    pub fn visible(&self) -> Vec<(String, Var)> {
        self.names().into_iter().filter_map(|n| self.get(&n).map(|v| (n, v.clone()))).collect()
    }
}
