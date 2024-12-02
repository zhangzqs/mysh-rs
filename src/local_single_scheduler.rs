use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use futures::{future::LocalBoxFuture, task::LocalSpawnExt};
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
    node_registry: Rc<RefCell<HashMap<NodeID, Rc<dyn Node>>>>,
    async_queue: Rc<RefCell<VecDeque<AsyncQueueElement>>>,
    future_pool: Rc<RefCell<futures::executor::LocalPool>>,
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
            .insert(node.node_id(), Rc::new(node));
    }

    fn gen_ctx(&self, node_id: NodeID) -> RcContext {
        Rc::new(ContextImpl {
            node_id,
            nodes: self.node_registry.clone(),
            async_queue: self.async_queue.clone(),
            future_pool: self.future_pool.clone(),
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
        // 调度基于回调的异步
        if let Some(async_element) = self.async_queue.borrow_mut().pop_front() {
            if let Some(node) = self.node_registry.borrow().get(&async_element.to) {
                let poll = node.async_poll_handle(
                    self.gen_ctx(async_element.to.clone()),
                    async_element.call_id,
                    async_element.method_name.clone(),
                    async_element.payload.clone(),
                );
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
        // 调度话题广播
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
        let _ = self.future_pool.borrow_mut().try_run_one();
    }
}

struct AsyncQueueElement {
    to: NodeID,
    call_id: AsyncCallID,
    method_name: MethodName,
    payload: RawPayload,
    callback_once: AsyncCallbackOnce,
}

#[derive(Clone)]
struct ContextImpl {
    node_id: NodeID,
    nodes: Rc<RefCell<HashMap<NodeID, Rc<dyn Node>>>>,

    // 异步队列
    async_queue: Rc<RefCell<VecDeque<AsyncQueueElement>>>,
    future_pool: Rc<RefCell<futures::executor::LocalPool>>,

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
    ) -> Result<LocalBoxFuture<'static, Result<RawPayload>>> {
        let (sender, receiver) = futures::channel::oneshot::channel();
        self.async_call_with_callback(
            node_id,
            method_name,
            payload,
            Box::new(move |payload| {
                let _ = sender.send(payload);
            }),
        )?;
        Ok(Box::pin(async move {
            receiver.await.expect("The sender was dropped")
        }))
    }

    fn spawn_future(&self, future: LocalBoxFuture<'static, ()>) -> Result<()> {
        if let Err(_) = self.future_pool.borrow().spawner().spawn_local(future) {
            Err(Error::SpawnError)
        } else {
            Ok(())
        }
    }

    fn async_call_with_callback(
        &self,
        node_id: NodeID,
        method_name: MethodName,
        payload: RawPayload,
        callback: AsyncCallbackOnce,
    ) -> Result<()> {
        if let Some(c) = self.nodes.borrow().get(&node_id) {
            let call_id = AsyncCallID(Uuid::new_v4());
            match c.async_poll_handle(
                Rc::new(self.clone()),
                call_id,
                method_name.clone(),
                payload.clone(),
            )? {
                std::task::Poll::Ready(v) => {
                    callback(Ok(v));
                    Ok(())
                }
                std::task::Poll::Pending => {
                    self.async_queue.borrow_mut().push_back(AsyncQueueElement {
                        to: node_id,
                        call_id,
                        method_name,
                        payload,
                        callback_once: callback,
                    });
                    Ok(())
                }
            }
        } else {
            log::error!("not found node {:?}", node_id);
            Err(Error::NodeNotFound(node_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        node::{MethodName, Node, NodeID, RcContext, TopicName},
        payload::{EncodeType, Payload, PayloadType},
    };

    use super::Scheduler;

    struct NodeA {}

    impl Node for NodeA {
        fn node_id(&self) -> NodeID {
            NodeID::try_from("node_a".to_string()).unwrap()
        }

        fn init(&self, ctx: RcContext) {
            println!("node a init");
            ctx.subscribe_topic(TopicName::try_from("hello".to_string()).unwrap())
                .unwrap();
        }

        fn message_handle(
            &self,
            ctx: RcContext,
            topic: TopicName,
            payload: crate::payload::RawPayload,
        ) -> crate::node::Result<()> {
            println!("node a recv message: {payload:?}");
            Ok(())
        }

        fn async_poll_handle(
            &self,
            ctx: RcContext,
            call_id: crate::node::AsyncCallID,
            method_name: MethodName,
            payload: crate::payload::RawPayload,
        ) -> std::task::Poll<crate::node::Result<crate::payload::RawPayload>> {
            let ping_text = PingPayload::from_raw_payload(&payload).unwrap().ping_text;
            std::task::Poll::Ready(Ok(PongPayload {
                pong_text: format!("pong text(ping is {}, call_id is {})", ping_text, call_id.0),
            }
            .to_raw_payload(EncodeType::Json)
            .unwrap()))
        }
    }

    struct NodeB {}

    impl Node for NodeB {
        fn node_id(&self) -> NodeID {
            NodeID::try_from("node_b".to_string()).unwrap()
        }

        fn init(&self, ctx: RcContext) {
            println!("node b init");
            ctx.broadcast_topic(
                TopicName::try_from("hello".to_string()).unwrap(),
                PingPayload {
                    ping_text: "hello".into(),
                }
                .to_raw_payload(EncodeType::Bincode)
                .unwrap(),
            )
            .unwrap();
            ctx.spawn_future({
                let ctx = ctx.clone();
                Box::pin(async move {
                    let resp = ctx
                        .async_call(
                            NodeID::try_from("node_a".to_string()).unwrap(),
                            MethodName::try_from("test_method".to_string()).unwrap(),
                            PingPayload {
                                ping_text: "hello1".into(),
                            }
                            .to_raw_payload(EncodeType::Bincode)
                            .unwrap(),
                        )
                        .unwrap()
                        .await
                        .unwrap();
                    let pong_text = String::from_utf8(
                        PongPayload::from_raw_payload(&resp)
                            .unwrap()
                            .try_encode(EncodeType::Json)
                            .unwrap(),
                    )
                    .unwrap();
                    println!("resp1: {pong_text}");
                })
            })
            .unwrap();
            ctx.async_call_with_callback(
                NodeID::try_from("node_a".to_string()).unwrap(),
                MethodName::try_from("test_method".to_string()).unwrap(),
                PingPayload {
                    ping_text: "hello2".into(),
                }
                .to_raw_payload(EncodeType::Bincode)
                .unwrap(),
                Box::new(|x| {
                    let resp = x.unwrap();
                    let pong_text = String::from_utf8(
                        PongPayload::from_raw_payload(&resp)
                            .unwrap()
                            .try_encode(EncodeType::Json)
                            .unwrap(),
                    )
                    .unwrap();
                    println!("resp2: {}", pong_text);
                }),
            )
            .unwrap();
        }
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct PingPayload {
        ping_text: String,
    }

    impl Payload for PingPayload {
        fn payload_type() -> PayloadType {
            PayloadType::try_from("ping".to_string()).unwrap()
        }
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct PongPayload {
        pong_text: String,
    }

    impl Payload for PongPayload {
        fn payload_type() -> PayloadType {
            PayloadType::try_from("pong".to_string()).unwrap()
        }
    }

    #[test]
    fn test_ping() {
        let sche = Scheduler::new();
        sche.register_node(NodeA {});
        sche.register_node(NodeB {});
        sche.init();
        sche.schedule_once();
        sche.schedule_once();
    }
}
