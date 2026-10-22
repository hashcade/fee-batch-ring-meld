use crate::batch::Plan;

#[derive(Clone, Debug)]
pub struct Selection {
  pub menu: usize,
  pub rnid: String,
  pub prompt: String,
  pub plan: Plan,
}

#[derive(Default)]
pub struct Session {
  selection: Option<Selection>,
  displayed: bool,
  confirmed: bool,
}

impl Session {
  pub fn prepare(&mut self, selection: Option<Selection>) {
    self.clear();
    self.selection = selection;
  }

  pub fn prompt(&self) -> Option<&str> {
    self.selection.as_ref().map(|s| s.prompt.as_str())
  }

  pub fn mark_displayed(&mut self) {
    self.displayed = self.selection.is_some();
  }

  pub fn confirm(&mut self, menu: usize) {
    self.confirmed = self.displayed && self.selection.as_ref().is_some_and(|s| s.menu == menu);
  }

  pub fn destroy_menu(&mut self, menu: usize) {
    // The original sequence destroys the menu before executing its result.
    // Only an explicitly confirmed plan survives this normal hand-off.
    if !self.confirmed && self.selection.as_ref().is_some_and(|s| s.menu == menu) {
      self.clear();
    }
  }

  pub fn take_confirmed(&mut self) -> Option<Selection> {
    let selected = if self.confirmed { self.selection.take() } else { None };
    self.clear();
    selected
  }

  pub fn clear(&mut self) {
    self.selection = None;
    self.displayed = false;
    self.confirmed = false;
  }
}

#[derive(Default)]
pub struct Execution {
  running: bool,
  batch: bool,
  selection: Option<Selection>,
}

pub enum Dispatch {
  Vanilla,
  Batch(Selection),
  Suppress,
}

impl Execution {
  pub fn begin(&mut self, selection: Option<Selection>) -> bool {
    if self.running {
      return false;
    }
    self.running = true;
    self.batch = selection.is_some();
    self.selection = selection;
    true
  }

  pub fn dispatch(&mut self) -> Dispatch {
    if !self.batch {
      return Dispatch::Vanilla;
    }
    match self.selection.take() {
      Some(selection) => Dispatch::Batch(selection),
      None => Dispatch::Suppress,
    }
  }

  pub fn end(&mut self) {
    *self = Self::default();
  }
}
