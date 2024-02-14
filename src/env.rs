use std::collections::HashMap;
use crate::lispErr::LispErr;
use crate::lispErr::LispErr::Runtime;
use crate::lispval::LispVal;

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

    pub fn set(&mut self, name: String, val: LispVal) -> Result<LispVal, LispErr>{
        if (self.vars.contains_key(&name.clone())){
            self.vars.insert(name, val.clone());
            Ok(val)
        } else {
            match &self.parent {
                None => Err(Runtime("Variable is not defined".to_string())),
                Some(c) => c.clone().set(name, val)
            }
        }
    }

    pub fn define(&mut self, name: String, val: LispVal) -> Result<LispVal, LispErr> {
        self.vars.insert(name, val.clone());
        Ok(val)
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
