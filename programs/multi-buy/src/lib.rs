pub mod constants;
pub mod instructions;

use anchor_lang::prelude::*;

pub use constants::*;
use instructions::*;

declare_id!("FcftZkUR8VcDCNLsnwNorzqeG6bDphL3MCcTXFghqx28");

#[program]
pub mod multi_buy {
    use super::*;

    pub fn multi_buy<'info>(
        ctx: Context<'_, '_, '_, 'info, MultiBuy<'info>>,
        amount_in: u64,
        minimum_amount_out: u64,
        token_length: u8,
    ) -> Result<()> {
        instructions::multi_buy(ctx, amount_in, minimum_amount_out, token_length)
    }
}
