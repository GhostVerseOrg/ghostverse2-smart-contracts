#![no_std]


#[allow(unused_imports)]
multiversx_sc::imports!();
multiversx_sc::derive_imports!();

// Default values for the marketplace logic.
// We sell single NFT objects only and take 3% marketplace fee and royalties to the creator.
const nft_amount: u32 = 1; // We sell single NFT objects only.
const marketplace_cut_percentage: u32 = 300; // We take 3% as fixed marketplace fee.
const creator_royalties_percentage: u32 = 500; // NFT creator royalties, TODO, fetch this from the NFT properties.
const percentage_total: u64 = 10000; // Total percentage for calculations.

// Store the NFT listing objects in the storage.
#[derive(TypeAbi, TopEncode, TopDecode, ManagedVecItem, NestedEncode, NestedDecode)]
pub struct NftListing<M: ManagedTypeApi> {
    pub token: TokenIdentifier<M>, // Collection identifier, e.g. name + 6 random symbols, e.g. <GHOSTSET-531aff>-01.
    pub nonce: u64, // nonce, id for NFT in collection, e.g. GHOSTSET-531aff-<01>.
    pub original_owner: ManagedAddress<M>, // The owner of the NFT who this listing belongs to.
    pub amount: BigUint<M>, // Price of the NFT listing.
    pub publish_time: u64,
}

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait Ghostversemarketplace {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    // Map NFT using some custom mapper identifier.
    // Map NFT by unique token+nonce pair for NFT object.   
    #[storage_mapper("listingDetails")]
    fn listing_details(&self) -> MapMapper<(TokenIdentifier, u64), NftListing<Self::Api>>;

    // Get all NFT listings from SC memory.
    // #[allow(clippy::type_complexity)]
    #[view(getFullMarketplaceData)]
    fn get_full_marketplace_data(&self) 
    -> MultiValueManagedVec<NftListing<Self::Api>> {
        let storageDetails = self.listing_details(); // store in a intermediate variable not dropping the results
        let storageIterator = storageDetails.values(); // get iterator from the results retrieved above 
        let mut listingsFound: MultiValueManagedVec<Self::Api, NftListing<Self::Api>> = MultiValueManagedVec::new();

        for nft in storageIterator {
            listingsFound.push(
                NftListing{
                token: nft.token,
                nonce: nft.nonce,
                original_owner: nft.original_owner,
                amount: nft.amount,
                publish_time: self.blockchain().get_block_timestamp(),
                }
            )
        }

        return listingsFound;
    }


    // // owner-only endpoints
    // #[payable("EGLD")]
    // #[endpoint(list_nft)] // endpoint name
    // fn list_nft( // list NFT e.g. TR11-531aff-01 for sale
    //     &self,
    //     token_id: TokenIdentifier, // collection identifier, e.g. name + 6 random symbols, e.g. TR11-531aff(-01)
    //     nonce: u64, // nonce, e.g. id for NFT in collection, e.g. (TR11-531aff)-01
    //     selling_price: BigUint, // e.g. 0.5 EGLD
    // ) -> SCResult<()> {
    //     self.nft_detail().insert((token_id.clone(), nonce.clone()), NftListing{
    //         owner: self.blockchain().get_caller(),
    //         token: token_id,
    //         nonce: nonce,
    //         amount: selling_price,
    //     });

    //     SCResult::Ok(())
    // }

    // // endpoints
    // #[payable("EGLD")]
    // #[allow(clippy::too_many_arguments)]
    // #[endpoint(buy_nft)]
    // fn buy_nft(
    //     &self,
    //     #[payment_amount] payment_amount: BigUint,
    //     nft_token_id: TokenIdentifier,
    //     nft_nonce: u64,
    // ) -> SCResult<()> {

    //     // Check the mapped value exists.
    //     require!(
    //         self.nft_detail().contains_key(&(nft_token_id.clone(), nft_nonce.clone())),
    //         "Invalid NFT token, nonce or NFT was already sold");

    //     // Retreive NFT data from SC storage.
    //     let curNft = self.nft_detail().get(&(nft_token_id.clone(), nft_nonce.clone())).unwrap();

    //     require!(nft_token_id == curNft.token, "Invalid token used as payment");
    //     require!(nft_nonce == curNft.nonce, "Invalid nonce for payment token");
    //     require!(payment_amount == curNft.amount, "Invalid amount as payment");

    //     //___________________
    //     // Calculacte marketplace 3% marketplace_service_fee.
    //     let marketplace_service_fee = curNft.amount.clone() * MARKETPLACE_CUT / PERCENTAGE_TOTAL; 
    //     // Calculacte NFT creator royalties.
    //     let royalties = curNft.amount.clone() * ROYALTIES / PERCENTAGE_TOTAL; 
    //     // Calculate seller's revenue.
    //     let leftovermoney = curNft.amount.clone() - marketplace_service_fee.clone() - royalties.clone();

    //     //___________________
    //     // buyer's address
    //     let caller = self.blockchain().get_caller();
    //     // marketplace smart contract owner
    //     let marketplace_owner = self.blockchain().get_owner_address();
    //     // NFT owner (seller)
    //     let nftOwner = curNft.owner;

    //     //___________________
    //     // fetch info from NFT that is on balance of smart contract now.
    //     let scAddress = self.blockchain().get_sc_address();
    //     let tokenIfo = self.blockchain().get_esdt_token_data(&scAddress, &nft_token_id.clone(), nft_nonce.clone());

    //     //___________________
    //     // send NFT to buyer
    //     self.send().direct(
    //         &caller,
    //         &nft_token_id,
    //         nft_nonce,
    //         &BigUint::from(NFT_AMOUNT),
    //         &[],
    //     );

    //     // send marketplace fee to marketplace owner
    //     self.send().direct(
    //         &marketplace_owner,
    //         &TokenIdentifier::egld(),
    //         0,
    //         &marketplace_service_fee,
    //         &[],
    //     );

    //     // send royalties to NFT creator
    //         self.send().direct(
    //         &tokenIfo.creator,
    //         &TokenIdentifier::egld(),
    //         0,
    //         &royalties,
    //         &[],
    //     );

    //         // send revenue to NFT seller
    //         self.send().direct(
    //         &nftOwner,
    //         &TokenIdentifier::egld(),
    //         0,
    //         &leftovermoney,
    //         &[],
    //     );

    //     // Clear NFT storage data after it's sold.
    //     self.nft_detail().remove(&(nft_token_id.clone(), nft_nonce.clone()));

    //     SCResult::Ok(())
    // }

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

    // Map NFT using some custom mapper identifier.
    // Map NFT by unique token+nonce pair for NFT object.   
    // #[storage_mapper("NftListing")]
    // fn nft_detail(&self) -> MapMapper<(TokenIdentifier, u64), NftListing<Self::Api>>;

    // #[allow(clippy::type_complexity)]
    // #[view(get_listing)]
    // fn get_listing(
    //     &self,
    //     token_id: TokenIdentifier,
    //     nonce: u64,
    // ) -> OptionalValue<MultiValue4<ManagedAddress, TokenIdentifier, u64, BigUint>> {
    //     if !self.nft_detail().contains_key(&(token_id.clone(), nonce.clone())) {
    //         // NFT was already sold
    //         OptionalValue::None
    //     } else {
    //         // Retreive NFT data from SC storage.
    //         let curNft = self.nft_detail().get(&(token_id.clone(), nonce.clone())).unwrap();
    //         OptionalValue::Some((
    //             curNft.owner,
    //             curNft.token,
    //             curNft.nonce,
    //             curNft.amount,
    //             ).into())
    //     }
    // }
}
