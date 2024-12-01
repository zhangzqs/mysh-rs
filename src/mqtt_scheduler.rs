use crate::{
    node::{AsyncCallbackOnce, Context, MethodName, NodeID, Result, TopicName},
    payload::RawPayload,
};

pub struct Scheduler {}

impl Scheduler {
    pub fn init(&self) {}

    pub fn schedule_once(&self) {}
}

struct ContextImpl {}

impl Context for ContextImpl {
    fn broadcast_topic(&self, topic: TopicName, payload: RawPayload) {
        // 发布话题 <namespace>/broadcast/<topic>
        todo!()
    }

    fn subscribe_topic(&self, topic: TopicName) {
        // 订阅话题 <namespace>/broadcast/<topic>
        // 本地scheduler也需要维护一个mqtt topic到node_id的一个订阅表
        todo!()
    }

    fn unsubscribe_topic(&self, topic: TopicName) {
        // 解除订阅话题 <namespace>/broadcast/<topic>
        // 当本地订阅表中的某个topic的订阅数为0时，解除mqtt订阅消息
        todo!()
    }

    fn sync_call(
        &self,
        node_type: NodeID,
        method_name: MethodName,
        payload: RawPayload,
    ) -> Result<RawPayload> {
        // 暂时仅支持到本机的调用，可以接受阻塞式轮询mqtt消息？
        todo!()
    }

    fn async_call(
        &self,
        node_type: NodeID,
        method_name: MethodName,
        payload: RawPayload,
        callback: AsyncCallbackOnce,
    ) -> Result<()> {
        // 发布话题 <namespace>/async/<node_type>/<method_name>/<seq>
        // 各个scheduler对所有已注册的组件订阅话题 <namespace>/async_call/<node_type>/**
        todo!()
    }
}
