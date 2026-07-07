use std::error::Error;

use fluxdi::{Application, Injector, Module, Provider, Shared};

use crate::cli::TmuxArgs;

struct TmuxModule {
    args: TmuxArgs,
}

impl TmuxModule {
    fn new(args: TmuxArgs) -> Self {
        Self { args }
    }
}

impl Module for TmuxModule {
    fn providers(&self, injector: &Injector) {
        let args = self.args.clone();
        injector.provide::<TmuxArgs>(Provider::root(move |_| Shared::new(args.clone())));
        injector.provide::<TmuxRunner>(Provider::root(|injector| {
            let args = injector.resolve::<TmuxArgs>();
            Shared::new(TmuxRunner::new(args.as_ref().clone()))
        }));
    }
}

struct TmuxRunner {
    args: TmuxArgs,
}

impl TmuxRunner {
    fn new(args: TmuxArgs) -> Self {
        Self { args }
    }

    fn run(&self) -> Result<(), Box<dyn Error>> {
        crate::run_command(self.args.clone())
    }
}

pub(crate) fn run(args: TmuxArgs) -> Result<(), Box<dyn Error>> {
    let mut app = Application::new(TmuxModule::new(args));
    app.bootstrap_sync()?;
    let runner = app.injector().resolve::<TmuxRunner>();
    runner.run()
}
