use {
    anchor_lang::{Id, InstructionData, ToAccountMetas, prelude::System}, anchor_spl::{associated_token::AssociatedToken, mint, token::Token}, solana_keypair::Keypair, solana_message::Instruction, solana_pubkey::Pubkey, solana_signer::Signer
};

pub fn deposit(
    payer: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    config: Pubkey,
    mint_lp: Pubkey,
    vault_x: Pubkey,
    vault_y: Pubkey,
    user_x: Pubkey,
    user_y: Pubkey,
    user_lp: Pubkey,
) -> Instruction {
    let signer = payer.pubkey();

    Instruction::new_with_bytes(
        amm::id(),
        &amm::instruction::Deposit {
            amount: 10,
            max_x: 30,
            max_y:30,
        }
        .data(),
        amm::accounts::Deposit {
            signer: signer,
            mint_x,
            mint_y,
            config,
            mint_lp,
            vault_x,
            vault_y,
            user_x,
            user_y,
            user_lp,
            token_program: Token::id(),
            associated_token_program: AssociatedToken::id(),
            system_program: System::id(),
        }
        .to_account_metas(None),
    )
}
