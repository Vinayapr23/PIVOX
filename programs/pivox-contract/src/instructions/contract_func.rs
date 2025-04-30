use crate::state::{Contract, VaultAccount};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, close_account, transfer, CloseAccount, Token, TokenAccount, Transfer},
};

#[error_code]
pub enum ContractError {
    Unauthorized,
    MilestoneError,
    AlreadyReleased,
}

#[derive(Accounts)]
pub struct ContractFunc<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"contract", contract.client.as_ref(), contract.freelancer.as_ref()],
        bump = contract.contract_bump
    )]
    pub contract: Account<'info, Contract>,

    #[account(
        mut,
        seeds = [b"vault_account", contract.client.as_ref(), contract.freelancer.as_ref()],
        bump = vault_account.vault_bump
    )]
    pub vault_account: Account<'info, VaultAccount>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault_account
    )]
    pub vault_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = contract.freelancer
    )]
    pub freelancer_ata: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, token::Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ContractFunc<'info> {
    pub fn complete_or_cancel_contract(&mut self) -> Result<()> {
        let contract = &mut self.contract;
        let signer_key = self.signer.key();

        if signer_key == contract.client {
            contract.client_approved = true;
        } else if signer_key == contract.freelancer {
            contract.freelancer_approved = true;
        } else {
            return Err(error!(ContractError::Unauthorized));
        }

        if contract.client_approved && contract.freelancer_approved {
            contract.status = "terminated".to_string();

            let seeds = &[
                b"vault_account",
                contract.client.as_ref(),
                contract.freelancer.as_ref(),
                &[self.vault_account.vault_bump],
            ];
            let signer_seeds = &[&seeds[..]];

            if self.vault_ata.amount > 0 {
                let cpi_ctx = CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    Transfer {
                        from: self.vault_ata.to_account_info(),
                        to: self.freelancer_ata.to_account_info(),
                        authority: self.vault_account.to_account_info(),
                    },
                    signer_seeds,
                );
                transfer(cpi_ctx, self.vault_ata.amount)?;
            }

            let cpi_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                CloseAccount {
                    account: self.vault_ata.to_account_info(),
                    destination: self.signer.to_account_info(),
                    authority: self.vault_account.to_account_info(),
                },
                signer_seeds,
            );
            close_account(cpi_ctx)?;

            self.vault_account.balance = 0;
            self.vault_account.vault_status = "Terminated".to_string();
        }

        Ok(())
    }

    pub fn release_milestone_payment(&mut self, milestone_index: u64) -> Result<()> {
        let milestone_index = milestone_index as usize;
    
        let contract = &mut self.contract;
    
        require!(
            milestone_index < contract.milestones.len(),
            ContractError::MilestoneError
        );
    
        let client_key = contract.client.clone();
        let freelancer_key = contract.freelancer.clone();
        let milestone = &mut contract.milestones[milestone_index];
    
        require!(milestone.freelancer_submitted, ContractError::MilestoneError);
        require!(milestone.client_approved, ContractError::MilestoneError);
        require!(!milestone.is_released, ContractError::AlreadyReleased);
    
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
    
        transfer(cpi_ctx, milestone.amount)?;
    
        milestone.is_released = true;
        milestone.freelancer_confirmed = true;
    
        self.vault_account.balance = self
            .vault_account
            .balance
            .saturating_sub(milestone.amount);
    
        Ok(())
    }
    
}
