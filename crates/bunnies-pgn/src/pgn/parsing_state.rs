/// Represents the state of the PGN parser.
#[derive(Debug)]
#[derive_const(PartialEq)]
pub enum PgnParsingState {
    Tags,
    Moves { move_number_just_seen: bool },
    ResultFound,
}
