use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use uuid::Uuid;

use crate::{
    node::{
        AsyncCallID, AsyncCallbackOnce, Context, Error, MethodName, Node, NodeID, RcContext,
        Result, TopicName,
    },
    payload::RawPayload,
};

#[derive(Default)]
pub struct Scheduler {
    node_registry: Rc<RefCell<HashMap<NodeID, Box<dyn Node>>>>,
    async_queue: Rc<RefCell<VecDeque<AsyncQueueElement>>>,
    topic_subscriber_registry: Rc<RefCell<HashMap<TopicName, VecDeque<NodeID>>>>,
    topic_queue: Rc<RefCell<VecDeque<(TopicName, RawPayload)>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_node<N: Node + 'static>(&self, node: N) {
        self.node_registry
            .borrow_mut()
            .insert(node.node_id(), Box::new(node));
    }

    fn gen_ctx(&self, node_id: NodeID) -> RcContext {
        Rc::new(ContextImpl {
            node_id,
            nodes: self.node_registry.clone(),
            async_queue: self.async_queue.clone(),
            subscriber: self.topic_subscriber_registry.clone(),
            topic_queue: self.topic_queue.clone(),
        })
    }

    pub fn init(&self) {
        for (node_id, node) in self.node_registry.borrow().iter() {
            node.init(self.gen_ctx(node_id.clone()));
        }
    }

    pub fn schedule_once(&self) {
        if let Some(async_element) = self.async_queue.borrow_mut().pop_front() {
            if let Some(node) = self.node_registry.borrow().get(&async_element.to) {
                if let Ok(poll) = node.async_poll_handle(
                    self.gen_ctx(async_element.to.clone()),
                    async_element.call_id,
                ) {
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
        if let Some((topic_name, payload)) = self.topic_queue.borrow_mut().pop_front() {
            if let Some(subscriber) = self.topic_subscriber_registry.borrow().get(&topic_name) {
                for node_id in subscriber {
                    if let Some(node) = self.node_registry.borrow().get(node_id) {
                        let _ = node.message_handle(
                            self.gen_ctx(node_id.clone()),
                            topic_name.clone(),
                            payload.clone(),
                        );
                    }
                }
            }
        }
    }
}

struct AsyncQueueElement {
    to: NodeID,
    call_id: AsyncCallID,
    callback_once: AsyncCallbackOnce,
}

#[derive(Clone)]
struct ContextImpl {
    node_id: NodeID,
    nodes: Rc<RefCell<HashMap<NodeID, Box<dyn Node>>>>,

    // 异步队列
    async_queue: Rc<RefCell<VecDeque<AsyncQueueElement>>>,

    // 话题订阅列表
    subscriber: Rc<RefCell<HashMap<TopicName, VecDeque<NodeID>>>>,
    topic_queue: Rc<RefCell<VecDeque<(TopicName, RawPayload)>>>,
}

impl Context for ContextImpl {
    fn broadcast_topic(&self, topic: TopicName, payload: RawPayload) -> Result<()> {
        self.topic_queue.borrow_mut().push_back((topic, payload));
        Ok(())
    }

    fn subscribe_topic(&self, topic: TopicName) -> Result<()> {
        let mut topic_subscriber_list = self.subscriber.borrow_mut();
        let subscribers = topic_subscriber_list.entry(topic).or_default();
        if !subscribers.contains(&self.node_id) {
            subscribers.push_back(self.node_id.clone());
        }
        Ok(())
    }

    fn unsubscribe_topic(&self, topic: TopicName) -> Result<()> {
        let node = &self.node_id;
        self.subscriber.borrow_mut().entry(topic).and_modify(|x| {
            x.retain(|x| x != node);
        });
        Ok(())
    }

    fn async_call(
        &self,
        node_id: NodeID,
        method_name: MethodName,
        payload: RawPayload,
        callback: AsyncCallbackOnce,
    ) -> Result<()> {
        if let Some(c) = self.nodes.borrow().get(&node_id) {
            let call_id = AsyncCallID(Uuid::new_v4());
            c.async_call_handle(Rc::new(self.clone()), call_id, method_name, payload)?;
            self.async_queue.borrow_mut().push_back(AsyncQueueElement {
                to: node_id,
                call_id,
                callback_once: callback,
            });
            Ok(())
        } else {
            log::error!("not found node {:?}", node_id);
            Err(Error::ComponentNotFound(node_id))
        }
    }
}
