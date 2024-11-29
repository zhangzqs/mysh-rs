use crate::payload::RawPayload;
use std::{rc::Rc, task::Poll};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ComponentType(String);

pub enum Error {
    Unimplemented,
}

pub type Result<T> = std::result::Result<T, Error>;

pub type RcContext = Rc<dyn Context>;

/// 非阻塞消息处理函数
pub trait Component {
    fn component_type(&self) -> ComponentType;
    fn sync_call_handle(&self, ctx: RcContext, payload: RawPayload) -> Result<RawPayload> {
        let _ = (ctx, payload);
        Err(Error::Unimplemented)
    }
    fn async_call_handle(&self, ctx: RcContext, seq: usize, payload: RawPayload) -> Result<()> {
        let _ = (ctx, seq, payload);
        Err(Error::Unimplemented)
    }
    fn async_poll_handle(&self, ctx: RcContext, seq: usize) -> Result<Poll<RawPayload>> {
        let _ = (ctx, seq);
        Err(Error::Unimplemented)
    }
    fn message_handle(&self, ctx: RcContext, topic: Topic, payload: RawPayload) -> Result<()> {
        let _ = (ctx, topic, payload);
        Err(Error::Unimplemented)
    }
}

pub type AsyncCallbackOnce = Box<dyn FnOnce(RawPayload)>;

pub struct Topic(String);

pub trait Context {
    /// 发送话题消息
    fn broadcast_topic(&self, topic: Topic, payload: RawPayload);

    /// 订阅话题
    fn subscribe_topic(&self, topic: Topic);

    /// 解除订阅话题
    fn unsubscribe_topic(&self, topic: Topic);

    /// 同步调用
    fn sync_call(&self, component_type: ComponentType, payload: RawPayload) -> Result<RawPayload>;

    /// 异步调用
    fn async_call(
        &self,
        component_type: ComponentType,
        payload: RawPayload,
        callback: AsyncCallbackOnce,
    ) -> Result<()>;
}
