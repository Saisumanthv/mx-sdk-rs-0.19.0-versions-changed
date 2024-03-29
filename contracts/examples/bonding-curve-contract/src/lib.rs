#![no_std]

dharitri_wasm::imports!();
dharitri_wasm::derive_imports!();

use dharitri_wasm_module_bonding_curve::utils::{events, owner_endpoints, storage, user_endpoints};

#[dharitri_wasm::contract]
pub trait Contract:
    dharitri_wasm_module_bonding_curve::BondingCurveModule
    + storage::StorageModule
    + events::EventsModule
    + user_endpoints::UserEndpointsModule
    + owner_endpoints::OwnerEndpointsModule
{
    #[init]
    fn init(&self) {}
}
