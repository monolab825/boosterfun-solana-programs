use anchor_lang::prelude::*;

#[error_code]
pub enum TutorialError {
    #[msg("Invalid fee value")]
    InvalidFee,

    #[msg("Invalid buy too many tokens")]
    InvalidTooMany,

    #[msg("Output is below the minimum expected")]
    OutputTooSmall,
}
