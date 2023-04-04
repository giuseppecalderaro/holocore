use crate::components::processing_units::strategy::Algorithms;
use crate::components::processing_units::strategy::trend_following;

pub fn make(strategy: &str) -> Algorithms {
    match strategy {
        "TrendFollowing" => Algorithms::TrendFollowing(trend_following::TrendFollowing::new()),
        _ => panic!("Cannot build algorithm {}", strategy)
    }
}
