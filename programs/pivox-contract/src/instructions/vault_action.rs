use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::state::VaultAccount;

#[derive(Accounts)]
pub struct VaultConfig<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(mut)]
    pub freelancer: Signer<'info>,

    pub usdc_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault_account", client.key().as_ref(), freelancer.key().as_ref()],
        bump,
    )]
    pub vault_account: Account<'info, VaultAccount>,

    pub system_program: Program<'info, System>,
}

impl<'info> VaultConfig<'info> {
    pub fn pause_vault(&mut self) -> Result<()> {
        if !self.client.is_signer || !self.freelancer.is_signer {
            return Ok(());
        }

        self.vault_account.vault_status = "Paused".to_string();
        Ok(())
    }

    pub fn restart_vault(&mut self) -> Result<()> {
        if !self.client.is_signer || !self.freelancer.is_signer {
            return Ok(());
        }

        self.vault_account.vault_status = "Active".to_string();
        Ok(())
    }
}
