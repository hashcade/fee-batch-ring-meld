use fee_batch_ring_meld::{
  batch::{Cost, Plan},
  confirmation,
};

fn plan() -> Plan {
  Plan { cost: Cost { rings: 4, fragments: 10_000 }, count: 5 }
}

#[test]
fn chinese_prompt_shows_each_name_once_and_both_total_costs() {
  assert_eq!(
    confirmation::format(10, "米娜娃", 2, plan()).unwrap(),
    "米娜娃 A ×20 → S ×5\n羁绊碎片 ×50000"
  );
}

#[test]
fn all_game_text_languages_have_localized_two_line_prompts() {
  let expected = [
    "絆のかけら",
    "Bond Fragments",
    "Fragments de lien",
    "Fragmentos de vínculo",
    "Bond Fragments",
    "Fragments de lien",
    "Fragmentos de vínculo",
    "Bond-Fragmente",
    "Frammenti di legame",
    "羈絆碎片",
    "羁绊碎片",
    "인연의 조각",
  ];
  for (language, label) in expected.iter().enumerate() {
    let prompt = confirmation::format(language as i32, "Minerva", 2, plan()).unwrap();
    assert_eq!(prompt, format!("Minerva A ×20 → S ×5\n{label} ×50000"));
    assert_eq!(prompt.lines().count(), 2);
    assert_eq!(prompt.matches("Minerva").count(), 1);
    assert!(!prompt.contains("Meld this batch"));
    assert!(!prompt.contains("Cost:"));
  }
}

#[test]
fn language_is_selected_for_each_prompt_and_unknown_ids_fall_back_to_english() {
  for language in [10, 0, 11, 9, 1, -1, 12, i32::MAX] {
    let prompt = confirmation::format(language, "Ring", 2, plan()).unwrap();
    let label = match language {
      10 => "羁绊碎片",
      0 => "絆のかけら",
      11 => "인연의 조각",
      9 => "羈絆碎片",
      _ => "Bond Fragments",
    };
    assert!(prompt.ends_with(&format!("{label} ×50000")));
  }
}

#[test]
fn every_source_rank_uses_its_actual_batch_totals() {
  for (rank, cost, expected) in [
    (0, Cost { rings: 2, fragments: 100 }, "Ring C ×6 → B ×3\nBond Fragments ×300"),
    (1, Cost { rings: 3, fragments: 1_000 }, "Ring B ×9 → A ×3\nBond Fragments ×3000"),
    (2, Cost { rings: 4, fragments: 10_000 }, "Ring A ×12 → S ×3\nBond Fragments ×30000"),
  ] {
    assert_eq!(confirmation::format(1, "Ring", rank, Plan { cost, count: 3 }).unwrap(), expected);
  }
}

#[test]
fn invalid_source_ranks_cannot_produce_a_confirmation() {
  for rank in [-1, 3, i32::MAX] {
    assert!(confirmation::format(1, "Ring", rank, plan()).is_none());
  }
}
