//! Pure transaction planning, independent of the game runtime.

pub const MAX_RING_COUNT: i32 = 99;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cost {
  pub rings: i32,
  pub fragments: i32,
}

impl Cost {
  pub fn is_valid(self) -> bool {
    matches!((self.rings, self.fragments), (2, 100) | (3, 1_000) | (4, 10_000))
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stock {
  pub source_free: i32,
  pub source_equipped: i32,
  pub output_total: i32,
  pub output_equipped: i32,
  pub fragments: i32,
  pub can_add: bool,
}

impl Stock {
  fn is_valid(self) -> bool {
    self.source_free >= 0
      && self.source_equipped >= 0
      && self.source_free.checked_add(self.source_equipped).is_some_and(|v| v <= MAX_RING_COUNT)
      && (0..=MAX_RING_COUNT).contains(&self.output_total)
      && (0..=self.output_total).contains(&self.output_equipped)
      && self.fragments >= 0
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
  pub cost: Cost,
  pub count: u32,
}

impl Plan {
  pub fn new(cost: Cost, stock: Stock) -> Option<Self> {
    if !cost.is_valid() || !stock.is_valid() || !stock.can_add {
      return None;
    }
    let count = (stock.source_free / cost.rings)
      .min(stock.fragments / cost.fragments)
      .min(MAX_RING_COUNT - stock.output_total);
    (count > 0).then_some(Self { cost, count: count as u32 })
  }

  pub fn total_cost(self) -> Cost {
    Cost {
      rings: self.cost.rings * self.count as i32,
      fragments: self.cost.fragments * self.count as i32,
    }
  }
}

pub trait Inventory {
  fn stock(&mut self) -> Option<Stock>;
  /// Returns the game's new-acquisition flag, NOT transaction success.
  fn meld_one(&mut self, cost: Cost) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
  Completed,
  ResourcesChanged,
  UnexpectedDelta,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
  pub completed: u32,
  pub any_new: bool,
  pub stop: Stop,
}

pub fn execute(inventory: &mut impl Inventory, plan: Plan) -> Outcome {
  let mut outcome = Outcome { completed: 0, any_new: false, stop: Stop::ResourcesChanged };
  if !plan.cost.is_valid() || plan.count == 0 || plan.count > MAX_RING_COUNT as u32 {
    return outcome;
  }
  for _ in 0..plan.count {
    let Some(before) = inventory.stock() else { return outcome };
    if Plan::new(plan.cost, before).is_none() {
      return outcome;
    }
    outcome.any_new |= inventory.meld_one(plan.cost);
    let Some(after) = inventory.stock() else {
      outcome.stop = Stop::UnexpectedDelta;
      return outcome;
    };
    // Fragment refill cheats may restore the balance immediately. Inventory
    // deltas remain authoritative; the initial plan still bounds the loop.
    let expected_fragments = before.fragments - plan.cost.fragments;
    if !after.is_valid()
      || after.source_free != before.source_free - plan.cost.rings
      || after.output_total != before.output_total + 1
      || after.source_equipped != before.source_equipped
      || after.output_equipped != before.output_equipped
      || after.fragments < expected_fragments
    {
      outcome.stop = Stop::UnexpectedDelta;
      return outcome;
    }
    outcome.completed += 1;
  }
  outcome.stop = Stop::Completed;
  outcome
}
