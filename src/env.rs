use std::collections::HashMap;
use crate::parser::LispVal;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Closure {
    parent: Option<Box<Closure>>,
    vars: HashMap<String, LispVal>,
}

impl Closure {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn child(&self) -> Self {
        Closure { parent: Some(Box::new(self.clone())), vars: HashMap::new() }
    }

    pub fn set(&mut self, name: String, val: LispVal) -> Option<LispVal>{
        self.vars.insert(name, val)
    }

    pub fn get(&self, name: &String) -> Option<&LispVal>{
        let res = &self.vars.get(name);
        if res.is_some() {
            return *res;
        }
        match &self.parent {
            Some(parent) => parent.get(name),
            None => None
        }
    }
}
