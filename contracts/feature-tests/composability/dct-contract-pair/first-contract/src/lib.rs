#![no_std]

dharitri_wasm::imports!();

const DCT_TRANSFER_STRING: &[u8] = b"DCTTransfer";
const SECOND_CONTRACT_ACCEPT_DCT_PAYMENT: &[u8] = b"acceptDctPayment";
const SECOND_CONTRACT_REJECT_DCT_PAYMENT: &[u8] = b"rejectDctPayment";

#[dharitri_wasm::contract]
pub trait FirstContract {
    #[init]
    fn init(&self, dct_token_name: TokenIdentifier, second_contract_address: ManagedAddress) {
        self.set_contract_dct_token_name(&dct_token_name);
        self.set_second_contract_address(&second_contract_address);
    }

    #[payable("*")]
    #[endpoint(transferToSecondContractFull)]
    fn transfer_to_second_contract_full(
        &self,
        #[payment] dct_value: BigUint,
        #[payment_token] actual_token_name: TokenIdentifier,
    ) -> SCResult<()> {
        let expected_token_name = self.get_contract_dct_token_name();

        require!(dct_value > 0, "no dct transfered!");
        require!(actual_token_name == expected_token_name, "Wrong dct token");

        self.call_dct_second_contract(
            &expected_token_name,
            &dct_value,
            &self.get_second_contract_address(),
            &self
                .types()
                .managed_buffer_from(SECOND_CONTRACT_ACCEPT_DCT_PAYMENT),
            &ManagedVec::new_empty(self.type_manager()),
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint(transferToSecondContractHalf)]
    fn transfer_to_second_contract_half(
        &self,
        #[payment] dct_value: BigUint,
        #[payment_token] actual_token_name: TokenIdentifier,
    ) -> SCResult<()> {
        let expected_token_name = self.get_contract_dct_token_name();

        require!(dct_value > 0, "no dct transfered!");
        require!(actual_token_name == expected_token_name, "Wrong dct token");

        self.call_dct_second_contract(
            &expected_token_name,
            &(dct_value / 2u32),
            &self.get_second_contract_address(),
            &self
                .types()
                .managed_buffer_from(SECOND_CONTRACT_ACCEPT_DCT_PAYMENT),
            &ManagedVec::new_empty(self.type_manager()),
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint(transferToSecondContractRejected)]
    fn transfer_to_second_contract_rejected(
        &self,
        #[payment] dct_value: BigUint,
        #[payment_token] actual_token_name: TokenIdentifier,
    ) -> SCResult<()> {
        let expected_token_name = self.get_contract_dct_token_name();

        require!(dct_value > 0, "no dct transfered!");
        require!(actual_token_name == expected_token_name, "Wrong dct token");

        self.call_dct_second_contract(
            &expected_token_name,
            &dct_value,
            &self.get_second_contract_address(),
            &self
                .types()
                .managed_buffer_from(SECOND_CONTRACT_REJECT_DCT_PAYMENT),
            &ManagedVec::new_empty(self.type_manager()),
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint(transferToSecondContractRejectedWithTransferAndExecute)]
    fn transfer_to_second_contract_rejected_with_transfer_and_execute(
        &self,
        #[payment] dct_value: BigUint,
        #[payment_token] actual_token_name: TokenIdentifier,
    ) -> SCResult<()> {
        let second_contract_address = self.get_second_contract_address();
        let expected_token_name = self.get_contract_dct_token_name();

        require!(dct_value > 0, "no dct transfered!");
        require!(actual_token_name == expected_token_name, "Wrong dct token");

        let _ = self.raw_vm_api().direct_dct_execute(
            &second_contract_address,
            &expected_token_name,
            &dct_value,
            self.blockchain().get_gas_left(),
            &self
                .types()
                .managed_buffer_from(SECOND_CONTRACT_REJECT_DCT_PAYMENT),
            &ManagedArgBuffer::new_empty(self.type_manager()),
        );

        Ok(())
    }

    #[payable("*")]
    #[endpoint(transferToSecondContractFullWithTransferAndExecute)]
    fn transfer_to_second_contract_full_with_transfer_and_execute(
        &self,
        #[payment] dct_value: BigUint,
        #[payment_token] actual_token_name: TokenIdentifier,
    ) -> SCResult<()> {
        let second_contract_address = self.get_second_contract_address();
        let expected_token_name = self.get_contract_dct_token_name();

        require!(dct_value > 0, "no dct transfered!");
        require!(actual_token_name == expected_token_name, "Wrong dct token");

        let _ = self.raw_vm_api().direct_dct_execute(
            &second_contract_address,
            &expected_token_name,
            &dct_value,
            self.blockchain().get_gas_left(),
            &self
                .types()
                .managed_buffer_from(SECOND_CONTRACT_ACCEPT_DCT_PAYMENT),
            &ManagedArgBuffer::new_empty(self.type_manager()),
        );

        Ok(())
    }

    fn call_dct_second_contract(
        &self,
        dct_token_name: &TokenIdentifier,
        amount: &BigUint,
        to: &ManagedAddress,
        func_name: &ManagedBuffer,
        args: &ManagedVec<Self::Api, ManagedBuffer>,
    ) {
        let mut arg_buffer = ManagedArgBuffer::new_empty(self.type_manager());
        arg_buffer.push_arg(dct_token_name);
        arg_buffer.push_arg(amount);
        arg_buffer.push_arg(func_name);
        for arg in args.into_iter() {
            arg_buffer.push_arg_raw(arg);
        }

        self.raw_vm_api().async_call_raw(
            to,
            &self.types().big_uint_zero(),
            &self.types().managed_buffer_from(DCT_TRANSFER_STRING),
            &arg_buffer,
        );
    }

    // storage

    #[storage_set("dctTokenName")]
    fn set_contract_dct_token_name(&self, dct_token_name: &TokenIdentifier);

    #[view(getDctTokenName)]
    #[storage_get("dctTokenName")]
    fn get_contract_dct_token_name(&self) -> TokenIdentifier;

    #[storage_set("secondContractAddress")]
    fn set_second_contract_address(&self, address: &ManagedAddress);

    #[view(getSecondContractAddress)]
    #[storage_get("secondContractAddress")]
    fn get_second_contract_address(&self) -> ManagedAddress;
}
