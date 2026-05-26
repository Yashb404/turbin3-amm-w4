use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, Burn, Mint, Token, TokenAccount, Transfer, transfer},
};
use constant_product_curve::ConstantProduct;

use crate::{error::ErrorCode, state::Config};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

impl<'info> Withdraw<'info> {
    pub fn withdraw(
        &mut self,
        amount:u64, //Amount of Lp tokens that the user wants to burn in exchange for x and y
        min_x:u64,  // Min amount of X that the user is willing to receive
        min_y:u64 // Min amount of Y that the user is willing to receive
    ) -> Result<()> {
        require!(!self.config.locked, ErrorCode::PoolLocked);
        require_neq!(amount,0,ErrorCode::InvalidAmount);

        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            amount,
            6
        )
        .unwrap();

        require!(amounts.x >= min_x && amounts.y >= min_y, ErrorCode::SlippageExceeded);
        let (x, y) = (amounts.x, amounts.y);

        self.burn_lp_tokens(amount)?;
        self.withdraw_tokens(true, x)?;
        self.withdraw_tokens(false, y)
    }

    pub fn withdraw_tokens(&self, is_x:bool, amount:u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
            ),
            false => (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
            )
        };

        let cpi_accounts = Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"config",
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ]];

        let cpi_program = self.token_program.key();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer(cpi_ctx, amount)
    }

    pub fn burn_lp_tokens(&self, amount:u64) -> Result<()> {
        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.signer.to_account_info(),
        };
        let cpi_program = self.token_program.key();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        burn(cpi_ctx, amount)
    }
}
