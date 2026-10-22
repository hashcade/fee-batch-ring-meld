use fee_batch_ring_meld::batch::*;

struct Game {
  stock: Stock,
  calls: u32,
  new: bool,
  refill: bool,
  broken: bool,
  close_after: Option<u32>,
}

impl Game {
  fn new(free: i32, fragments: i32) -> Self {
    Self {
      stock: Stock {
        source_free: free,
        source_equipped: 0,
        output_total: 0,
        output_equipped: 0,
        fragments,
        can_add: true,
      },
      calls: 0,
      new: false,
      refill: false,
      broken: false,
      close_after: None,
    }
  }

  fn run(&mut self, cost: Cost) -> Outcome {
    let plan = Plan::new(cost, self.stock).unwrap();
    execute(self, plan)
  }
}

impl Inventory for Game {
  fn stock(&mut self) -> Option<Stock> {
    if self.close_after.is_some_and(|count| self.calls >= count) {
      self.stock.can_add = false;
    }
    Some(self.stock)
  }

  fn meld_one(&mut self, cost: Cost) -> bool {
    self.calls += 1;
    self.stock.source_free -= cost.rings;
    if !self.broken {
      self.stock.output_total += 1;
    }
    self.stock.fragments -= cost.fragments;
    if self.refill {
      self.stock.fragments = 9_999_999;
    }
    self.new && self.calls == 1
  }
}

const C: Cost = Cost { rings: 2, fragments: 100 };
const B: Cost = Cost { rings: 3, fragments: 1_000 };
const A: Cost = Cost { rings: 4, fragments: 10_000 };

#[test]
fn all_three_ranks_charge_the_normal_cost_with_leftovers() {
  for cost in [C, B, A] {
    let mut game = Game::new(19, 999_999);
    let n = 19 / cost.rings;
    let result = game.run(cost);
    assert_eq!(result.completed, n as u32);
    assert_eq!(game.stock.source_free, 19 % cost.rings);
    assert_eq!(game.stock.output_total, n);
    assert_eq!(game.stock.fragments, 999_999 - cost.fragments * n);
    assert_eq!(result.stop, Stop::Completed);
  }
}

#[test]
fn only_one_group_is_supported() {
  assert_eq!(Game::new(4, 10_000).run(A).completed, 1);
}

#[test]
fn insufficient_rings_or_fragments_do_not_plan() {
  for (rings, fragments) in [(0, 100), (1, 100), (2, 0), (2, 99)] {
    assert!(Plan::new(C, Game::new(rings, fragments).stock).is_none());
  }
}

#[test]
fn fragments_can_limit_the_batch() {
  let mut game = Game::new(99, 2_999);
  assert_eq!(game.run(B).completed, 2);
  assert_eq!(game.stock.fragments, 999);
}

#[test]
fn equipped_source_and_result_copies_are_preserved_and_count_toward_capacity() {
  let mut game = Game::new(20, 999_999);
  game.stock.source_equipped = 7;
  game.stock.output_total = 98;
  game.stock.output_equipped = 8;
  assert_eq!(game.run(C).completed, 1);
  assert_eq!(game.stock.output_total, 99);
  assert_eq!(game.stock.source_equipped, 7);
  assert_eq!(game.stock.output_equipped, 8);
}

#[test]
fn full_or_native_blocked_inventory_does_not_plan() {
  let mut game = Game::new(20, 999_999);
  game.stock.output_total = 99;
  assert!(Plan::new(C, game.stock).is_none());
  game.stock.output_total = 0;
  game.stock.can_add = false;
  assert!(Plan::new(C, game.stock).is_none());
}

#[test]
fn native_false_return_is_not_a_failure() {
  let mut game = Game::new(12, 999_999);
  let result = game.run(C);
  assert_eq!(result.completed, 6);
  assert!(!result.any_new);
}

#[test]
fn first_acquisition_flag_is_accumulated() {
  let mut game = Game::new(12, 999_999);
  game.new = true;
  let result = game.run(C);
  assert_eq!(result.completed, 6);
  assert!(result.any_new);
}

#[test]
fn refill_cheat_cannot_extend_the_initial_plan() {
  let mut game = Game::new(99, 200);
  game.refill = true;
  assert_eq!(game.run(C).completed, 2);
  assert_eq!(game.calls, 2);
}

#[test]
fn capacity_is_checked_again_before_every_transaction() {
  let mut game = Game::new(20, 999_999);
  game.close_after = Some(2);
  let result = game.run(C);
  assert_eq!(result.completed, 2);
  assert_eq!(game.calls, 2);
  assert_eq!(result.stop, Stop::ResourcesChanged);
}

#[test]
fn mismatch_stops_without_retry_or_fallback() {
  let mut game = Game::new(20, 999_999);
  game.broken = true;
  let result = game.run(C);
  assert_eq!(result.stop, Stop::UnexpectedDelta);
  assert_eq!(game.calls, 1);
  assert_eq!(result.completed, 0);
}

#[test]
fn changes_between_confirmation_and_execution_are_rechecked() {
  let mut game = Game::new(20, 999_999);
  let plan = Plan::new(C, game.stock).unwrap();
  game.stock.fragments = 0;
  assert_eq!(execute(&mut game, plan).completed, 0);
  assert_eq!(game.calls, 0);
}

#[test]
fn invalid_costs_and_corrupt_stock_fail_closed() {
  for cost in [
    Cost { rings: 0, fragments: 0 },
    Cost { rings: 2, fragments: -1 },
    Cost { rings: 4, fragments: 1 },
  ] {
    assert!(Plan::new(cost, Game::new(99, 999_999).stock).is_none());
  }
  for free in [-1, 100, i32::MAX] {
    assert!(Plan::new(C, Game::new(free, 999_999).stock).is_none());
  }
}

#[test]
fn plan_totals_are_for_display_only() {
  let plan = Plan::new(A, Game::new(20, 999_999).stock).unwrap();
  assert_eq!(plan.total_cost(), Cost { rings: 20, fragments: 50_000 });
  assert_eq!(plan.cost, A);
}

#[test]
fn planning_never_exceeds_any_limit() {
  for cost in [C, B, A] {
    for free in 0..=99 {
      for output in 0..=99 {
        let mut stock = Game::new(free, 25_123).stock;
        stock.output_total = output;
        if let Some(plan) = Plan::new(cost, stock) {
          let totals = plan.total_cost();
          assert!(totals.rings <= free);
          assert!(totals.fragments <= stock.fragments);
          assert!(output + plan.count as i32 <= 99);
        }
      }
    }
  }
}
