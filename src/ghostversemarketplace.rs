#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

// Default values for the marketplace logic.
// We sell single NFT objects only and take 3% marketplace fee and royalties to the creator.
const NFT_AMOUNT: u32 = 1; // We sell single NFT objects only.
const MARKETPLACE_CUT: u32 = 300; // We take 3% as fixed marketplace fee.
const ROYALTIES: u32 = 500; // NFT creator royalties, TODO, fetch this from the NFT properties.
const PERCENTAGE_TOTAL: u64 = 10000; // Total percentage for calculations.

// Store the NFT listing objects in the storage.
pub struct NftListing<M: ManagedTypeApi> {
    pub token: TokenIdentifier<M>, // Collection identifier, e.g. name + 6 random symbols, e.g. <GHOSTSET-531aff>-01.
    pub nonce: u64, // nonce, id for NFT in collection, e.g. GHOSTSET-531aff-<01>.
    pub owner: ManagedAddress<M>, // The owner of the NFT who this listing belongs to.
    pub price: BigUint<M>, // Price of the NFT listing.
}

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait Ghostversemarketplace {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
