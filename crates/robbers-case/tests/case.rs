use robbers_case::*;
use robbers_types::{Direction, Hex, TileKind};

struct Map;
impl CityMap for Map {
    fn kind(&self, h: Hex) -> Option<TileKind> {
        match (h.q, h.r) {
            (0, 0) => Some(TileKind::Bank),
            (1, 0) => Some(TileKind::Police),
            (2, 0) => Some(TileKind::Park),
            (3, 0) => Some(TileKind::Airport),
            _ => None,
        }
    }
    fn tile_count(&self) -> usize {
        4
    }
}

#[test]
fn evidence_uses_explicit_order_and_one_based_steps() {
    let route: Vec<_> = (0..4).map(|q| Hex::new(q, 0)).collect();
    for evidence in [
        Evidence::Visits {
            kind: TileKind::Police,
        },
        Evidence::Avoids {
            kind: TileKind::Hospital,
        },
        Evidence::AtStep {
            step: 3,
            kind: TileKind::Park,
        },
        Evidence::Before {
            first: TileKind::Police,
            second: TileKind::Park,
        },
        Evidence::ImmediatelyAfter {
            first: TileKind::Police,
            second: TileKind::Park,
        },
        Evidence::Departs {
            kind: TileKind::Police,
            direction: Direction::E,
        },
        Evidence::FinalMove {
            direction: Direction::E,
        },
    ] {
        assert!(evidence.matches(&route, &Map), "{evidence:?}");
        assert!(!evidence.statement().is_empty());
    }
    for evidence in [
        Evidence::AtStep {
            step: 0,
            kind: TileKind::Bank,
        },
        Evidence::Before {
            first: TileKind::Park,
            second: TileKind::Police,
        },
        Evidence::Before {
            first: TileKind::School,
            second: TileKind::Park,
        },
        Evidence::ImmediatelyAfter {
            first: TileKind::Bank,
            second: TileKind::Airport,
        },
        Evidence::Departs {
            kind: TileKind::Airport,
            direction: Direction::E,
        },
    ] {
        assert!(!evidence.matches(&route, &Map));
    }
}

#[test]
fn briefing_serialization_does_not_publish_evidence_or_the_answer() {
    let case = bundled_case();
    let public = serde_json::to_value(case.public_briefing()).unwrap();
    assert!(public.get("solution").is_none());
    assert!(public.get("clues").is_none());
    assert_eq!(public["clue_sites"].as_array().unwrap().len(), 7);
}

#[test]
fn malformed_authoring_is_rejected() {
    let mut case = CaseDefinition {
        id: "test".into(),
        title: "Test".into(),
        briefing: "Test case".into(),
        bank: Hex::new(0, 0),
        airport: Hex::new(3, 0),
        route_length: 4,
        clues: vec![],
        solution: (0..4).map(|q| Hex::new(q, 0)).collect(),
    };
    assert!(case.validate(&Map).is_ok());
    case.solution.swap(1, 2);
    assert!(case.validate(&Map).is_err());
    case.solution.swap(1, 2);
    case.clues.push(Clue {
        id: "bad".into(),
        at: Hex::new(1, 0),
        witness: "Test".into(),
        evidence: Evidence::Visits {
            kind: TileKind::School,
        },
    });
    assert!(case.validate(&Map).unwrap_err().contains("contradicts"));
}
