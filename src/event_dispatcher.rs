use std::any::Any;
use std::marker::PhantomData;
use dynamic::Dynamic;

pub trait EventHandler<M, E> {
    fn handle(&mut self, model: &mut M, event: &E);
}

impl<M, E, F: Fn(&mut M, &E)> EventHandler<M, E> for F {
    fn handle(&mut self, model: &mut M, event: &E) {
        self(model, event);
    }
}

trait HandlerWrapper<M> {
    fn attempt_dispatch(&mut self, model: &mut M, event: &Dynamic);
}

struct ConcreteHandlerWrapper<M, E, H> {
    handler: H,
    _m: PhantomData<M>,
    _e: PhantomData<E>,
}

impl<M, E, H: EventHandler<M, E>> ConcreteHandlerWrapper<M, E, H> {
    fn new(handler: H) -> Self {
        ConcreteHandlerWrapper {
            handler: handler,
            _m: PhantomData::<M>,
            _e: PhantomData::<E>,
        }
    }
}

impl<M, E: Any, H: EventHandler<M, E>> HandlerWrapper<M> for ConcreteHandlerWrapper<M, E, H> {
    fn attempt_dispatch(&mut self, model: &mut M, event: &Dynamic) {
        if let Some(ref evt) = event.downcast_ref::<E>() {
            self.handler.handle(model, evt);
        }
    }
}

pub struct EventDispatcher<M> {
    handlers: Vec<Box<HandlerWrapper<M>>>
}

impl<M: 'static> EventDispatcher<M> {
    pub fn new() -> Self {
        EventDispatcher {
            handlers: Vec::new()
        }
    }

    pub fn add_handler<E: Any, H: EventHandler<M, E> + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(ConcreteHandlerWrapper::new(handler)));
    }

    pub fn dispatch<E: Any>(&mut self, model: &mut M, event: E) {
        let dyn_event = Dynamic::new(event);
        // TODO: Use a table lookup with the type rather than relying on every handler to check
        for handler in &mut self.handlers {
            handler.attempt_dispatch(model, &dyn_event);
        }
    }

    pub fn dispatch_dynamic(&mut self, model: &mut M, event: Box<Dynamic>) {
        for handler in &mut self.handlers {
            handler.attempt_dispatch(model, &event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EventDispatcher;

    #[test]
    fn dispatches_event_correctly() {
        struct Model { raised_a: bool, raised_b: bool };
        struct CorrectEvt;
        struct IgnoreEvt;

        let mut model = Model { raised_a: false, raised_b: false };
        let mut dispatcher = EventDispatcher::new();
        dispatcher.add_handler(|m: &mut Model, _e: &CorrectEvt| m.raised_a = true);
        dispatcher.add_handler(|m: &mut Model, _e: &IgnoreEvt| m.raised_b = true);

        dispatcher.dispatch(&mut model, CorrectEvt);

        assert!(model.raised_a);
        assert!(!model.raised_b);
    }
}
