use std::collections::HashMap;
use crate::lispErr::LispErr;
use crate::lispErr::LispErr::Runtime;
use crate::lispval::LispVal;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Closure {
    parent: Option<Rc<RefCell<Closure>>>,
    vars: HashMap<String, LispVal>,
}

impl Closure {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn child(parent: Rc<RefCell<Closure>> ) -> Closure {
        Closure { parent: Some(parent), vars: HashMap::new() }
    }

    pub fn set(&mut self, name: String, val: LispVal) -> Result<LispVal, LispErr>{
        if self.vars.contains_key(&name.clone()) {
            self.vars.insert(name.clone(), val.clone());
            Ok(val)
        } else {
            match &self.parent {
                None => Err(Runtime("Variable is not defined".to_string())),
                Some(c) => c.borrow_mut().set(name, val)
            }
        }
    }

    pub fn define(&mut self, name: String, val: LispVal) -> Result<LispVal, LispErr> {
        self.vars.insert(name, val.clone());
        Ok(val)
    }

    pub fn get(&self, name: &String) -> Option<LispVal>{
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
