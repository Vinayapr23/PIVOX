use anchor_lang::prelude::*;
use crate::state::Contract;

#[derive(Accounts)]
pub struct MilestoneLifecycle<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"contract", contract.client.as_ref(), contract.freelancer.as_ref()],
        bump = contract.contract_bump
    )]
    pub contract: Account<'info, Contract>,
}

#[error_code]
pub enum MilestoneError {
    InvalidSigner,
    AlreadySubmitted,
    AlreadyApproved,
    AlreadyConfirmed,
    NotSubmitted,
    NotApproved,
}

impl<'info> MilestoneLifecycle<'info> {
    pub fn freelancer_submit(&mut self, milestone_index: u64) -> Result<()> {
        let milestone_index = milestone_index as usize;

        let freelancer_key = self.contract.freelancer;

        let milestone = self
            .contract
            .milestones
            .get_mut(milestone_index)
            .ok_or(MilestoneError::NotSubmitted)?;

        require!(
            self.signer.key() == freelancer_key,
            MilestoneError::InvalidSigner
        );
        require!(!milestone.freelancer_submitted, MilestoneError::AlreadySubmitted);

        milestone.freelancer_submitted = true;
        Ok(())
    }

    pub fn client_approve(&mut self, milestone_index: u64) -> Result<()> {
        let milestone_index = milestone_index as usize;

        let client_key = self.contract.client;

        let milestone = self
            .contract
            .milestones
            .get_mut(milestone_index)
            .ok_or(MilestoneError::NotSubmitted)?;

        require!(
            self.signer.key() == client_key,
            MilestoneError::InvalidSigner
        );
        require!(milestone.freelancer_submitted, MilestoneError::NotSubmitted);
        require!(!milestone.client_approved, MilestoneError::AlreadyApproved);

        milestone.client_approved = true;
        Ok(())
    }

    pub fn freelancer_confirm(&mut self, milestone_index: u64) -> Result<()> {
        let milestone_index = milestone_index as usize;

        let freelancer_key = self.contract.freelancer;

        let milestone = self
            .contract
            .milestones
            .get_mut(milestone_index)
            .ok_or(MilestoneError::NotSubmitted)?;

        require!(
            self.signer.key() == freelancer_key,
            MilestoneError::InvalidSigner
        );
        require!(milestone.freelancer_submitted, MilestoneError::NotSubmitted);
        require!(milestone.client_approved, MilestoneError::NotApproved);
        require!(!milestone.freelancer_confirmed, MilestoneError::AlreadyConfirmed);

        milestone.freelancer_confirmed = true;
        Ok(())
    }
}
