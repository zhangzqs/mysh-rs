use crate::{
    node::{AsyncCallbackOnce, Context, MethodName, NodeID, Result, TopicName},
    payload::RawPayload,
};

pub struct Scheduler {
    namespace: String,
}

impl Scheduler {
    pub fn init(&self) {}

    pub fn schedule_once(&self) {}
}

struct ContextImpl {}

impl Context for ContextImpl {
    fn broadcast_topic(&self, topic: TopicName, payload: RawPayload) -> Result<()> {
        // 发布话题 <namespace>/broadcast/<topic>
        todo!()
    }

    fn subscribe_topic(&self, topic: TopicName) -> Result<()> {
        // 订阅话题 <namespace>/broadcast/<topic>
        // mqtt broker会维护topic到mqtt client的订阅表A
        // 本地scheduler需要维护一个mqtt topic到node_id的一个订阅表B
        // 这样根据A和B的二级路由即可将mqtt消息广播路由到对应节点上

        // 接口订阅保证幂等
        // 订阅topic时，先判断B是否存在，
        //      如果B中存在则直接返回OK，
        //      如果B中不存在，则注册到订阅表B，并向broker注册A
        todo!()
    }

    fn unsubscribe_topic(&self, topic: TopicName) -> Result<()> {
        // 解除订阅话题 <namespace>/broadcast/<topic>

        // 接口保证幂等
        // 解除订阅先解除本地订阅表B
        // 当本地订阅表B中的某个topic的订阅数为0时，解除远程mqtt订阅消息
        todo!()
    }

    fn async_call(
        &self,
        node_id: NodeID,
        method_name: MethodName,
        payload: RawPayload,
        callback: AsyncCallbackOnce,
    ) -> Result<()> {
        // 发布话题 <namespace>/async_call/request/<node_id>
        // 消息内容：
        /*
        {
            "from_client_id": <client_id>,
            "from_node": <node_id>,
            "to_client_id": <client_id>,
            "to_node": <node_id>,
            "call_id": <call_id>, // 生成新的call_id
            "payload": <payload>,
        }
        */
        // 所有scheduler对每个注册节点都要订阅话题<namespace>/async_call/request/<node_id>
        // 当一个scheduler收到对应的订阅消息时，将payload解开，根据to_node路由到对应节点上

        // 当节点处理完请求，返回响应时，发布话题<namespace>/async_call/response/<node_id>
        // 消息内容
        /*
        {
            "from_client_id": <client_id>,
            "from_node": <node_id>,
            "to_client_id": <client_id>,
            "to_node": <node_id>,
            "call_id": <call_id>, // 使用请求对应的call_id
            "payload": <payload>,
        }
        */
        // 所有scheduler对每个注册节点都要订阅话题<namespace>/async_call/response/<node_id>
        // 当一个scheduler收到对应的订阅消息时，将payload解开，根据to_node路由到对应节点上
        todo!()
    }
}
