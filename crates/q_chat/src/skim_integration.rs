use eyre::{
    Result,
    eyre,
};
use rustyline::{
    Cmd,
    ConditionalEventHandler,
    EventContext,
    RepeatCount,
};
use tempfile::NamedTempFile;
use std::sync::Arc;
use fig_terminal::Selector;

pub fn select_item(items: Vec<String>, prompt: &str) -> Result<Option<String>> {
    let mut selector = Selector::new(items, prompt)?;
    selector.run().map_err(|e| eyre!("Selector error: {}", e))
}

pub struct SkimHandler;

impl ConditionalEventHandler for SkimHandler {
    fn handle(&self, _evt: &rustyline::Event, _n: RepeatCount, _ctx: &EventContext) -> Option<Cmd> {
        None
    }
}
