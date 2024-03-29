dharitri_wasm::imports!();

use super::storage;

const PERCENTAGE_TOTAL: u64 = 10_000; // 100%

#[dharitri_wasm::module]
pub trait ForwarderDctModule: storage::ForwarderStorageModule {
    #[view(getFungibleDctBalance)]
    fn get_fungible_dct_balance(&self, token_identifier: &TokenIdentifier) -> BigUint {
        self.blockchain()
            .get_dct_balance(&self.blockchain().get_sc_address(), token_identifier, 0)
    }

    #[endpoint]
    fn send_dct(
        &self,
        to: &ManagedAddress,
        token_id: TokenIdentifier,
        amount: &BigUint,
        #[var_args] opt_data: OptionalArg<ManagedBuffer>,
    ) {
        let data = match opt_data {
            OptionalArg::Some(data) => data,
            OptionalArg::None => self.types().managed_buffer_empty(),
        };
        self.send().direct(to, &token_id, 0, amount, data);
    }

    #[payable("*")]
    #[endpoint]
    fn send_dct_with_fees(
        &self,
        #[payment_token] token_id: TokenIdentifier,
        #[payment_amount] payment: BigUint,
        to: ManagedAddress,
        percentage_fees: BigUint,
    ) {
        let fees = &payment * &percentage_fees / PERCENTAGE_TOTAL;
        let amount_to_send = payment - fees;

        self.send().direct(&to, &token_id, 0, &amount_to_send, &[]);
    }

    #[endpoint]
    fn send_dct_twice(
        &self,
        to: &ManagedAddress,
        token_id: TokenIdentifier,
        amount_first_time: &BigUint,
        amount_second_time: &BigUint,
        #[var_args] opt_data: OptionalArg<ManagedBuffer>,
    ) {
        let data = match opt_data {
            OptionalArg::Some(data) => data,
            OptionalArg::None => self.types().managed_buffer_empty(),
        };
        self.send()
            .direct(to, &token_id, 0, amount_first_time, data.clone());
        self.send()
            .direct(to, &token_id, 0, amount_second_time, data);
    }

    #[endpoint]
    fn send_dct_direct_multi_transfer(
        &self,
        to: ManagedAddress,
        #[var_args] token_payments: VarArgs<MultiArg3<TokenIdentifier, u64, BigUint>>,
    ) {
        let mut all_token_payments = ManagedVec::new_empty(self.type_manager());

        for multi_arg in token_payments.into_vec() {
            let (token_name, token_nonce, amount) = multi_arg.into_tuple();
            let payment = DctTokenPayment {
                token_name,
                token_nonce,
                amount,
                token_type: DctTokenType::Invalid, // not used
            };

            all_token_payments.push(payment);
        }

        let _ = self.raw_vm_api().direct_multi_dct_transfer_execute(
            &to,
            &all_token_payments,
            self.blockchain().get_gas_left(),
            &self.types().managed_buffer_empty(),
            &ManagedArgBuffer::new_empty(self.type_manager()),
        );
    }

    #[payable("MOAX")]
    #[endpoint]
    fn issue_fungible_token(
        &self,
        #[payment] issue_cost: BigUint,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        initial_supply: BigUint,
    ) -> AsyncCall {
        let caller = self.blockchain().get_caller();

        self.send()
            .dct_system_sc_proxy()
            .issue_fungible(
                issue_cost,
                &token_display_name,
                &token_ticker,
                &initial_supply,
                FungibleTokenProperties {
                    num_decimals: 0,
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_mint: true,
                    can_burn: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().dct_issue_callback(&caller))
    }

    #[callback]
    fn dct_issue_callback(
        &self,
        caller: &ManagedAddress,
        #[payment_token] token_identifier: TokenIdentifier,
        #[payment] returned_tokens: BigUint,
        #[call_result] result: AsyncCallResult<()>,
    ) {
        // callback is called with DCTTransfer of the newly issued token, with the amount requested,
        // so we can get the token identifier and amount from the call data
        match result {
            AsyncCallResult::Ok(()) => {
                self.last_issued_token().set(&token_identifier);
                self.last_error_message().clear();
            },
            AsyncCallResult::Err(message) => {
                // return issue cost to the caller
                if token_identifier.is_moax() && returned_tokens > 0 {
                    self.send().direct_moax(caller, &returned_tokens, &[]);
                }

                self.last_error_message().set(&message.err_msg);
            },
        }
    }

    #[endpoint]
    fn local_mint(&self, token_identifier: TokenIdentifier, amount: BigUint) {
        self.send().dct_local_mint(&token_identifier, 0, &amount);
    }

    #[endpoint]
    fn local_burn(&self, token_identifier: TokenIdentifier, amount: BigUint) {
        self.send().dct_local_burn(&token_identifier, 0, &amount);
    }
}
