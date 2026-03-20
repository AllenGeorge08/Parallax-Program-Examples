#[cfg(test)]
mod tests {
    use parallax::litesvm::TestHarness;
    use parallax::OracleBehaviour;
    use solana_instruction::Instruction;
    use solana_keypair::Keypair;

    use anchor_spl::{
        associated_token::{self, spl_associated_token_account},
        token::spl_token,
    };

    use super::*;
    use anchor_lang::AccountDeserialize;
    use anchor_lang::{InstructionData, ToAccountMetas};
    use litesvm_token::{
        spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
    };
    use litesvm::LiteSVM;
    use price_crash::accounts::Deposit;
    use price_crash::accounts::Withdraw;
    use price_crash::instruction::Deposit as DepositIx;
    use price_crash::instruction::Withdraw as WithdrawIx;
    use solana_message::Message;
    use solana_pubkey::Pubkey;
    use solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use std::fmt::Error;

    // fn setup() -> (Pubkey,Pubkey,Pubkey){
    //     let user = Keypair::new();
    //     let user_key = user.pubkey();
    //     let mut harness = TestHarness::new(&user);
    //     let seed = 1u64;
    //     harness.deploy_program();
    //     harness.send_tx(&user, 1000);

    //     let mint = harness.create_mint(&user);

    //     let mint_retrieved = harness.get_mint(&mint).unwrap_or_default();
    //     println!("Mint Created: {}",mint_retrieved);

    //     assert_eq!(mint,mint_retrieved);

    //     let user_ata = harness.create_ata(&user, &user, &mint);
    //     println!("Ata created succesfully for user Account : {:?} : \n ATA:  {:?}",user_ata,user.to_base58_string());

    //     let (oracle_pda, _bump) = OracleBehaviour::derive_oracle_pda(seed);
    //     let mut oracle = OracleBehaviour::new(&mut harness, oracle_pda);
    //     oracle.initialize_oracle(seed, 1_500_000, -3, 2, true);

    //     println!("Oracle initialized successfully");
    //     println!("Setup succesful..");

    //     (oracle_pda, mint, user_ata)
    // }

    pub struct TestSetup {
        pub user: Keypair,
        pub seed: u64,
        pub oracle_pda: Pubkey,
    }

    fn setup() -> TestSetup {
        let user = Keypair::new();
        let seed = 1u64;
        let (oracle_pda, _) = OracleBehaviour::derive_oracle_pda(seed);

        TestSetup {
            user,
            seed,
            oracle_pda,
        }
    }

    #[test]
    fn test_price_crash() {
        let mut program = LiteSVM::new();
        let s = setup();
        let mut harness = TestHarness::new(&s.user);
        harness.deploy_program();
        harness.send_tx(&s.user, 1000);

        let mint = harness.create_mint(&s.user);
        let user_ata = harness.create_ata(&s.user, &s.user, &mint);
        harness.mint_to(user_ata, 1_000_000, &mint);

        //Creating and initializing oracle account..

        //Deposit ix
        //user, user_ata,mint_account, vault,vault_ata,oracle,token_program,ata program,sysProgram
        println!("Mint Account Created: {:?}", mint);
        println!("User ATA Account Created: {:?}", user_ata);

        //Deriving vault pda and ata..
        let (vault_pda, bump) = Pubkey::find_program_address(
            &[b"vault", s.user.pubkey().as_ref()],
            &price_crash::ID,
        );

        let vault_ata =
            spl_associated_token_account::get_associated_token_address(&vault_pda, &mint);

        println!("Vault PDA is: {:?}", vault_pda);
        println!("Vault ATA is: {:?}", vault_ata);

        
        let mut oracle = OracleBehaviour::new(&mut harness, s.oracle_pda);
        oracle.initialize_oracle(s.seed, 1_500_000, -3, 2, true);
        let (exponent, mantissa) = oracle.read_oracle();
        println!("Current Exponent  : {:?}", exponent);
        println!("Current Mantissa  : {:?}", mantissa);
        

        let deposit_ix = Instruction {
            program_id: price_crash::ID.to_bytes().into(),
            accounts: Deposit {
                user: s.user.pubkey(),
                user_ata: user_ata,
                mint_account: mint,
                vault: vault_pda,
                vault_ata: vault_ata,
                oracle: s.oracle_pda,
                associated_token_program: spl_associated_token_account::ID,
                token_program: TOKEN_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: DepositIx { amount: 15 }.data(),
        };

        let message = Message::new(&[deposit_ix], Some(&s.user.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&s.user], message, recent_blockhash);
        let final_tx = program.send_transaction(transaction).unwrap_or_default();

        //  from: i64,
        // to: i64,
        // slots: u64,
        // confidence: u64,
        // price_exponent: i32,
        // seed: u64,

        //  1_500_000, -3, 2,

        println!("Price crashes over 5 slots...");
        oracle.price_crash(1_500_000,9_000_000,5,2,-3,1u64);
        let (exponent, mantissa) = oracle.read_oracle();
        println!("Current Exponent after crash  : {:?}", exponent);
        println!("Current Mantissa after crash : {:?}", mantissa);

     

        let withdraw_ix = Instruction {
            program_id: price_crash::ID.to_bytes().into(),
            accounts: Withdraw {
                user: s.user.pubkey(),
                user_ata: user_ata,
                mint_account: mint,
                vault: vault_pda,
                vault_ata: vault_ata,
                oracle: s.oracle_pda,
                associated_token_program: spl_associated_token_account::ID,
                token_program: TOKEN_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: WithdrawIx { requested_units: 15 }.data(),
        };

        let message = Message::new(&[withdraw_ix], Some(&s.user.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&s.user], message, recent_blockhash);
        let final_tx = program.send_transaction(transaction).unwrap_or_default();

        println!("Withdraw tx succesful");
    }
}
