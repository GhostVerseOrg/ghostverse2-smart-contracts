#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

// Default values for the marketplace logic.
// We sell single NFT objects only and take 3% marketplace fee and royalties to the creator.
const NFT_AMOUNT: u32 = 1; // We sell single NFT objects only.
const MARKETPLACE_CUT: u32 = 300; // 3%.
pub const PERCENTAGE_TOTAL: u64 = 10_000; // 100%.

// Store the NFT listing objects in the storage.
#[derive(TypeAbi, TopEncode, TopDecode, ManagedVecItem, NestedEncode, NestedDecode)]
pub struct NftListing<M: ManagedTypeApi> {
    pub nft_token: TokenIdentifier<M>, // Collection identifier, e.g. name + 6 random symbols, e.g. <GHOSTSET-531aff>-01.
    pub nft_nonce: u64, // nonce, id for NFT in collection, e.g. GHOSTSET-531aff-<01>.
    pub nft_original_owner: ManagedAddress<M>, // The owner of the NFT who this listing belongs to.
    pub listing_amount: BigUint<M>, // Price of the NFT listing.
    pub listing_publish_time: u64, // Store the publish data of the listing for sorting.
}

#[multiversx_sc::contract]
pub trait Ghostversemarketplace{
    // We must have init function in order for SC to build.
    #[init]
    fn init(&self) {}

    // We reserve the Upgrade function for later use.
    #[upgrade]
    fn upgrade(&self) {}

    /* ________________________________ */
    /* Data mapper and data fetch logic */

    // We must map our data structs explicitly, we map NFTs by unique token+nonce pair for internal NFT listing data.
    // NB! Mapper returns only the correctly added NFT-s to the marketplace, any NFTs directly spammed to the wallet are simply ignored.
    // TODO: implement a method to remove NFTs from the wallet that are not listed/mapped in the marketplace.
    #[storage_mapper("listingDetails")]
    fn listing_details(&self) -> MapMapper<(TokenIdentifier, u64), NftListing<Self::Api>>;

    // Get all NFT listings from SC memory, iterate over the storage and return all the data.
    #[view(getFullMarketplaceData)]
    fn get_full_marketplace_data(&self) 
    -> MultiValueManagedVec<NftListing<Self::Api>> {
        let storage_details = self.listing_details(); // Store results from SC storage in variable to extract iterator from it.
        let storage_iterator = storage_details.values(); // Extract the iterator to loop over the storage values.
        let mut listings: MultiValueManagedVec<Self::Api, NftListing<Self::Api>> = MultiValueManagedVec::new();

        // Iterate through all available data and return everything.
        for listing in storage_iterator {
            listings.push(
                NftListing{
                nft_token: listing.nft_token,
                nft_nonce: listing.nft_nonce,
                nft_original_owner: listing.nft_original_owner,
                listing_amount: listing.listing_amount,
                listing_publish_time: listing.listing_publish_time,
                }
            )
        }

        return listings;
    }

    // Get requested NFT listing data using token+nonce tuple as key.
    #[view(get_listing)]
    fn get_listing(
        &self,
        nft_token: TokenIdentifier,
        nft_nonce: u64,
    ) -> OptionalValue<MultiValue5<ManagedAddress, TokenIdentifier, u64, BigUint, u64>> {
        require!(
            self.listing_details().contains_key(&(nft_token.clone(), nft_nonce.clone())),
            "Invalid NFT token or nonce or it was already sold"
        );
        
        let listing = self.listing_details().get(&(nft_token.clone(), nft_nonce.clone())).unwrap();
        OptionalValue::Some((
            listing.nft_original_owner,
            listing.nft_token,
            listing.nft_nonce,
            listing.listing_amount,
            listing.listing_publish_time,
        ).into())
    }

    /* _________________________ */
    /* Marketplace functionality */

    // List NFT for sale in the marketplace.
    #[payable("*")]
    #[endpoint(list_nft)]
    fn list_nft(
        &self,
        nft_token: TokenIdentifier,
        nft_nonce: u64,
        listing_amount: BigUint,
    ) -> SCResult<()> {
        // NB! Security double-fix, when we are in the middle of a transaction we look for the specified NFT in the current transaction's SC wallet state, we can do this as we are in the middle of a transaction after the transfer happened.
        // When we see NFT listed on the current SC wallet (we're in the middle of a transaction), we can be sure that the NFT is owned by the SC wallet now and we need to proceed with the results.
        let nft_info = self.get_nft_info(&nft_token, nft_nonce);

        // Ref to doc: https://docs.multiversx.com/developers/developer-reference/sc-api-functions/#get_esdt_token_data
        // """token_type is an enum, which can have one of the following values: pub enum EsdtTokenType { Fungible, NonFungible, SemiFungible, Meta, Invalid}>>""".
        // """You will only receive basic distinctions for the token type, i.e. only Fungible and NonFungible (The smart contract has no way of telling the difference between non-fungible, semi-fungible and meta tokens)""".
        // """Amount is the current owned balance of the account.""".
        // We double verify that we have only 1 NFT token and it's non-fungible.
        require!(
            nft_info.amount == 1 && nft_info.token_type == EsdtTokenType::NonFungible,
            "You can only sell single NFT objects!"
        );

        let creator_royalties_percentage = nft_info.royalties;

        require!(
            BigUint::from(MARKETPLACE_CUT) + &creator_royalties_percentage < PERCENTAGE_TOTAL,
            "Marketplace cut plus royalties exceeds 100%"
        );

        self.listing_details().insert((nft_token.clone(), nft_nonce.clone()), NftListing{
            nft_token: nft_token,
            nft_nonce: nft_nonce,
            nft_original_owner: self.blockchain().get_caller(),
            listing_amount: listing_amount,
            listing_publish_time: self.blockchain().get_block_timestamp(),
        });

        SCResult::Ok(())
    }

