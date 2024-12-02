use futures::future::LocalBoxFuture;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::payload::{self, RawPayload};
use std::{fmt::Display, rc::Rc, task::Poll};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct NodeID(String);

impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<NodeID> for String {
    fn from(val: NodeID) -> Self {
        val.0
    }
}

impl TryFrom<String> for NodeID {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]{0,15}$").unwrap());
        if RE.is_match(&value) {
            Ok(Self(value))
        } else {
            Err(Error::InvalidNodeID(value))
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct MethodName(String);

impl From<MethodName> for String {
    fn from(val: MethodName) -> Self {
        val.0
    }
}

impl TryFrom<String> for MethodName {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]{0,15}$").unwrap());
        if RE.is_match(&value) {
            Ok(Self(value))
        } else {
            Err(Error::InvalidMethodName(value))
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct TopicName(String);

impl From<TopicName> for String {
    fn from(val: TopicName) -> Self {
        val.0
    }
}

impl TryFrom<String> for TopicName {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]{0,15}$").unwrap());
        if RE.is_match(&value) {
            Ok(Self(value))
        } else {
            Err(Error::InvalidTopicName(value))
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("the node id `{0}` is invalid")]
    InvalidNodeID(String),
    #[error("the method name `{0}` is invalid")]
    InvalidMethodName(String),
    #[error("the topic name `{0}` is invalid")]
    InvalidTopicName(String),
    #[error("the node id `{0}` is invalid")]
    NodeNotFound(NodeID),
    #[error("unimemented operation")]
    UnimplementedOperation,
    #[error("payload error: {0}")]
    PayloadError(#[from] payload::Error),
    #[error("spawn future error")]
    SpawnError,
}

pub type Result<T> = std::result::Result<T, Error>;

pub type RcContext = Rc<dyn Context>;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct AsyncCallID(pub uuid::Uuid);

/// 非阻塞消息处理函数
pub trait Node {
    fn node_id(&self) -> NodeID;
    fn async_poll_handle(
        &self,
        ctx: RcContext,
        call_id: AsyncCallID,
        method_name: MethodName,
        payload: RawPayload,
    ) -> Poll<Result<RawPayload>> {
        let _ = (ctx, call_id, method_name, payload);
        Poll::Ready(Err(Error::UnimplementedOperation))
    }
    fn message_handle(&self, ctx: RcContext, topic: TopicName, payload: RawPayload) -> Result<()> {
        let _ = (ctx, topic, payload);
        Err(Error::UnimplementedOperation)
    }
    fn init(&self, ctx: RcContext) {
        let _ = ctx;
    }
}

pub type AsyncCallbackOnce = Box<dyn FnOnce(Result<RawPayload>)>;

pub trait Context {
    /// 发送话题消息
    fn broadcast_topic(&self, topic: TopicName, payload: RawPayload) -> Result<()>;

    /// 订阅话题
    fn subscribe_topic(&self, topic: TopicName) -> Result<()>;

    /// 解除订阅话题
    fn unsubscribe_topic(&self, topic: TopicName) -> Result<()>;

    fn async_call_with_callback(
        &self,
        node_id: NodeID,
        method_name: MethodName,
        payload: RawPayload,
        callback: AsyncCallbackOnce,
    ) -> Result<()>;

    /// 异步RPC调用
    fn async_call(
        &self,
        node_id: NodeID,
        method_name: MethodName,
        payload: RawPayload,
    ) -> Result<LocalBoxFuture<'static, Result<RawPayload>>>;

    fn spawn_future(&self, future: LocalBoxFuture<'static, ()>) -> Result<()>;
}
