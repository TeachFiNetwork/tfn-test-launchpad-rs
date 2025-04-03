#![no_std]

multiversx_sc::imports!();

pub mod common;

use common::{config::*, errors::*};
use tfn_launchpad::common::{errors::*, consts::*};
use tfn_platform::common::config::ProxyTrait as _;

#[multiversx_sc::contract]
pub trait TFNTestLaunchpadContract<ContractReader>:
    common::config::ConfigModule
{
    #[init]
    fn init(&self) {
        let caller = self.blockchain().get_caller();
        if self.blockchain().is_smart_contract(&caller) {
            self.platform_sc().set(caller);
            self.set_state_active();
        }
    }

    #[upgrade]
    fn upgrade(&self) {
    }

    #[endpoint(newLaunchpad)]
    fn new_launchpad(
        &self,
        description: ManagedBuffer,
        token: TokenIdentifier,
        payment_token: TokenIdentifier,
        price: BigUint, // if payment token is USDC (6 decimals), price should be x_000_000
        min_buy_amount: BigUint,
        max_buy_amount: BigUint,
        start_time: u64,
        end_time: u64
    ) -> u64 {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        self.check_whitelisted(&caller);

        require!(price > 0, ERROR_ZERO_PRICE);
        require!(min_buy_amount <= max_buy_amount, ERROR_WRONG_MIN_MAX_AMOUNTS);

        let now = self.blockchain().get_block_timestamp();
        require!(now < start_time, ERROR_WRONG_START_TIME);
        require!(start_time < end_time, ERROR_WRONG_END_TIME);

        require!(self.token_launchpad_id(&token).is_empty(), ERROR_TOKEN_ALREADY_LAUNCHED);

        let launchpad = Launchpad{
            id: self.last_launchpad_id().get(),
            owner: caller,
            description,
            token: token.clone(),
            amount: BigUint::zero(),
            payment_token,
            price,
            min_buy_amount,
            max_buy_amount,
            start_time,
            end_time,
            total_raised: BigUint::zero(),
            total_sold: BigUint::zero(),
            launched: false,
            status: Status::Pending,
            user_bought: BigUint::zero(),
        };
        self.launchpads(launchpad.id).set(&launchpad);
        self.token_launchpad_id(&token).set(launchpad.id);
        self.last_launchpad_id().set(launchpad.id + 1);

        launchpad.id
    }

    #[payable("*")]
    #[endpoint(addTokens)]
    fn add_tokens(&self, id: u64) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(!self.launchpads(id).is_empty(), ERROR_LAUNCHPAD_NOT_FOUND);

        let mut launchpad = self.launchpads(id).get();
        require!(launchpad.end_time > self.blockchain().get_block_timestamp(), ERROR_LAUNCHPAD_INACTIVE);

        let payment = self.call_value().single_esdt();
        require!(launchpad.token == payment.token_identifier, ERROR_WRONG_TOKEN);

        launchpad.amount += payment.amount;
        self.launchpads(id).set(launchpad);
    }

    #[endpoint(cancelLaunchpad)]
    fn cancel_launchpad(&self, id: u64) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(!self.launchpads(id).is_empty(), ERROR_LAUNCHPAD_NOT_FOUND);

        let launchpad = self.launchpads(id).get();
        require!(launchpad.owner == self.blockchain().get_caller(), ERROR_ONLY_LAUNCHPAD_OWNER);
        require!(launchpad.total_sold == 0, ERROR_DELETING_LAUNCHPAD);

        self.launchpads(id).clear();
        self.token_launchpad_id(&launchpad.token).clear();

        if launchpad.amount > 0 {
            self.send().direct_esdt(
                &launchpad.owner,
                &launchpad.token,
                0,
                &launchpad.amount
            );
        }
    }

    #[payable("*")]
    #[endpoint(buy)]
    fn buy(&self, id: u64) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(!self.launchpads(id).is_empty(), ERROR_LAUNCHPAD_NOT_FOUND);

        let mut launchpad = self.launchpads(id).get();
        require!(launchpad.is_active(self.blockchain().get_block_timestamp()), ERROR_LAUNCHPAD_INACTIVE);

        let payment = self.call_value().single_esdt();
        require!(payment.token_identifier == launchpad.payment_token, ERROR_WRONG_TOKEN);

        let token_amount = &payment.amount * ONE / &launchpad.price;
        let caller = self.blockchain().get_caller();
        let old_bought_amount = self.user_participation(&caller, id).get();
        require!(
            &token_amount + &old_bought_amount >= launchpad.min_buy_amount,
            ERROR_LOW_AMOUNT
        );
        require!(
            &token_amount + &old_bought_amount <= launchpad.max_buy_amount,
            ERROR_HIGH_AMOUNT
        );
        require!(
            &token_amount + &launchpad.total_sold <= launchpad.amount,
            ERROR_INSUFFICIENT_FUNDS
        );

        self.send().direct_esdt(
            &caller,
            &launchpad.token,
            0,
            &token_amount
        );

        launchpad.total_raised += payment.amount;
        launchpad.total_sold += &token_amount;
        self.launchpads(id).set(launchpad);

        self.user_participation(&caller, id).update(|value| *value += &token_amount);
        self.user_launchpads(&caller).insert(id);
        self.launchpad_users(id).insert(caller);
    }

    #[endpoint(launch)]
    fn launch(&self, id: u64) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(!self.launchpads(id).is_empty(), ERROR_LAUNCHPAD_NOT_FOUND);

        let mut launchpad = self.launchpads(id).get();
        require!(launchpad.end_time < self.blockchain().get_block_timestamp(), ERROR_LAUNCHPAD_NOT_ENDED);
        require!(!launchpad.launched, ERROR_ALREADY_LAUNCHED);

        let mut payments: ManagedVec<EsdtTokenPayment> = ManagedVec::new();
        if launchpad.total_raised > 0 {
            payments.push(EsdtTokenPayment::new(launchpad.payment_token.clone(), 0, launchpad.total_raised.clone()));
        }

        let left_amount = &launchpad.amount - &launchpad.total_sold;
        if left_amount > 0 {
            payments.push(EsdtTokenPayment::new(launchpad.token.clone(), 0, left_amount.clone()));
        }

        if !payments.is_empty() {
            self.send().direct_multi(&launchpad.owner, &payments);
        }

        launchpad.launched = true;
        self.launchpads(id).set(launchpad);
    }

    // helpers
    fn check_whitelisted(&self, address: &ManagedAddress) {
        self.platform_contract_proxy()
            .contract(self.platform_sc().get())
            .check_whitelisted(address)
            .execute_on_dest_context::<()>();
    }

    // proxies
    #[proxy]
    fn platform_contract_proxy(&self) -> tfn_platform::Proxy<Self::Api>;
}
