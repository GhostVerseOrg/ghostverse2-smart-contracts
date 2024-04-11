#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod nft_module;

#[elrond_wasm::contract]
pub trait NftMinter: nft_module::NftModule {
    // Initialize empty storage for NFT.
    #[init]
    fn init(&self) {}
}

mod nft_marketplace_proxy {
    elrond_wasm::imports!();
}
