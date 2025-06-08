use anchor_lang::InstructionData;
#[cfg(test)]
use mollusk_svm::{program, result::Check, Mollusk};
use solana_sdk::{
    account::{Account, WritableAccount},
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use vault_program;

#[test]

fn test_initialize() {
    let (system_program, system_account) = program::keyed_account_for_system_program();

    let program_id = Pubkey::new_from_array(five8_const::decode_32_const(
        "AToi9NHsoSohi319YvQ1mgbeby2GbneaSupF7YUFApuv",
    ));

    let mollusk = Mollusk::new(&program_id, "../../target/deploy/vault_program");

    let user = Keypair::new();

    let (state_pda, _bump) =
        Pubkey::find_program_address(&[b"state".as_ref(), user.pubkey().as_ref()], &program_id);

    let (vault_pda, _vault_bump) =
        Pubkey::find_program_address(&["vault".as_ref(), state_pda.as_ref()], &program_id);

    let user_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);
    let state_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

    let vault_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

    let ix_accounts = vec![
        AccountMeta::new(user.pubkey(), true),
        AccountMeta::new(state_pda, false),
        AccountMeta::new(vault_pda, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let data = (vault_program::instruction::Initialize {}).data();

    let ix = Instruction::new_with_bytes(program_id, &data, ix_accounts);

    let tx_account = &vec![
        (user.pubkey(), user_account.clone()),
        (state_pda, state_account.clone()),
        (vault_pda, vault_account.clone()),
        (system_program, system_account.clone().into()),
    ];

    let _init_result =
        mollusk.process_and_validate_instruction(&ix, tx_account, &[Check::success()]);
}

#[test]
fn test_deposit() {
    //system_program account
    let (system_program, system_account) = mollusk_svm::program::keyed_account_for_system_program();

    //five8_const decodes from base58-encoded string to an 32-byte array [u8, 32] we could as easily used crare bs58 but that would be more lines of code
    let program_id = Pubkey::new_from_array(five8_const::decode_32_const(
        "AToi9NHsoSohi319YvQ1mgbeby2GbneaSupF7YUFApuv",
    ));

    //Initialize Mollusk (Program ID + Program's BPF)
    let mollusk = Mollusk::new(&program_id, "../../target/deploy/vault_program");

    //Keypair for user
    let user = Keypair::new();

    //Derive vault state PDA
    let (state_pda, state_bump) =
        Pubkey::find_program_address(&["state".as_ref(), user.pubkey().as_ref()], &program_id);

    //Derive Vault PDA
    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&["vault".as_ref(), state_pda.as_ref()], &program_id);

    //Initialize Acounts
    let user_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

    //In deposit instruction account has been initialized so we need give minimum balance and the state data
    let state_account_size = 8 + 8 + 8;
    let mut state_account = Account::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(state_account_size.clone()),
        state_account_size,
        &program_id,
    );
    // Serialize the initial VaultState data into the account's data buffer
    let initial_state = vault_program::VaultState {
        vault_bump,
        state_bump,
    };
    let mut state_data = state_account.data_as_mut_slice();
    anchor_lang::AccountSerialize::try_serialize(&initial_state, &mut state_data)
        .expect("Failed to serialize state account data");

    let vault_account = Account::new(0, 0, &system_program);

    //--------------- Deposit Funds and test it -----------------
    let deposit_accounts = vec![
        AccountMeta::new(user.pubkey(), true),
        AccountMeta::new(state_pda, false),
        AccountMeta::new(vault_pda, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let transfer_amount = 500_000_000;
    let data = (vault_program::instruction::Deposit {
        amount: transfer_amount,
    })
    .data();

    let deposit_instruction = Instruction::new_with_bytes(program_id, &data, deposit_accounts);

    let tx2_accounts = &vec![
        (user.pubkey(), user_account),
        (state_pda, state_account),
        (vault_pda, vault_account),
        (system_program, system_account.into()),
    ];

    let user_bind = &user.pubkey();
    let checks = &vec![
        Check::success(),
        Check::account(user_bind).lamports(transfer_amount).build(),
    ];

    let deposit_result =
        mollusk.process_and_validate_instruction(&deposit_instruction, tx2_accounts, checks);
}
fn test_withdraw() {
    //system_program account
    let (system_program, system_account) = program::keyed_account_for_system_program();

    let program_id = Pubkey::new_from_array(five8_const::decode_32_const(
        "AToi9NHsoSohi319YvQ1mgbeby2GbneaSupF7YUFApuv",
    ));

    //Initialize Mollusk (Program ID + Program's BPF)
    let mollusk = Mollusk::new(&program_id, "../../target/deploy/vault_program");

    //Keypair for user
    let user = Keypair::new();

    //Derive vault state PDA
    let (state_pda, state_bump) =
        Pubkey::find_program_address(&["state".as_ref(), user.pubkey().as_ref()], &program_id);

    //Derive Vault PDA
    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&["vault".as_ref(), state_pda.as_ref()], &program_id);

    //Initialize Acounts
    let user_account = Account::new(500_000_000, 0, &system_program);

    //In deposit instruction account has been initialized so we need give minimum balance and the state data
    let state_account_size = 8 + 8 + 8;
    let mut state_account = Account::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(state_account_size.clone()),
        state_account_size,
        &program_id,
    );
    // Serialize the initial VaultState data into the account's data buffer
    let initial_state = vault_program::VaultState {
        vault_bump,
        state_bump,
    };

    //Get data allocated in state_account
    let mut state_data = state_account.data_as_mut_slice();

    //Serialize and add data to the alocated space
    anchor_lang::AccountSerialize::try_serialize(&initial_state, &mut state_data)
        .expect("Failed to serialize state account data");

    let vault_account = Account::new(500_000_000, 0, &system_program);

    //--------------- Deposit Funds and test it -----------------
    let withdraw_accounts = vec![
        AccountMeta::new(user.pubkey(), true),
        AccountMeta::new(state_pda, false),
        AccountMeta::new(vault_pda, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let transfer_amount = 500_000_000;
    let data = (vault_program::instruction::Withdraw {
        amount: transfer_amount,
    })
    .data();

    let deposit_instruction = Instruction::new_with_bytes(program_id, &data, withdraw_accounts);

    let tx_accounts = &vec![
        (user.pubkey(), user_account),
        (state_pda, state_account),
        (vault_pda, vault_account),
        (system_program, system_account.into()),
    ];

    let vault_bind = &vault_pda;
    let checks = &vec![
        Check::success(),
        Check::account(vault_bind).lamports(0).build(),
    ];

    let _withdraw_result =
        mollusk.process_and_validate_instruction(&deposit_instruction, tx_accounts, checks);
}

#[test]
fn test_close() {
    //system_program account
    let (system_program, system_account) = mollusk_svm::program::keyed_account_for_system_program();

    //five8_const decodes from base58-encoded string to an 32-byte array [u8, 32] we could as easily used crare bs58 but that would be more lines of code
    let program_id = Pubkey::new_from_array(five8_const::decode_32_const(
        "AToi9NHsoSohi319YvQ1mgbeby2GbneaSupF7YUFApuv",
    ));

    //Initialize Mollusk (Program ID + Program's BPF)
    let mollusk = Mollusk::new(&program_id, "../../target/deploy/vault_program");

    //Keypair for user
    let user = Keypair::new();

    //Derive vault state PDA
    let (state_pda, state_bump) =
        Pubkey::find_program_address(&["state".as_ref(), user.pubkey().as_ref()], &program_id);

    //Derive Vault PDA
    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&["vault".as_ref(), state_pda.as_ref()], &program_id);

    //Initialize Acounts
    let user_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

    //In deposit instruction account has been initialized so we need give minimum balance and the state data
    let state_account_size = 8 + 8 + 8;
    let mut state_account = Account::new(
        mollusk
            .sysvars
            .rent
            .minimum_balance(state_account_size.clone()),
        state_account_size,
        &program_id,
    );
    // Serialize the initial VaultState data into the account's data buffer
    let initial_state = vault_program::VaultState {
        vault_bump,
        state_bump,
    };
    let mut state_data = state_account.data_as_mut_slice();
    anchor_lang::AccountSerialize::try_serialize(&initial_state, &mut state_data)
        .expect("Failed to serialize state account data");

    let vault_account = Account::new(0, 0, &system_program);

    //--------------- Deposit Funds and test it -----------------
    let close_accounts = vec![
        AccountMeta::new(user.pubkey(), true),
        AccountMeta::new(state_pda, false),
        AccountMeta::new(vault_pda, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let data = (vault_program::instruction::Close {}).data();

    let deposit_instruction = Instruction::new_with_bytes(program_id, &data, close_accounts);

    let tx_accounts = &vec![
        (user.pubkey(), user_account),
        (state_pda, state_account),
        (vault_pda, vault_account),
        (system_program, system_account.into()),
    ];

    let vault_bind = &vault_pda;
    let checks = &vec![
        Check::success(),
        Check::account(vault_bind).closed().build(),
    ];

    let _close_result =
        mollusk.process_and_validate_instruction(&deposit_instruction, tx_accounts, checks);
}
