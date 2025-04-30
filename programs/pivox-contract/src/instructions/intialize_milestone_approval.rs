use anchor_lang::prelude::*;
use crate::state::MilestoneApproval;

#[derive(Accounts)]
pub struct InitializeMilestoneApproval<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK
    pub client: AccountInfo<'info>,

    /// CHECK
    pub freelancer: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + MilestoneApproval::INIT_SPACE,
        seeds = [b"milestone_approval", client.key().as_ref(), freelancer.key().as_ref()],
        bump
    )]
    pub milestone_approval: Account<'info, MilestoneApproval>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeMilestoneApproval<'info> {
    pub fn initialize(&mut self, threshold: u16) -> Result<()> {
        let milestone_approval = &mut self.milestone_approval;

        milestone_approval.client = self.client.key();
        milestone_approval.freelancer = self.freelancer.key();
        milestone_approval.threshold = threshold;
        milestone_approval.approvals = 0;
        milestone_approval.approved_by = vec![];
        milestone_approval.is_signed = false;

        Ok(())
    }
}
