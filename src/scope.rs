use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{
    errors::{Error, Hydrator},
    eval::{
        builtins::get_builtin,
        intrinsics::{get_intrinsic, list_instrinsics},
    },
    object::{ContextualObject, Object},
};

pub type MutScope<'a> = Rc<RefCell<Scope<'a>>>;

#[derive(Debug, Clone)]
pub struct Scope<'a> {
    pub name: String,
    store: BTreeMap<String, ContextualObject<'a>>,
    parent: Option<MutScope<'a>>,
    slf: Option<ContextualObject<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(name: &str) -> MutScope<'a> {
        Rc::new(RefCell::new(Scope {
            name: name.to_string(),
            store: BTreeMap::new(),
            parent: None,
            slf: None,
        }))
    }

    pub fn new_child(parent: MutScope<'a>, name: &str) -> MutScope<'a> {
        Rc::new(RefCell::new(Scope {
            name: name.to_string(),
            store: BTreeMap::new(),
            parent: Some(parent),
            slf: None,
        }))
    }

    pub fn get(&self, ident: &str) -> Option<ContextualObject<'a>> {
        match self.store.get(ident) {
            Some(obj) => Some(obj.clone()),
            None => match &self.parent {
                Some(parent) => parent.borrow().get(ident),
                None => get_builtin(ident),
            },
        }
    }

    pub fn set(&mut self, ident: &str, obj: ContextualObject<'a>) {
        (*self).store.insert(ident.to_string(), obj);
    }

    pub fn get_self(&self) -> Option<ContextualObject<'a>> {
        match &self.slf {
            Some(obj) => Some(obj.clone()),
            None => match &self.parent {
                Some(parent) => parent.borrow().get_self(),
                None => None,
            },
        }
    }

    pub fn list_vars(&self) -> Vec<String> {
        let mut vars = self.store.keys().cloned().collect::<Vec<String>>();
        match &self.parent {
            Some(parent) => vars.extend(parent.borrow().list_vars()),
            None => (),
        }
        vars
    }

    pub fn new_from_object(o: ContextualObject<'a>, _h: Hydrator) -> Result<MutScope<'a>, Error> {
        let mut scope = Scope {
            name: "object".to_string(),
            store: BTreeMap::new(),
            parent: None,
            slf: Some(o.clone()),
        };

        match &o.0 {
            Object::Array(a) => {
                for (i, obj) in a.iter().enumerate() {
                    scope.set(&i.to_string(), obj.clone());
                }
            }
            Object::Map(m) => {
                for (k, v) in m.iter() {
                    scope.set(&k.0.to_string(), v.clone());
                }
            }
            Object::String(s) => {
                for (i, c) in s.chars().enumerate() {
                    scope.set(&i.to_string(), Object::String(c.to_string()).anonymous());
                }
            }
            _ => {}
        }

        for intrinsic in list_instrinsics(&o.0.typed()) {
            scope.set(intrinsic, get_intrinsic(intrinsic).unwrap());
        }

        Ok(Rc::new(RefCell::new(scope.clone())))
    }
}
