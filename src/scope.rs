use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{eval::builtins::get_builtin, object::ContextualObject};

pub type MutScope<'a> = Rc<RefCell<Scope<'a>>>;

#[derive(Debug, Clone)]
pub struct Scope<'a> {
    store: BTreeMap<String, ContextualObject<'a>>,
    parent: Option<MutScope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new() -> MutScope<'a> {
        Rc::new(RefCell::new(Scope {
            store: BTreeMap::new(),
            parent: None,
        }))
    }

    pub fn new_child(parent: MutScope) -> MutScope {
        Rc::new(RefCell::new(Scope {
            store: BTreeMap::new(),
            parent: Some(parent),
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
        self.store.insert(ident.to_string(), obj);
    }

    pub fn list_vars(&self) -> Vec<String> {
        let mut vars = self.store.keys().cloned().collect::<Vec<String>>();
        match &self.parent {
            Some(parent) => vars.extend(parent.borrow().list_vars()),
            None => (),
        }
        vars
    }
}
