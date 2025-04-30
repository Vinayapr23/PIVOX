use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Milestone {
    #[max_len(100)]
    pub description: String, // Max length must be bounded!
    pub amount: u64,
    pub freelancer_submitted: bool,
    pub client_approved: bool,
    pub freelancer_confirmed: bool,
    pub is_released: bool,
}

#[account]
#[derive(InitSpace)]
pub struct Contract {
    pub client: Pubkey,
    pub freelancer: Pubkey,
    pub vault_account: Pubkey,
    pub multisig_account: Pubkey,
    pub initial_payment: u64,
    pub project_start: i128,
    pub project_duration: u64,
    pub client_share: u8,
    pub freelancer_share: u8,
    #[max_len(400)]
    pub dispute_resolution: String,
    #[max_len(10)]
    pub status: String,
    pub contract_bump: u8,
    pub client_approved: bool,
    pub freelancer_approved: bool,
    #[max_len(10)]
    pub milestones: Vec<Milestone>,
}
