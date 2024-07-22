use std::{
    collections::BTreeMap,
    env,
    sync::{Arc, RwLock},
    thread,
};

use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};

use crate::{
    ast::{ContextualNode, Program},
    errors::Hydrator,
    object::{ContextualObject, Object},
    scope::MutScope,
};

#[allow(non_upper_case_globals)]
pub static Microtasker: once_cell::sync::Lazy<Arc<RwLock<MicrotaskScheduler<'static>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(MicrotaskScheduler::new())));

pub enum MicrotaskInstruction {
    Task(Vec<ContextualNode<'static>>, MutScope<'static>, Hydrator),
    Result(ContextualObject<'static>),
}

#[derive(Clone)]
pub struct MicrotaskChannel {
    pub tx: Arc<UnboundedSender<MicrotaskInstruction>>,
    pub rx: Arc<Mutex<UnboundedReceiver<MicrotaskInstruction>>>,
    pub id: Arc<RwLock<String>>,
}

impl MicrotaskChannel {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();

        let mtc = Self {
            tx: Arc::new(tx),
            rx: Arc::new(Mutex::new(rx)),
            id: Arc::new(RwLock::new(String::new())),
        };

        thread::spawn({
            let mtc = mtc.clone();
            || async move {
                loop {
                    match mtc.rx.clone().lock().await.recv().await {
                        Some(MicrotaskInstruction::Task(nodes, scope, hydrator)) => {
                            let program = Program::from((nodes, hydrator));
                            let result = program.eval(Some(scope)).unwrap();
                            mtc.tx
                                .clone()
                                .send(MicrotaskInstruction::Result(result))
                                .unwrap();
                        }
                        _ => {}
                    }
                }
            }
        });

        mtc
    }

    pub fn queue_microtask<'a>(
        &self,
        task: Vec<ContextualNode<'a>>,
        scope: MutScope<'a>,
        hydrator: Hydrator,
    ) -> ContextualObject<'a> {
        let uuid = uuid::Uuid::new_v4().to_string();

        let task = Box::leak(Box::new(task)).to_owned();
        let task = unsafe {
            std::mem::transmute::<Vec<ContextualNode<'a>>, Vec<ContextualNode<'static>>>(task)
        };

        let scope = Box::leak(Box::new(scope)).to_owned();
        let scope = unsafe { std::mem::transmute::<MutScope<'a>, MutScope<'static>>(scope) };
        *self.id.write().unwrap() = uuid.clone();

        self.tx
            .clone()
            .send(MicrotaskInstruction::Task(task, scope, hydrator))
            .unwrap();

        Object::Promise(String::new(), uuid).anonymous()
    }
}

pub struct MicrotaskScheduler<'a> {
    pub channels: Vec<MicrotaskChannel>,
    pub queue: Vec<(Vec<ContextualNode<'a>>, MutScope<'a>, Hydrator)>,
    pub free_channels: RwLock<Vec<usize>>,
    pub promises: BTreeMap<String, ContextualObject<'a>>,
}

impl<'a> MicrotaskScheduler<'a> {
    pub fn new() -> Self {
        let count: usize = env::var("PET_MT_THREADS")
            .unwrap_or("0".to_string())
            .parse()
            .expect("Invalid PET_MT_THREADS value");

        let mut channels = vec![];
        for _ in 0 as usize..count {
            channels.push(MicrotaskChannel::new());
        }

        Self {
            channels,
            queue: vec![],
            free_channels: RwLock::new((0..count).collect()),
            promises: BTreeMap::new(),
        }
    }

    pub fn tick(&mut self) {
        if self.free_channels.read().unwrap().is_empty() || self.queue.is_empty() {
            return;
        }

        let channel = self.free_channels.write().unwrap().pop().unwrap();
        let (task, scope, hydrator) = self.queue.pop().unwrap();

        self.channels[channel].queue_microtask(task, scope, hydrator);
    }

    pub fn wait(&mut self, id: String) -> ContextualObject<'a> {
        loop {
            if let Some(result) = self.promises.remove(&id) {
                return result;
            }

            self.tick();
        }
    }
}
