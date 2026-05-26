use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to,transfer,Mint, MintTo,Token, TokenAccount,Transfer},
};
use constant_product_curve::ConstantProduct;

use crate::{error::ErrorCode, state::Config};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint_x: Box<Account<'info,Mint>>,
    pub mint_y: Box<Account<'info,Mint>>,
    #[account(
        has_one=mint_x,
        has_one=mint_y,
        seeds = [b"config",config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info,Config>,
    #[account(
        mut,
        seeds = [b"lp",config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: Box<Account<'info,Mint>>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,)]
    pub vault_x: Box<Account<'info,TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,)]
    pub vault_y: Box<Account<'info,TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = signer,
    )]
    pub user_x: Box<Account<'info,TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = signer,
    )]
    pub user_y: Box<Account<'info,TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = signer,
    )]
    pub user_lp: Account<'info,TokenAccount>,
    pub token_program: Program<'info,Token>,
    pub associated_token_program: Program<'info,AssociatedToken>,
    pub system_program: Program<'info,System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(
        &mut self, 
        amount:u64, //Amount of Lp tokens to claim
        max_x:u64,  // Max amount of X that the user is willing to deposit
        max_y:u64 // Max amount of Y that the user is willing to deposit
    ) -> Result<()> {
        require!(!self.config.locked, ErrorCode::PoolLocked);
        require_neq!(amount,0,ErrorCode::InvalidAmount);

        let (x, y) = 
        if self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0 {
            (max_x,max_y)
        } else {
            let amounts  = ConstantProduct::xy_deposit_amounts_from_l(
                self.vault_x.amount, 
                self.vault_y.amount, 
                self.mint_lp.supply, 
                amount, 
                6
            )
            .unwrap();

            require!(amounts.x <= max_x && amounts.y <= max_y, ErrorCode::SlippageExceeded);
            (amounts.x,amounts.y)
        };

        //deposit tokens x and y from user to vault

        self.deposit_tokens(true, x)?;
        self.deposit_tokens(false, y)?;
        self.mint_lp_tokens(amount)

    }

    pub fn deposit_tokens(&self, is_x:bool, amount:u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.user_x.to_account_info(),
                self.vault_x.to_account_info(), 
            ),
            false => (
                self.user_y.to_account_info(),
                self.vault_y.to_account_info(), 
            )
        };

        let cpi_accounts = Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        let cpi_program = self.token_program.key();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)
    }

    pub fn mint_lp_tokens(&self, amount:u64) -> Result<()> {
        let cpi_accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let cpi_program = self.token_program.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        mint_to(cpi_ctx, amount)
    }
}
