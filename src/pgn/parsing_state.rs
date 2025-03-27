/// Represents the state of the PGN parser.
#[derive(Debug, PartialEq)]
pub enum PgnParsingState {
    Tags,
    Moves {
        move_number_just_seen: bool,
    },
    ResultFound
}