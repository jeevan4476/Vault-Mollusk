#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};
declare_id!("AToi9NHsoSohi319YvQ1mgbeby2GbneaSupF7YUFApuv");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + VaultState::INIT_SPACE,
        seeds = [b"state".as_ref(), user.key().as_ref()],
        bump
    )]
    pub state: Account<'info, VaultState>,

    // this is not a regular PDA, but a system account. The vault will only store lamports and not spl
    //Don't need to Init this account, you need to transfer lamports to it and the system program will initialize it
    #[account(seeds = [b"vault".as_ref(), state.key().as_ref()], bump)]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.state.vault_bump = bumps.vault;
        self.state.state_bump = bumps.state;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(seeds = [b"state".as_ref(), user.key().as_ref()], bump = state.state_bump)]
    pub state: Account<'info, VaultState>,

    #[account(
            mut,
            seeds = [b"vault".as_ref(), state.key().as_ref()],
            bump = state.vault_bump,
        )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        system_program::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(seeds = [b"state".as_ref(), user.key().as_ref()], bump = state.state_bump)]
    pub state: Account<'info, VaultState>,

    #[account(
            mut,
            seeds = [b"vault".as_ref(), state.key().as_ref()],
            bump = state.vault_bump,
        )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let seeds = &[
            b"vault",
            self.state.to_account_info().key.as_ref(),
            &[self.state.vault_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        system_program::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(seeds = [b"state".as_ref(), user.key().as_ref()], bump = state.state_bump)]
    pub state: Account<'info, VaultState>,

    #[account(
            mut,
            seeds = [b"vault".as_ref(), state.key().as_ref()],
            bump = state.vault_bump,
            //cant close this account with close constraint as it is a SystemAccount and not PDA
        )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {
        let seeds = &[
            b"vault",
            self.state.to_account_info().key.as_ref(),
            &[self.state.vault_bump],
        ];

        let signer_seeds = &[&seeds[..]];
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        system_program::transfer(cpi_ctx, self.vault.lamports())?;
        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}
