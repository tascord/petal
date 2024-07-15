use std::{cell::RefCell, collections::BTreeMap, rc::Rc, sync::Arc};

use pest::Span;

use crate::{
    errors::{Error, Hydrator},
    eval::{
        builtins::get_builtin,
        intrinsics::{get_intrinsic, list_instrinsics},
    },
    object::{ContextualObject, Object},
};

pub type MutScope<'a> = Arc<RefCell<Scope<'a>>>;

#[derive(Debug, Clone)]
pub struct Scope<'a> {
    pub name: String,
    store: BTreeMap<String, ContextualObject<'a>>,
    parent: Option<MutScope<'a>>,
    slf: Option<ContextualObject<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(name: &str) -> MutScope<'a> {
        Arc::new(RefCell::new(Scope {
            name: name.to_string(),
            store: BTreeMap::new(),
            parent: None,
            slf: None,
        }))
    }

    pub fn new_child(parent: MutScope<'a>, name: &str) -> MutScope<'a> {
        Arc::new(RefCell::new(Scope {
            name: name.to_string(),
            store: BTreeMap::new(),
            parent: Some(parent),
            slf: None,
        }))
    }

    pub fn get(&self, ident: &str) -> Option<ContextualObject<'a>> {
        match self.store.get(ident) {
            Some(obj) => Some(obj.clone()),
            None => match &self.parent.as_ref().and_then(|a| match self.name.as_str() {
                "object" => None,
                _ => Some(a),
            }) {
                Some(parent) => parent.borrow().get(ident),
                None => get_builtin(ident),
            },
        }
    }

    pub fn set(&mut self, ident: &str, obj: ContextualObject<'a>, s: Span, h: Hydrator) -> Result<(), Error> {
        if self.store.contains_key(ident) {
            return Err(partial!(
                "setting variable",
                format!("Variable {} already exists", ident),
                "You may have meant to reassign the variable, in which case you should use the `=` operator".to_string(),
                s, 
                h.clone()
            ));
        }

        (*self).store.insert(ident.to_string(), obj);
        Ok(())
    }

    pub fn assign(&mut self, ident: &str, obj: ContextualObject<'a>, s: Span, h: Hydrator) -> Result<(), Error> {
        match self.store.get_mut(ident) {
            Some(o) => {
                *o = obj;
                Ok(())
            }
            None => match &self.parent {
                Some(parent) => parent.borrow_mut().assign(ident, obj, s, h),
                None => Err(partial!(
                    "assigning variable",
                    format!("Variable {} does not exist", ident),
                    "You may have meant to declare a variable with the 'let' keyword.".to_string(),
                    s,
                    h.clone()
                )),
            },
        }
    }

    pub fn force_set(&mut self, ident: &str, obj: ContextualObject<'a>) {
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

    pub fn new_from_object(
        o: ContextualObject<'a>,
        parent: MutScope<'a>,
    ) -> Result<MutScope<'a>, Error> {
        let mut scope = Scope {
            name: "object".to_string(),
            store: BTreeMap::new(),
            parent: Some(parent.clone()),
            slf: Some(o.clone()),
        };

        match &o.0 {
            Object::Array(a) => {
                for (i, obj) in a.iter().enumerate() {
                    scope.force_set(&i.to_string(), obj.clone());
                }
            }
            Object::Map(m) => {
                for (k, v) in m.iter() {
                    scope.force_set(&k.0.to_string(), v.clone());
                }
            }
            Object::String(s) => {
                for (i, c) in s.chars().enumerate() {
                    scope.force_set(&i.to_string(), Object::String(c.to_string()).anonymous());
                }
            }
            _ => {}
        }

        for intrinsic in list_instrinsics(&o.0.typed()) {
            scope.force_set(intrinsic, get_intrinsic(intrinsic).unwrap());
        }

        Ok(Arc::new(RefCell::new(scope.clone())))
    }

    // This is just for nider debugging in the repl
    pub fn to_object(&self) -> ContextualObject<'a> {
        Object::Map({
            let mut m = BTreeMap::new();
            m.insert(string("name"), string(&self.name));
            m.insert(
                string("parent"),
                match self.parent.clone() {
                    Some(p) => Object::String(p.borrow().name.clone()).anonymous(),
                    None => string("none"),
                },
            );

            m.insert(
                string("self"),
                self.slf.clone().unwrap_or(Object::Null.anonymous()),
            );
            m.insert(
                string("store"),
                Object::Map({
                    self.store
                        .clone()
                        .iter()
                        .map(|(k, v)| (string(k.as_str()), v.clone()))
                        .collect::<BTreeMap<ContextualObject<'a>, ContextualObject<'a>>>()
                })
                .anonymous(),
            );

            m
        })
        .anonymous()
    }
}

fn string<'a>(s: &str) -> ContextualObject<'a> {
    Object::String(s.to_string()).anonymous()
}
