//! Exhaustive authoring tool. Search uses only public rules and typed evidence.
use robbers_case::{CaseDefinition, CityMap};
use robbers_types::{Direction, Hex};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ValidationReport {
    pub case_id: String,
    pub candidate_routes: u64,
    pub matching_routes: u64,
    pub nodes_visited: u64,
    pub complete: bool,
    pub unique: bool,
}

/// The budget bounds custom-case work. Incomplete searches can never certify uniqueness.
pub fn validate_case(
    case: &CaseDefinition,
    map: &impl CityMap,
    node_budget: u64,
) -> Result<ValidationReport, String> {
    case.validate(map)?;
    let mut report = ValidationReport {
        case_id: case.id.clone(),
        candidate_routes: 0,
        matching_routes: 0,
        nodes_visited: 0,
        complete: true,
        unique: false,
    };
    let mut route = vec![case.bank];
    search(case, map, &mut route, node_budget, &mut report);
    report.unique = report.complete && report.matching_routes == 1;
    Ok(report)
}

fn search(
    case: &CaseDefinition,
    map: &impl CityMap,
    route: &mut Vec<Hex>,
    budget: u64,
    report: &mut ValidationReport,
) {
    if report.nodes_visited >= budget {
        report.complete = false;
        return;
    }
    report.nodes_visited += 1;
    let current = *route.last().unwrap();
    let remaining = case.route_length - route.len();
    if current.distance(case.airport) > remaining as u64 {
        return;
    }
    if remaining == 0 {
        if current == case.airport {
            report.candidate_routes += 1;
            if case
                .clues
                .iter()
                .all(|clue| clue.evidence.matches(route, map))
            {
                report.matching_routes += 1;
            }
        }
        return;
    }
    if current == case.airport {
        return;
    }
    for direction in Direction::ALL {
        let next = current.neighbor(direction);
        if map.kind(next).is_some() && !route.contains(&next) {
            route.push(next);
            search(case, map, route, budget, report);
            route.pop();
            if !report.complete {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use robbers_case::bundled_case;
    use robbers_game::Board;

    #[test]
    fn bundled_case_has_exactly_one_solution_among_many_legal_routes() {
        let result = validate_case(&bundled_case(), &Board::default(), 1_000_000).unwrap();
        assert!(result.unique && result.complete, "{result:?}");
        assert_eq!(result.matching_routes, 1);
        assert!(result.candidate_routes > 100);
    }

    #[test]
    fn missing_clues_and_exhausted_budgets_cannot_certify_uniqueness() {
        let mut case = bundled_case();
        case.clues.clear();
        let result = validate_case(&case, &Board::default(), 1_000_000).unwrap();
        assert!(!result.unique && result.complete);
        assert_eq!(result.matching_routes, result.candidate_routes);
        assert!(result.matching_routes > 1);
        let incomplete = validate_case(&case, &Board::default(), 1).unwrap();
        assert!(!incomplete.unique && !incomplete.complete);
    }

    #[test]
    fn contradictory_authored_solution_is_rejected_before_search() {
        let mut case = bundled_case();
        case.solution.swap(1, 2);
        assert!(validate_case(&case, &Board::default(), 1_000_000).is_err());
    }
}
