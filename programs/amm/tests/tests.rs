use {
    anchor_spl::associated_token,
    litesvm::LiteSVM,
    litesvm_token::CreateMint,
    litesvm_token::MintTo,
    litesvm_token::Transfer,
    litesvm_token::CreateAssociatedTokenAccount,
    solana_message::{Instruction,Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
    solana_pubkey::Pubkey,
    amm::state::Config,
    anchor_lang::AccountDeserialize,
    anchor_spl::token::TokenAccount,
};

mod ix_handlers;
use ix_handlers::*;


fn send(
    svm: &mut LiteSVM,
    ixs:&[Instruction],
    payer: &Keypair,
    signers: &[&dyn Signer]
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}


fn setup() -> (
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,

) {
    let program_id = amm::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = std::fs::read("../../target/deploy/amm.so").expect("build the program first: `anchor build` or `cargo build-bpf`");
    svm.add_program(program_id, bytes.as_slice()).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mint_x = CreateMint::new(&mut svm, &payer)
    .decimals(6)
    .authority(&payer.pubkey())
    .send()
    .unwrap();


    let mint_y = CreateMint::new(&mut svm, &payer)
    .decimals(6)
    .authority(&payer.pubkey())
    .send()
    .unwrap();

    let config = Pubkey::find_program_address(
        &[b"config", 0u64.to_le_bytes().as_ref()],
        &program_id,
    ).0;

    let mint_lp = Pubkey::find_program_address(
        &[b"lp",config.as_ref()],
        &program_id,
    ).0;

    let vault_x = associated_token::get_associated_token_address(
        &config,
        &mint_x,
    );

    let vault_y = associated_token::get_associated_token_address(
        &config,
        &mint_y,
    );      


    (
        svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
    )
}

#[test]
fn test_initialize() {
    let (
        mut svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
    ) = setup();

    let ix = create_initialize_ix(
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y
    );

    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let account = svm.get_account(&config).unwrap();
    let config_state = Config::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(config_state.fee, 30);
    assert_eq!(config_state.seed, 0);
    assert_eq!(config_state.mint_x, mint_x);
    assert_eq!(config_state.mint_y, mint_y);
    assert_eq!(config_state.locked, false);
    assert_eq!(config_state.authority, Some(payer.pubkey()));
}

// Additional tests mirrored from amm_turbine

#[test]
fn test_deposit() {
    let (
        mut svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
    ) = setup();

    let ix_init = create_initialize_ix(&payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y);
    send(&mut svm, &[ix_init], &payer, &[&payer]).unwrap();

    let user = payer.pubkey();

    let user_x = CreateAssociatedTokenAccount::new(
        &mut svm, 
        &payer,
        &mint_x)
        .owner(&user)
        .send()
        .unwrap();


    let user_y = CreateAssociatedTokenAccount::new(
        &mut svm, 
        &payer,
        &mint_y)
        .owner(&user)
        .send()
        .unwrap();
    

    let user_lp = CreateAssociatedTokenAccount::new(
        &mut svm, 
        &payer,
        &mint_lp)
        .owner(&user)
        .send()
        .unwrap();

    MintTo::new(
        &mut svm,
        &payer,
        &mint_x,
        &user_x,
        1000
    )
    .send()
    .unwrap();

    MintTo::new(
        &mut svm,
        &payer,
        &mint_y,
        &user_y,
        1000
    )
    .send()
    .unwrap();

    let ix = deposit(
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        user_x,
        user_y,
        user_lp
    );

    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let vault_x_account = svm.get_account(&vault_x).unwrap();
    let vault_x_state = TokenAccount::try_deserialize(&mut vault_x_account.data.as_slice()).unwrap();
    assert_eq!(vault_x_state.amount, 30);

    let vault_y_account = svm.get_account(&vault_y).unwrap();
    let vault_y_state = TokenAccount::try_deserialize(&mut vault_y_account.data.as_slice()).unwrap();
    assert_eq!(vault_y_state.amount, 30);

    let user_lp_account = svm.get_account(&user_lp).unwrap();
    let user_lp_state = TokenAccount::try_deserialize(&mut user_lp_account.data.as_slice()).unwrap();
    assert_eq!(user_lp_state.amount, 10);

    let user_x_account = svm.get_account(&user_x).unwrap();
    let user_x_state = TokenAccount::try_deserialize(&mut user_x_account.data.as_slice()).unwrap();
    assert_eq!(user_x_state.amount, 970);
    let user_y_account = svm.get_account(&user_y).unwrap();
    let user_y_state = TokenAccount::try_deserialize(&mut user_y_account.data.as_slice()).unwrap();
    assert_eq!(user_y_state.amount, 970);
}


#[test]
fn test_withdraw() {

    let (
        mut svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
    ) = setup();

    let ix_init = create_initialize_ix(&payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y);
    send(&mut svm, &[ix_init], &payer, &[&payer]).unwrap();


    let user = payer.pubkey();

    let user_x = CreateAssociatedTokenAccount::new(
        &mut svm, 
        &payer,
        &mint_x)
        .owner(&user)
        .send()
        .unwrap();


    let user_y = CreateAssociatedTokenAccount::new(
        &mut svm, 
        &payer,
        &mint_y)
        .owner(&user)
        .send()
        .unwrap();
    

    let user_lp = CreateAssociatedTokenAccount::new(
        &mut svm, 
        &payer,
        &mint_lp)
        .owner(&user)
        .send()
        .unwrap();

    MintTo::new(
        &mut svm,
        &payer,
        &mint_x,
        &user_x,
        1000
    )
    .send()
    .unwrap();

    MintTo::new(
        &mut svm,
        &payer,
        &mint_y,
        &user_y,
        1000
    )
    .send()
    .unwrap();

    let deposit_ix = deposit(
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        user_x,
        user_y,
        user_lp
    );

    send(&mut svm, &[deposit_ix], &payer, &[&payer]).unwrap();


    let withdraw_ix = withdraw(
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        user_x,
        user_y,
        user_lp
    );

    let res = send(&mut svm, &[withdraw_ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let vault_x_account = svm.get_account(&vault_x).unwrap();
    let vault_x_state = TokenAccount::try_deserialize(&mut vault_x_account.data.as_slice()).unwrap();
    assert_eq!(vault_x_state.amount, 0);
    let vault_y_account = svm.get_account(&vault_y).unwrap();
    let vault_y_state = TokenAccount::try_deserialize(&mut vault_y_account.data.as_slice()).unwrap();
    assert_eq!(vault_y_state.amount, 0);
    let user_lp_account = svm.get_account(&user_lp).unwrap();
    let user_lp_state = TokenAccount::try_deserialize(&mut user_lp_account.data.as_slice()).unwrap();
    assert_eq!(user_lp_state.amount, 0);

    let user_x_account = svm.get_account(&user_x).unwrap();
    let user_x_state = TokenAccount::try_deserialize(&mut user_x_account.data.as_slice()).unwrap();
    assert_eq!(user_x_state.amount, 1000);
    let user_y_account = svm.get_account(&user_y).unwrap();
    let user_y_state = TokenAccount::try_deserialize(&mut user_y_account.data.as_slice()).unwrap();
    assert_eq!(user_y_state.amount, 1000);
}


#[test]
fn test_swap() {

    let (
        mut svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
    ) = setup();

    let ix_init = create_initialize_ix(&payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y);
    send(&mut svm, &[ix_init], &payer, &[&payer]).unwrap();

    let user = payer.pubkey();

    let user_x = CreateAssociatedTokenAccount::new(
        &mut svm, 
        &payer,
        &mint_x)
        .owner(&user)
        .send()
        .unwrap();


    let user_y = CreateAssociatedTokenAccount::new(
        &mut svm, 
        &payer,
        &mint_y)
        .owner(&user)
        .send()
        .unwrap();
    

    let user_lp = CreateAssociatedTokenAccount::new(
        &mut svm, 
        &payer,
        &mint_lp)
        .owner(&user)
        .send()
        .unwrap();

    MintTo::new(
        &mut svm,
        &payer,
        &mint_x,
        &user_x,
        1000
    )
    .send()
    .unwrap();

    MintTo::new(
        &mut svm,
        &payer,
        &mint_y,
        &user_y,
        1000
    )
    .send()
    .unwrap();

    let deposit_ix = deposit(
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        user_x,
        user_y,
        user_lp
    );

    let res = send(&mut svm, &[deposit_ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let vault_x_account = svm.get_account(&vault_x).unwrap();
    let vault_x_state = TokenAccount::try_deserialize(&mut vault_x_account.data.as_slice()).unwrap();
    assert_eq!(vault_x_state.amount, 30);

    let vault_y_account = svm.get_account(&vault_y).unwrap();
    let vault_y_state = TokenAccount::try_deserialize(&mut vault_y_account.data.as_slice()).unwrap();
    assert_eq!(vault_y_state.amount, 30);

    let user_lp_account = svm.get_account(&user_lp).unwrap();
    let user_lp_state = TokenAccount::try_deserialize(&mut user_lp_account.data.as_slice()).unwrap();
    assert_eq!(user_lp_state.amount, 10);

    let user_x_account = svm.get_account(&user_x).unwrap();
    let user_x_state = TokenAccount::try_deserialize(&mut user_x_account.data.as_slice()).unwrap();
    assert_eq!(user_x_state.amount, 970);
    let user_y_account = svm.get_account(&user_y).unwrap();
    let user_y_state = TokenAccount::try_deserialize(&mut user_y_account.data.as_slice()).unwrap();
    assert_eq!(user_y_state.amount, 970);


    let swap_ix = swap (
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        user_x,
        user_y,
        user_lp
    );

    let res = send(&mut svm, &[swap_ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let vault_x_account = svm.get_account(&vault_x).unwrap();
    let vault_x_state = TokenAccount::try_deserialize(&mut vault_x_account.data.as_slice()).unwrap();
    assert_eq!(vault_x_state.amount, 32);
    let vault_y_account = svm.get_account(&vault_y).unwrap();
    let vault_y_state = TokenAccount::try_deserialize(&mut vault_y_account.data.as_slice()).unwrap();
    assert_eq!(vault_y_state.amount, 29);

    let user_x_account = svm.get_account(&user_x).unwrap();
    let user_x_state = TokenAccount::try_deserialize(&mut user_x_account.data.as_slice()).unwrap();
    assert_eq!(user_x_state.amount, 968); 
    let user_y_account = svm.get_account(&user_y).unwrap();
    let user_y_state = TokenAccount::try_deserialize(&mut user_y_account.data.as_slice()).unwrap();
    assert_eq!(user_y_state.amount, 971);      
}


#[test]
fn test_lock() {
    let (
        mut svm,
        payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
    ) = setup();

    let init_ix = create_initialize_ix(
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y
    );

    send(&mut svm, &[init_ix], &payer, &[&payer]).unwrap();

    let lock_ix = lock(
        &payer,
        config,
        true
    );

    let res = send(&mut svm, &[lock_ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let account = svm.get_account(&config).unwrap();
    let config_state = Config::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(config_state.locked, true);
}
