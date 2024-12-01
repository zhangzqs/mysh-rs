use crate::payload::RawPayload;
use std::{rc::Rc, task::Poll};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct NodeID(String);

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct MethodName(String);

pub enum Error {
    ComponentNotFound(NodeID),
    Unimplemented,
}

pub type Result<T> = std::result::Result<T, Error>;

pub type RcContext = Rc<dyn Context>;

/// 非阻塞消息处理函数
pub trait Node {
    fn node_id(&self) -> NodeID;
    fn sync_call_handle(
        &self,
        ctx: RcContext,
        method_name: MethodName,
        payload: RawPayload,
    ) -> Result<RawPayload> {
        let _ = (ctx, payload, method_name);
        Err(Error::Unimplemented)
    }
    fn async_call_handle(
        &self,
        ctx: RcContext,
        seq: usize,
        method_name: MethodName,
        payload: RawPayload,
    ) -> Result<()> {
        let _ = (ctx, seq, payload, method_name);
        Err(Error::Unimplemented)
    }
    fn async_poll_handle(&self, ctx: RcContext, seq: usize) -> Result<Poll<RawPayload>> {
        let _ = (ctx, seq);
        Err(Error::Unimplemented)
    }
    fn message_handle(&self, ctx: RcContext, topic: TopicName, payload: RawPayload) -> Result<()> {
        let _ = (ctx, topic, payload);
        Err(Error::Unimplemented)
    }
    fn init(&self, ctx: RcContext) {
        let _ = ctx;
    }
}

pub type AsyncCallbackOnce = Box<dyn FnOnce(RawPayload)>;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct TopicName(String);

pub trait Context {
    /// 发送话题消息
    fn broadcast_topic(&self, topic: TopicName, payload: RawPayload);

    /// 订阅话题
    fn subscribe_topic(&self, topic: TopicName);

    /// 解除订阅话题
    fn unsubscribe_topic(&self, topic: TopicName);

    /// 同步RPC调用
    fn sync_call(
        &self,
        node_type: NodeID,
        method_name: MethodName,
        payload: RawPayload,
    ) -> Result<RawPayload>;

    /// 异步RPC调用
    fn async_call(
        &self,
        node_type: NodeID,
        method_name: MethodName,
        payload: RawPayload,
        callback: AsyncCallbackOnce,
    ) -> Result<()>;
}
