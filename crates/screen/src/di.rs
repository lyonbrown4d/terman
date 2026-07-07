use std::error::Error;

use fluxdi::{Application, Injector, Module, Provider, Shared};

use crate::cli::ScreenArgs;

struct ScreenModule {
    args: ScreenArgs,
}

impl ScreenModule {
    fn new(args: ScreenArgs) -> Self {
        Self { args }
    }
}

impl Module for ScreenModule {
    fn providers(&self, injector: &Injector) {
        let args = self.args.clone();
        injector.provide::<ScreenArgs>(Provider::root(move |_| Shared::new(args.clone())));
        injector.provide::<ScreenRunner>(Provider::root(|injector| {
            let args = injector.resolve::<ScreenArgs>();
            Shared::new(ScreenRunner::new(args.as_ref().clone()))
        }));
    }
}

struct ScreenRunner {
    args: ScreenArgs,
}

impl ScreenRunner {
    fn new(args: ScreenArgs) -> Self {
        Self { args }
    }

    fn run(&self) -> Result<(), Box<dyn Error>> {
        crate::run_command(self.args.clone())
    }
}

pub(crate) fn run(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let mut app = Application::new(ScreenModule::new(args));
    app.bootstrap_sync()?;
    let runner = app.injector().resolve::<ScreenRunner>();
    runner.run()
}
