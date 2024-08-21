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
        let creator_royalties_percentage = self.get_nft_info(&nft_token, nft_nonce).royalties;

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

        // Get buyer's address to transfer NFT to.
        let caller = self.blockchain().get_caller();
        // Get marketplace smart contract owner address to transfer service fee to.
        let marketplace_owner = self.blockchain().get_owner_address();
        // Get original owner of the NFT to transfer revenue to.
        let nft_owner = nft_listing.nft_original_owner;

        // Fetch available NFT data on the SC wallet balance.
        let token_info = self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            &nft_token.clone(),
            nft_nonce.clone()
        );

        // Transfer NFT to buyer.
        self.send().direct_esdt(
            &caller,
            &nft_token,
            nft_nonce,
            &BigUint::from(NFT_AMOUNT),
        );

        // Transfer service fee to marketplace SC owner. 
        self.send().direct_egld(
            &marketplace_owner,
            &marketplace_service_fee,
        );

        // Transfer royalties to original NFT creator.
        self.send().direct_egld(
            &token_info.creator,
            &royalties,
        );

        // Transfer revenue to NFT seller.
        self.send().direct_egld(
            &nft_owner,
            &leftovermoney,
        );

        // Remove listing details from the memory after the deal is done.
        self.listing_details().remove(&(nft_token.clone(), nft_nonce.clone()));

        SCResult::Ok(())
    }

    // #[payable("*")] 
    // #[endpoint(update_price)]
    // fn update_price(
    //     &self,
    //     token_id: TokenIdentifier,
    //     nonce: u64,
    //     payment_amount: BigUint,    
    // ) -> SCResult<()> {
    //     let caller = self.blockchain().get_caller();
    //     // Retreive NFT data from SC storage.
    //     let curNft = self.nft_detail().get(&(token_id.clone(), nonce.clone())).unwrap();

    //     require!(caller == curNft.owner, "You are not the owner of this token");
    //     require!(token_id == curNft.token, "Invalid token");
    //     require!(nonce == curNft.nonce, "Invalid nonce");

    //     self.nft_detail().insert((token_id.clone(), nonce.clone()), NftListing{
    //         owner: curNft.owner,
    //         token: curNft.token,
    //         nonce: curNft.nonce,
    //         amount: payment_amount,
    //     });

    //     SCResult::Ok(())
    // }


    // #[payable("*")] 
    // #[endpoint(cancel_listing)]
    // fn cancel_listing(
    //     &self,
    //     token_id: TokenIdentifier,
    //     nonce: u64,
    // ) -> SCResult<()> {
    //     // Check the mapped value exists.
    //     require!(
    //         self.nft_detail().contains_key(&(token_id.clone(), nonce.clone())),
    //         "Invalid NFT token, nonce or NFT was already sold");

    //     let caller = self.blockchain().get_caller();
    //     // Retreive NFT data from SC storage.
    //     let curNft = self.nft_detail().get(&(token_id.clone(), nonce.clone())).unwrap();

    //     require!(caller == curNft.owner, "You are not the owner of this token");
    //     require!(token_id == curNft.token, "Invalid token");
    //     require!(nonce == curNft.nonce, "Invalid nonce");
        
    //     self.nft_detail().remove(&(token_id.clone(), nonce.clone()));

    //     self.send().direct(
    //         &caller,
    //         &token_id,
    //         nonce,
    //         &BigUint::from(NFT_AMOUNT),
    //         &[],
    //     );

    //     SCResult::Ok(())
    // }

    /* ________________ */
    /* Helper functions */

    // Helper function to get NFT info from the chain.
    fn get_nft_info(&self, nft_type: &TokenIdentifier, nft_nonce: u64) -> EsdtTokenData<Self::Api> {
        self.blockchain().get_esdt_token_data(
            &self.blockchain().get_sc_address(),
            nft_type,
            nft_nonce,
        )
    }
}
