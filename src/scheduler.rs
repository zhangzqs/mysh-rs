use crate::{
    component::{Component, ComponentType, Context, Result},
    payload::RawPayload,
};

struct Scheduler {}

impl Scheduler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn register<C: Component>(&self, component: C) {}

    pub fn schedule_once(&self) {}
}

struct ContextImpl {}

impl Context for ContextImpl {
    fn broadcast_topic(&self, topic: crate::component::Topic, payload: RawPayload) {
        todo!()
    }

    fn subscribe_topic(&self, topic: crate::component::Topic) {
        todo!()
    }

    fn unsubscribe_topic(&self, topic: crate::component::Topic) {
        todo!()
    }

    fn sync_call(&self, component_type: ComponentType, payload: RawPayload) -> Result<RawPayload> {
        todo!()
    }

    fn async_call(
        &self,
        component_type: ComponentType,
        payload: RawPayload,
        callback: crate::component::AsyncCallbackOnce,
    ) -> Result<()> {
        todo!()
    }
}
