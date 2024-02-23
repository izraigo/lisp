use std::collections::HashMap;
use crate::error::LispErr;
use crate::error::LispErr::Runtime;
use crate::lispval::LispVal;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Env {
    parent: Option<Rc<RefCell<Env>>>,
    vars: HashMap<String, LispVal>,
}

impl Env {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn child(parent: Rc<RefCell<Env>> ) -> Env {
        Env { parent: Some(parent), vars: HashMap::new() }
    }

    pub fn set(&mut self, name: &str, val: LispVal) -> Result<LispVal, LispErr>{
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), val.clone());
            Ok(val)
        } else {
            match &self.parent {
                None => Err(Runtime("Variable is not defined".to_string())),
                Some(c) => c.borrow_mut().set(name, val)
            }
        }
    }

    pub fn define(&mut self, name: &str, val: LispVal) -> Result<LispVal, LispErr> {
        self.vars.insert(name.to_string(), val.clone());
        Ok(val)
    }

    pub fn get(&self, name: &str) -> Option<LispVal>{
        let res = &self.vars.get(name);
        if res.is_some() {
            return res.cloned();
        }
        match &self.parent {
            Some(p) => p.borrow().get(name).clone(),
            None => None
        }
    }
}
