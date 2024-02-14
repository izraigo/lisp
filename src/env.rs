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
        self.vars.get(name)
    }
}
