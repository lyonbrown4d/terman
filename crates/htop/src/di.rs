use std::error::Error;

use fluxdi::{Application, Injector, Module, Provider, Shared};

use crate::cli::HtopArgs;

struct HtopModule {
    args: HtopArgs,
}

impl HtopModule {
    fn new(args: HtopArgs) -> Self {
        Self { args }
    }
}

impl Module for HtopModule {
    fn providers(&self, injector: &Injector) {
        let args = self.args.clone();
        injector.provide::<HtopArgs>(Provider::root(move |_| Shared::new(args.clone())));
        injector.provide::<HtopRunner>(Provider::root(|injector| {
            let args = injector.resolve::<HtopArgs>();
            Shared::new(HtopRunner::new(args.as_ref().clone()))
        }));
    }
}

struct HtopRunner {
    args: HtopArgs,
}

impl HtopRunner {
    fn new(args: HtopArgs) -> Self {
        Self { args }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        crate::app::run(self.args.clone()).await
    }
}

pub(crate) async fn run(args: HtopArgs) -> Result<(), Box<dyn Error>> {
    let mut app = Application::new(HtopModule::new(args));
    app.bootstrap_sync()?;
    let runner = app.injector().resolve::<HtopRunner>();
    runner.run().await
}
