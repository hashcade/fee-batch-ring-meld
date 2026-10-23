//! Compact confirmation text, following App.Language's text-language IDs.

use crate::batch::Plan;

fn bond_fragments(language: i32) -> &'static str {
  match language {
    0 => "絆のかけら",
    2 | 5 => "Fragments de lien",
    3 | 6 => "Fragmentos de vínculo",
    7 => "Bond-Fragmente",
    8 => "Frammenti di legame",
    9 => "羈絆碎片",
    10 => "羁绊碎片",
    11 => "인연의 조각",
    _ => "Bond Fragments",
  }
}

pub fn format(language: i32, ring_name: &str, source_rank: i32, plan: Plan) -> Option<String> {
  let (source, output) = match source_rank {
    0 => ('C', 'B'),
    1 => ('B', 'A'),
    2 => ('A', 'S'),
    _ => return None,
  };
  let total = plan.total_cost();
  Some(format!(
    "{ring_name} {source} ×{} → {output} ×{}\n{} ×{}",
    total.rings,
    plan.count,
    bond_fragments(language),
    total.fragments
  ))
}
