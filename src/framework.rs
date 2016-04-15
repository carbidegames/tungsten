use time::{PreciseTime, Duration};
use EventDispatcher;

pub trait Frontend<M> {
    fn process_events(&mut self, dispatcher: &mut EventDispatcher<M>, model: &mut M);
    fn render(&mut self, model: &M);
}

pub struct UpdateEvent {
    pub delta: f32
}

pub struct Framework<M, F> {
    model: M,
    frontend: F,
    dispatcher: EventDispatcher<M>,
}

impl<M: 'static, F: Frontend<M>> Framework<M, F> {
    pub fn new(model: M, frontend: F, dispatcher: EventDispatcher<M>) -> Self {
        Framework {
            model: model,
            frontend: frontend,
            dispatcher: dispatcher,
        }
    }

    pub fn run<RC: Fn(&M) -> bool>(mut self, run_condition: RC) {
        let mut last_update = PreciseTime::now();

        // Loop as long as the run condition of our model is still true
        while run_condition(&self.model) {
            // Check time elapsed since the last update tick
            let now = PreciseTime::now();
            let elapsed = last_update.to(now);

            // If we're over a very small minimum, run the update tick
            // This is to avoid floating point errors with really small deltas
            if elapsed > Duration::milliseconds(1) {
                // Turn the delta into a multiplier and keep track of when this update tick happened
                let delta = elapsed.num_nanoseconds().unwrap() as f32 / 1_000_000_000.0;
                assert!(delta > 0.0);
                last_update = now;

                // Perform the actual update tick
                self.frontend.process_events(&mut self.dispatcher, &mut self.model);
                self.dispatcher.dispatch(&mut self.model, UpdateEvent { delta: delta });
                self.frontend.render(&self.model);
            } else {
                // Yield some CPU time
                // TODO: Yields too much on i7s and is overall unpredictable
                ::std::thread::sleep(::std::time::Duration::from_millis(1));
            }
        }
    }
}