    #[payable("EGLD")]
    #[endpoint(buy_nft)]
    fn buy_nft(
        &self,
        #[payment_amount] payment_amount: BigUint,
        nft_token: TokenIdentifier,
        nft_nonce: u64,
    ) -> SCResult<()> {

        // Check if the mapped value exists.
        require!(
            self.listing_details().contains_key(&(nft_token.clone(), nft_nonce.clone())),
            "Invalid NFT token or nonce or it was already sold"
        );

        // Retreive NFT data from SC storage.
        let nft_listing = self.listing_details().get(&(nft_token.clone(), nft_nonce.clone())).unwrap();

        require!(nft_token == nft_listing.nft_token, "Invalid token");
        require!(nft_nonce == nft_listing.nft_nonce, "Invalid nonce");
        require!(payment_amount == nft_listing.listing_amount, "Invalid amount");

        // Get current NFT royalties percentage.
        let creator_royalties_percentage = self.get_nft_info(&nft_token, nft_nonce).royalties;

        // Calculate marketplace 3% service fee.
        let marketplace_service_fee = nft_listing.listing_amount.clone() * MARKETPLACE_CUT / PERCENTAGE_TOTAL; 
        // Calculate NFT creator royalties.
        let royalties = BigUint::from(nft_listing.listing_amount.clone()) * creator_royalties_percentage / BigUint::from(PERCENTAGE_TOTAL); 
        // Calculate seller's revenue.
        let leftovermoney = nft_listing.listing_amount.clone() - marketplace_service_fee.clone() - royalties.clone();

        // Fetch available NFT data on the SC wallet balance.
        let token_info = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            &nft_token.clone(),
            nft_nonce.clone()
        );

        // Transfer NFT to buyer.
        self.send().direct_esdt(
            &self.blockchain().get_caller(),
            &nft_token,
            nft_nonce,
            &BigUint::from(NFT_AMOUNT),
        );

        // Transfer service fee to marketplace SC owner. 
        self.send().direct_egld(
            &self.blockchain().get_owner_address(),
            &marketplace_service_fee,
        );

        // Transfer royalties to original NFT creator.
        self.send().direct_egld(
            &token_info.creator,
            &royalties,
        );


        // Get original owner of the NFT to transfer revenue to, object type constraint here.
        let nft_owner = nft_listing.nft_original_owner;

        // Transfer revenue to NFT seller.
        self.send().direct_egld(
            &nft_owner,
            &leftovermoney,
        );

        // Remove listing details from the memory after the deal is done.
        self.listing_details().remove(&(nft_token.clone(), nft_nonce.clone()));

        SCResult::Ok(())
    }

    #[payable("*")] 
    #[endpoint(update_price)]
    fn update_price(
        &self,
        nft_token: TokenIdentifier,
        nft_nonce: u64,
        listing_amount: BigUint,    
    ) -> SCResult<()> {
        // Check if the mapped value exists.
        require!(
            self.listing_details().contains_key(&(nft_token.clone(), nft_nonce.clone())),
            "Invalid NFT token or nonce or it was already sold"
        );

        // Retreive NFT data from SC storage.
        let listing = self.listing_details().get(&(nft_token.clone(), nft_nonce.clone())).unwrap();

        let caller = self.blockchain().get_caller();

        // Verify the data.
        require!(caller == listing.nft_original_owner, "You are not the owner of this token");
        require!(nft_token == listing.nft_token, "Error in SC listing data");
        require!(nft_nonce == listing.nft_nonce, "Error in SC listing data");

        self.listing_details().insert((nft_token.clone(), nft_nonce.clone()), NftListing{
            nft_token: listing.nft_token,
            nft_nonce: listing.nft_nonce,
            nft_original_owner: listing.nft_original_owner,
            listing_amount: listing_amount,
            listing_publish_time: listing.listing_publish_time,
        });

        SCResult::Ok(())
    }


    #[payable("*")] 
    #[endpoint(cancel_listing)]
    fn cancel_listing(
        &self,
        nft_token: TokenIdentifier,
        nft_nonce: u64,
    ) -> SCResult<()> {

        // Check if the mapped value exists.
        require!(
            self.listing_details().contains_key(&(nft_token.clone(), nft_nonce.clone())),
            "Invalid NFT token or nonce or it was already sold"
        );

        let caller = self.blockchain().get_caller();
        // Retreive NFT data from SC storage.
        let listing = self.listing_details().get(&(nft_token.clone(), nft_nonce.clone())).unwrap();

        require!(caller == listing.nft_original_owner, "You are not the owner of this token");
        require!(nft_token == listing.nft_token, "Invalid token");
        require!(nft_nonce == listing.nft_nonce, "Invalid nonce");
        
        self.listing_details().remove(&(nft_token.clone(), nft_nonce.clone()));

        self.send().direct_esdt(
            &caller,
            &nft_token,
            nft_nonce,
            &BigUint::from(NFT_AMOUNT),
        );

        SCResult::Ok(())
    }

    /* ________________ */
    /* Helper functions */

    // Helper function to get NFT info from the chain.
    // NB! it's hardcoded to look into NFT-s available at the current SC wallet, which makes logic invulnerable to attacks.
    fn get_nft_info(&self, nft_type: &TokenIdentifier, nft_nonce: u64) -> EsdtTokenData<Self::Api> {
        self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            nft_type,
            nft_nonce,
        )
    }
}
