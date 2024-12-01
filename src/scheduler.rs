use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    component::{
        AsyncCallbackOnce, Component, ComponentType, Context, Error, RcContext, Result, Topic,
    },
    payload::RawPayload,
};

/// 全局消息计数器
static MSG_SEQ_COUNT: AtomicUsize = AtomicUsize::new(0);

fn gen_msg_seq() -> usize {
    MSG_SEQ_COUNT.fetch_add(1, Ordering::SeqCst);
    MSG_SEQ_COUNT.load(Ordering::SeqCst)
}

#[derive(Default)]
struct Scheduler {
    components: Rc<RefCell<HashMap<ComponentType, Box<dyn Component>>>>,
    async_queue: Rc<RefCell<VecDeque<AsyncQueueElement>>>,
    subscriber: Rc<RefCell<HashMap<Topic, VecDeque<ComponentType>>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<C: Component + 'static>(&self, component: C) {
        self.components
            .borrow_mut()
            .insert(component.component_type(), Box::new(component));
    }

    fn gen_ctx(&self, component_type: &ComponentType) -> RcContext {
        Rc::new(ContextImpl {
            components: self.components.clone(),
            async_queue: self.async_queue.clone(),
            subscriber: self.subscriber.clone(),
        })
    }

    pub fn schedule_once(&self) {
        if let Some(async_element) = self.async_queue.borrow_mut().pop_front() {
            if let Some(component) = self.components.borrow().get(&async_element.to) {
                if let Ok(poll) =
                    component.async_poll_handle(self.gen_ctx(&async_element.to), async_element.seq)
                {
                    match poll {
                        std::task::Poll::Ready(payload) => {
                            let callback = async_element.callback_once;
                            callback(payload);
                        }
                        std::task::Poll::Pending => {
                            self.async_queue.borrow_mut().push_back(async_element);
                        }
                    }
                }
            }
        }
    }
}

struct AsyncQueueElement {
    to: ComponentType,
    seq: usize,
    callback_once: AsyncCallbackOnce,
}

#[derive(Clone)]
struct ContextImpl {
    components: Rc<RefCell<HashMap<ComponentType, Box<dyn Component>>>>,
    async_queue: Rc<RefCell<VecDeque<AsyncQueueElement>>>,
    subscriber: Rc<RefCell<HashMap<Topic, VecDeque<ComponentType>>>>,
}

impl Context for ContextImpl {
    fn broadcast_topic(&self, topic: Topic, payload: RawPayload) {
        todo!()
    }

    fn subscribe_topic(&self, topic: Topic) {
        todo!()
    }

    fn unsubscribe_topic(&self, topic: Topic) {
        todo!()
    }

    fn sync_call(&self, component_type: ComponentType, payload: RawPayload) -> Result<RawPayload> {
        if let Some(c) = self.components.borrow().get(&component_type) {
            c.sync_call_handle(Rc::new(self.clone()), payload)
        } else {
            log::error!("not found component {:?}", component_type);
            Err(Error::ComponentNotFound(component_type))
        }
    }

    fn async_call(
        &self,
        component_type: ComponentType,
        payload: RawPayload,
        callback: AsyncCallbackOnce,
    ) -> Result<()> {
        if let Some(c) = self.components.borrow().get(&component_type) {
            let seq = gen_msg_seq();
            c.async_call_handle(Rc::new(self.clone()), seq, payload)?;
            self.async_queue.borrow_mut().push_back(AsyncQueueElement {
                to: component_type,
                seq: seq,
                callback_once: callback,
            });
            Ok(())
        } else {
            log::error!("not found component {:?}", component_type);
            Err(Error::ComponentNotFound(component_type))
        }
    }
}
