use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::state::{Contract, VaultAccount};

#[derive(Accounts)]
pub struct VaultWithdraw<'info> {
    /// CHECK
    #[account(mut)]
    pub client: AccountInfo<'info>,

    /// CHECK
    #[account(mut)]
    pub freelancer: AccountInfo<'info>,

    pub usdc_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"vault_account", client.key().as_ref(), freelancer.key().as_ref()],
        bump = vault_account.vault_bump,
    )]
    pub vault_account: Account<'info, VaultAccount>,

    #[account(
        mut,
        seeds = [b"contract", client.key().as_ref(), freelancer.key().as_ref()],
        bump = contract.contract_bump
    )]
    pub contract: Account<'info, Contract>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault_account,
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = freelancer,
    )]
    pub freelancer_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> VaultWithdraw<'info> {
    pub fn withdraw_funds(&mut self) -> Result<()> {
        if self.contract.status != "terminated" {
            return Err(error!(VaultWithdrawError::Unauthorized));
        }

        let remaining = self.vault_ata.amount;

        let client_key = self.client.key();
        let freelancer_key = self.freelancer.key();

        let seeds = &[
            b"vault_account",
            client_key.as_ref(),
            freelancer_key.as_ref(),
            &[self.vault_account.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.vault_ata.to_account_info(),
                to: self.freelancer_ata.to_account_info(),
                authority: self.vault_account.to_account_info(),
            },
            signer_seeds,
        );
        transfer(cpi_ctx, remaining)?;

        self.vault_account.balance = 0;
        Ok(())
    }
}

#[error_code]
pub enum VaultWithdrawError {
    #[msg("Contract not terminated yet.")]
    Unauthorized,
}
