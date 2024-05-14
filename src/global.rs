use unicode_joining_type::{get_joining_type, JoiningType};

pub(crate) fn is_kashida(c: char) -> bool {
    matches!(get_joining_type(c), JoiningType::JoinCausing)
}
pub(crate) fn joins_following(c: char) -> bool {
    matches!(get_joining_type(c), JoiningType::DualJoining | JoiningType::JoinCausing)
}
pub(crate) fn joins_preceding(c: char) -> bool {
    matches!(
        get_joining_type(c),
        JoiningType::DualJoining | JoiningType::JoinCausing | JoiningType::RightJoining
    )
}
