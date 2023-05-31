/*
ABOUT THIS CONTRACT...
This contract lets users create, administrate and use refferal/affiliate 
programs in the Geode ecosystem.
*/ 

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod geode_referrals {

    use ink::prelude::vec::Vec;
    // use ink::prelude::string::String;
    use ink::storage::Mapping;
    use ink::env::hash::{Sha2x256, HashOutput};
    use openbrush::{
        contracts::{
            reentrancy_guard::*,
            traits::errors::ReentrancyGuardError,
        },
        traits::{
            Storage,
            ZERO_ADDRESS
        },
    };

    // PRELIMINARY STORAGE STRUCTURES >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct AccountVector {
        accountvector: Vec<AccountId>,
    }

    impl Default for AccountVector {
        fn default() -> AccountVector {
            AccountVector {
              accountvector: <Vec<AccountId>>::default(),
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct HashVector {
        hashvector: Vec<Hash>,
    }

    impl Default for HashVector {
        fn default() -> HashVector {
            HashVector {
              hashvector: <Vec<Hash>>::default(),
            }
        }
    }


    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct Claim { 
        program_id: Hash,
        claim_id: Hash,
        parent: AccountId, 
        parent_ip: Vec<u8>,
        // caller and IP Address
        child: AccountId,
        // who they are claiming to have referred
        child_ip: Vec<u8>,
        timestamp: u64,
        grandparent: AccountId,
        branch: (AccountId, AccountId, AccountId),
        // parent, claimant, child
        pay_in: Balance,
        // coin the parent has paid in for the child account to retrieve
        endorsed_by: AccountId,
        // the child account will endorse the claim later
        payout_id: Hash,
        status: u8,
        // 0 = claim made, endorsement needed / 1 = waiting, 2 = approved, 3 = rejected
    }

    impl Default for Claim {
        fn default() -> Claim {
            Claim {
                program_id: Hash::default(),
                claim_id: Hash::default(),
                parent: ZERO_ADDRESS.into(), 
                parent_ip: <Vec<u8>>::default(),
                child: ZERO_ADDRESS.into(),
                child_ip: <Vec<u8>>::default(),
                timestamp: u64::default(),
                grandparent: ZERO_ADDRESS.into(),
                branch: (ZERO_ADDRESS.into(), ZERO_ADDRESS.into(), ZERO_ADDRESS.into()),
                pay_in: Balance::default(),
                endorsed_by: ZERO_ADDRESS.into(),
                payout_id: Hash::default(),
                status: u8::default(),
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct RewardPayout { 
        payout_id: Hash,
        program_id: Hash,
        claim_id: Hash,
        child_account: AccountId,
        child_payout: Balance,
        parent_account: AccountId,
        parent_payout: Balance,
        grandparent_account: AccountId,
        grandparent_payout: Balance,
        timestamp: u64,
        total_payout: Balance
    }

    impl Default for RewardPayout {
        fn default() -> RewardPayout {
            RewardPayout {
                payout_id: Hash::default(),
                program_id: Hash::default(),
                claim_id: Hash::default(),
                child_account: ZERO_ADDRESS.into(),
                child_payout: Balance::default(),
                parent_account: ZERO_ADDRESS.into(),
                parent_payout: Balance::default(),
                grandparent_account: ZERO_ADDRESS.into(),
                grandparent_payout: Balance::default(),
                timestamp: u64::default(),
                total_payout: Balance::default(),
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct Branch { 
        branch_id: Hash,
        program_id: Hash,
        branch: (AccountId, AccountId, AccountId)
    }

    impl Default for Branch {
        fn default() -> Branch {
            Branch {
                branch_id: Hash::default(),
                program_id: Hash::default(),
                branch: (ZERO_ADDRESS.into(), ZERO_ADDRESS.into(), ZERO_ADDRESS.into())
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct ProgramDetails { 
        program_id: Hash,
        owner: AccountId,
        title: Vec<u8>,
        description: Vec<u8>,
        more_info_link: Vec<u8>,
        photo: Vec<u8>,
        first_level_reward: Balance,
        second_level_reward: Balance,
        maximum_rewards: u128,
        rewards_given: u128,
        owner_approval_required: bool,
        pay_in_minimum: Balance,
        program_balance: Balance,
        claims_endorsed_approved: Vec<Hash>,
        claims_endorsed_rejected: Vec<Hash>,
        claims_endorsed_waiting: Vec<Hash>,
        claims_made: Vec<Hash>,
        branches: Vec<Hash>,
        payouts: Vec<Hash>,
        active: bool,
    }

    impl Default for ProgramDetails {
        fn default() -> ProgramDetails {
            ProgramDetails {
                program_id: Hash::default(),
                owner: ZERO_ADDRESS.into(),
                title: <Vec<u8>>::default(),
                description: <Vec<u8>>::default(),
                more_info_link: <Vec<u8>>::default(),
                photo: <Vec<u8>>::default(),
                first_level_reward: Balance::default(),
                second_level_reward: Balance::default(),
                maximum_rewards: u128::default(),
                rewards_given: u128::default(),
                owner_approval_required: bool::default(),
                pay_in_minimum: Balance::default(),
                program_balance: Balance::default(),
                claims_endorsed_approved: <Vec<Hash>>::default(),
                claims_endorsed_rejected: <Vec<Hash>>::default(),
                claims_endorsed_waiting: <Vec<Hash>>::default(),
                claims_made: <Vec<Hash>>::default(),
                branches: <Vec<Hash>>::default(),
                payouts: <Vec<Hash>>::default(),
                active: bool::default(),
            }
        }
    }

   
    // STORAGE STRUCTURES FOR PRIMARY GET MESSAGES >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct ProgramPublicDetails { 
        program_id: Hash,
        owner: AccountId,
        title: Vec<u8>,
        description: Vec<u8>,
        more_info_link: Vec<u8>,
        photo: Vec<u8>,
        first_level_reward: Balance,
        second_level_reward: Balance,
        maximum_rewards: u128,
        rewards_given: u128,
        owner_approval_required: bool,
        pay_in_minimum: Balance
    }

    impl Default for ProgramPublicDetails {
        fn default() -> ProgramPublicDetails {
            ProgramPublicDetails {
                program_id: Hash::default(),
                owner: ZERO_ADDRESS.into(),
                title: <Vec<u8>>::default(),
                description: <Vec<u8>>::default(),
                more_info_link: <Vec<u8>>::default(),
                photo: <Vec<u8>>::default(),
                first_level_reward: Balance::default(),
                second_level_reward: Balance::default(),
                maximum_rewards: u128::default(),
                rewards_given: u128::default(),
                owner_approval_required: bool::default(),
                pay_in_minimum: Balance::default()
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct BrowseAllPrograms { 
        programs: Vec<ProgramPublicDetails>
    }

    impl Default for BrowseAllPrograms {
        fn default() -> BrowseAllPrograms {
            BrowseAllPrograms {
                programs: <Vec<ProgramPublicDetails>>::default(),
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct ViewProgramDetails { 
        program_id: Hash,
        owner: AccountId,
        title: Vec<u8>,
        description: Vec<u8>,
        more_info_link: Vec<u8>,
        photo: Vec<u8>,
        first_level_reward: Balance,
        second_level_reward: Balance,
        maximum_rewards: u128,
        rewards_given: u128,
        owner_approval_required: bool,
        pay_in_minimum: Balance,
        program_balance: Balance,
        claims_endorsed_approved: Vec<Claim>,
        claims_endorsed_rejected: Vec<Claim>,
        claims_endorsed_waiting: Vec<Claim>,
        claims_made: Vec<Claim>,
        branches: Vec<Branch>,
        payouts: Vec<RewardPayout>,
        active: bool,
    }

    impl Default for ViewProgramDetails {
        fn default() -> ViewProgramDetails {
            ViewProgramDetails {
                program_id: Hash::default(),
                owner: ZERO_ADDRESS.into(),
                title: <Vec<u8>>::default(),
                description: <Vec<u8>>::default(),
                more_info_link: <Vec<u8>>::default(),
                photo: <Vec<u8>>::default(),
                first_level_reward: Balance::default(),
                second_level_reward: Balance::default(),
                maximum_rewards: u128::default(),
                rewards_given: u128::default(),
                owner_approval_required: bool::default(),
                pay_in_minimum: Balance::default(),
                program_balance: Balance::default(),
                claims_endorsed_approved: <Vec<Claim>>::default(),
                claims_endorsed_rejected: <Vec<Claim>>::default(),
                claims_endorsed_waiting: <Vec<Claim>>::default(),
                claims_made: <Vec<Claim>>::default(),
                branches: <Vec<Branch>>::default(),
                payouts: <Vec<RewardPayout>>::default(),
                active: bool::default(),
            }
        }
    }
    
    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct ViewMyPrograms {
        programs: Vec<ViewProgramDetails>
    }

    impl Default for ViewMyPrograms {
        fn default() -> ViewMyPrograms {
            ViewMyPrograms {
                programs: <Vec<ViewProgramDetails>>::default()
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct UserDataByProgram {
        program_id: Hash,
        title: Vec<u8>,
        description: Vec<u8>,
        more_info_link: Vec<u8>,
        photo: Vec<u8>,
        first_level_reward: Balance,
        second_level_reward: Balance,
        maximum_rewards: u128,
        rewards_given: u128,
        owner_approval_required: bool,
        pay_in_minimum: Balance, 
        claims: Vec<Claim>,
        payouts: Vec<RewardPayout>,
        branches: Vec<Branch>
    }

    impl Default for UserDataByProgram {
        fn default() -> UserDataByProgram {
            UserDataByProgram {
                program_id: Hash::default(),
                title: <Vec<u8>>::default(),
                description: <Vec<u8>>::default(),
                more_info_link: <Vec<u8>>::default(),
                photo: <Vec<u8>>::default(),
                first_level_reward: Balance::default(),
                second_level_reward: Balance::default(),
                maximum_rewards: u128::default(),
                rewards_given: u128::default(),
                owner_approval_required: bool::default(),
                pay_in_minimum: Balance::default(), 
                claims: <Vec<Claim>>::default(),
                payouts: <Vec<RewardPayout>>::default(),
                branches: <Vec<Branch>>::default()
            }
        }
    }

    #[derive(Clone, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std",
        derive(ink::storage::traits::StorageLayout, 
            scale_info::TypeInfo, Debug, PartialEq, Eq
        )
    )]
    pub struct ViewMyActivity {
        activity: Vec<UserDataByProgram>
    }

    impl Default for ViewMyActivity {
        fn default() -> ViewMyActivity {
            ViewMyActivity {
                activity: <Vec<UserDataByProgram>>::default()
            }
        }
    }

    
    // EVENT DEFINITIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> 
    
    #[ink(event)]
    // writes a new program to the blockchain 
    pub struct NewProgram {
        #[ink(topic)]
        program_id: Hash,
        owner: AccountId,
        #[ink(topic)]
        title: Vec<u8>,
        #[ink(topic)]
        first_level_reward: Balance,
        second_level_reward: Balance,
        maximum_rewards: u128,
        owner_approval_required: bool,
        pay_in_minimum: Balance
    }

    #[ink(event)]
    // writes a new reward payout to the blockchain 
    pub struct RewardPayoutMade {
        payout_id: Hash,
        program_id: Hash,
        claim_id: Hash,
        #[ink(topic)]
        child_account: AccountId,
        child_payout: Balance,
        #[ink(topic)]
        parent_account: AccountId,
        parent_payout: Balance,
        #[ink(topic)]
        grandparent_account: AccountId,
        grandparent_payout: Balance,
        timestamp: u64,
        total_payout: Balance,
    }


    // ERROR DEFINITIONS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        // a generic error
        GenericError,
        // Reentrancy Guard error
        ReentrancyError(ReentrancyGuardError),
        // insufficient pay in
        InsufficientPayment,
        // payout failed
        PayoutFailed,
    }

    impl From<ReentrancyGuardError> for Error {
        fn from(error:ReentrancyGuardError) -> Self {
            Error::ReentrancyError(error)
        }
    }


    // ACTUAL CONTRACT STORAGE STRUCT >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct ContractStorage {
        #[storage_field]
        guard: reentrancy_guard::Data,
        account_claims: Mapping<AccountId, HashVector>,
        claim_details: Mapping<Hash, Claim>,
        account_branches: Mapping<AccountId, HashVector>,
        branch_details: Mapping<Hash, Branch>,
        account_payouts: Mapping<AccountId, HashVector>,
        payout_details: Mapping<Hash, RewardPayout>,
        account_owned_programs: Mapping<AccountId, HashVector>,
        program_details: Mapping<Hash, ProgramDetails>,
        account_participated_programs: Mapping<AccountId, HashVector>,
        all_programs: Vec<Hash>,
        account_parent: Mapping<AccountId, AccountId>,
        all_claims: Vec<Hash>,
        ip_program_endorsements: Mapping<(Vec<u8>, Hash), u8>,
        program_child_accounts: Mapping<Hash, AccountVector>
    }


    // BEGIN CONTRACT LOGIC >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

    impl ContractStorage {
        
        // CONSTRUCTORS >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // Constructors are implicitly payable.

        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                guard: Default::default(),
                account_claims: Mapping::default(),
                claim_details: Mapping::default(),
                account_branches: Mapping::default(),
                branch_details: Mapping::default(),
                account_payouts: Mapping::default(),
                payout_details: Mapping::default(),
                account_owned_programs: Mapping::default(),
                program_details: Mapping::default(),
                account_participated_programs: Mapping::default(),
                all_programs: <Vec<Hash>>::default(),
                account_parent: Mapping::default(),
                all_claims: <Vec<Hash>>::default(),
                ip_program_endorsements: Mapping::default(),
                program_child_accounts: Mapping::default(),
            }
        }


        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // MESSAGE FUNCTIONS THAT CHANGE DATA IN THE CONTRACT STORAGE >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

        // 1 🟢 Claim
        #[ink(message, payable)]
        #[openbrush::modifiers(non_reentrant)]
        pub fn make_a_claim (&mut self, 
            program_id: Hash,
            parent_ip: Vec<u8>,
            child: AccountId,
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();
            let rightnow = self.env().block_timestamp();

            // get the program details
            let mut this_program = self.program_details.get(&program_id).unwrap_or_default();

            // make sure the program_id actually exists AND is active...
            if self.all_programs.contains(&program_id) && this_program.active == true {
                
                // get the grandparent
                let mut grand: AccountId = ZERO_ADDRESS.into();
                // If the caller has a parent, use that grandparent instead
                if self.account_parent.contains(&caller) {
                    grand = self.account_parent.get(&caller).unwrap();
                }
                
                // get the program child accounts
                let program_children = self.program_child_accounts.get(&program_id).unwrap_or_default();

                // COLLECT PAYMENT FROM THE CALLER
                // the 'payable' tag on this message allows the user to send any amount
                let amount_paid: Balance = self.env().transferred_value();
                if amount_paid < this_program.pay_in_minimum {
                    // error, did not pay enough
                    return Err(Error::InsufficientPayment);
                }
                else {
                    // make the claim_id hash
                    let encodable = (caller, program_id, child); // Implements `scale::Encode`
                    let mut new_id_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
                    ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut new_id_u8);
                    let new_claim_id: Hash = Hash::from(new_id_u8);

                    // make the branch_id hash
                    let encodable = (grand, caller, child); // Implements `scale::Encode`
                    let mut new_id_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
                    ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut new_id_u8);
                    let new_branch_id: Hash = Hash::from(new_id_u8);

                    // if the claim_id or the branch_id already exists in this program, error
                    // OR if the child exists on another endorsed claim in the same program, error
                    if self.all_claims.contains(&new_claim_id) || this_program.branches.contains(&new_branch_id) 
                    || program_children.accountvector.contains(&child) {
                        return Err(Error::GenericError);
                    }
                    // else, proceed...
                    else {
                        // make the Claim structure
                        let new_claim = Claim {
                            program_id: program_id,
                            claim_id: new_claim_id,
                            parent: caller, 
                            parent_ip: parent_ip,
                            child: child,
                            child_ip: <Vec<u8>>::default(),
                            timestamp: rightnow,
                            grandparent: grand,
                            branch: (grand, caller, child),
                            pay_in: amount_paid,
                            endorsed_by: ZERO_ADDRESS.into(),
                            payout_id: Hash::default(),
                            status: 0,
                        };

                        // UPDATE STORAGE MAPPINGS...

                        // account_claims: Mapping<AccountId, HashVector>
                        let mut myclaims = self.account_claims.get(&caller).unwrap_or_default();
                        myclaims.hashvector.push(new_claim_id);
                        self.account_claims.insert(&caller, &myclaims);

                        // claim_details: Mapping<Hash, Claim>
                        self.claim_details.insert(&new_claim_id, &new_claim);

                        // account_branches: Mapping<AccountId, HashVector>
                        let mut grand_branches = self.account_branches.get(&grand).unwrap_or_default();
                        grand_branches.hashvector.push(new_branch_id);
                        self.account_branches.insert(&grand, &grand_branches);

                        // branch_details: Mapping<Hash, Branch>
                        let new_branch = Branch {
                            branch_id: new_branch_id,
                            program_id: program_id,
                            branch: (grand, caller, child)
                        };
                        self.branch_details.insert(&new_branch_id, &new_branch);

                        // account_participated_programs: Mapping<AccountId, HashVector>
                        let mut caller_programs = self.account_participated_programs.get(&caller).unwrap_or_default();
                        if caller_programs.hashvector.contains(&program_id) {
                            // do nothing
                        }
                        else {
                            // add this program to the vector
                            caller_programs.hashvector.push(program_id);
                            self.account_participated_programs.insert(&caller, &caller_programs);
                        }

                        // program_details: Mapping<Hash, Program>
                        this_program.claims_made.push(new_claim_id);
                        this_program.branches.push(new_branch_id);
                        self.program_details.insert(&program_id, &this_program);

                        // account_parent: Mapping<AccountId, AccountId>
                        self.account_parent.insert(&child, &caller);

                        // all_claims: Vec<Hash>
                        self.all_claims.push(new_claim_id);

                        Ok(())
                    }  
                }
            }
            else {
                return Err(Error::GenericError);
            }
        }


        // 2 🟢 Endorse
        #[ink(message)]
        #[openbrush::modifiers(non_reentrant)]
        pub fn endorse_a_claim (&mut self, 
            claim_id: Hash,
            caller_ip: Vec<u8>,
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();
            let rightnow = self.env().block_timestamp();

            // get the claim details
            let mut this_claim = self.claim_details.get(&claim_id).unwrap_or_default();

            // get the program details
            let programid = this_claim.program_id;
            let mut this_program = self.program_details.get(&programid).unwrap_or_default();
            let mut program_endorsement_count = self.ip_program_endorsements.get(&(caller_ip.clone(), programid)).unwrap_or_default();
            let mut program_children = self.program_child_accounts.get(&programid).unwrap_or_default();
            let mut ischild = false;
            if program_children.accountvector.contains(&caller) {
                ischild = true;
            }

            // make sure this ip address has not made more than 2 endorsements for the same program
            // AND the claim exists but has not been endorsed
            // AND the caller is the child account in the claim
            // AND the endorsed_by is the zero address
            // AND the caller (child) ip is not the parent ip
            // AND the caller (child) is not already in the program child vector
            if program_endorsement_count < 2 && this_program.claims_made.contains(&claim_id) 
            && this_claim.child == caller && this_claim.parent_ip != caller_ip.clone() 
            && ischild == false && this_claim.status == 0 {
                // proceed...

                // update the Claim (child_ip and endorsed_by)
                this_claim.child_ip = caller_ip.clone();
                this_claim.endorsed_by = caller;

                if this_program.owner_approval_required == true {
                    // remove the claim from claims_made and move it to endorsed_waiting
                    this_program.claims_endorsed_waiting.push(claim_id);
                    this_program.claims_made.retain(|value| *value != claim_id);
                    this_claim.status = 1;

                    if this_program.program_balance < this_claim.pay_in {
                        return Err(Error::GenericError);
                    }
                    else {
                        // send the endosing child account what the parent had put in for them
                        self.env().transfer(this_claim.child, this_claim.pay_in).expect("payout failed");
                        if self.env().transfer(this_claim.child, this_claim.pay_in).is_err() {
                            return Err(Error::PayoutFailed);
                        }
                        // when the payout is approved, the parent and grandparent will receive their coin
                        // and the RewardPayout will store all of the data for all three recipients

                        this_program.program_balance -= this_claim.pay_in;
                    }

                }
                else {
                    // remove the claim from claims_made and move it to endorsed_approved
                    this_program.claims_endorsed_approved.push(claim_id);
                    this_program.claims_made.retain(|value| *value != claim_id);
                    this_claim.status = 2;

                    // make the payout id hash
                    let encodable = (programid, claim_id); // Implements `scale::Encode`
                    let mut new_id_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
                    ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut new_id_u8);
                    let new_payout_id: Hash = Hash::from(new_id_u8);

                    // make sure this payout has not happened before
                    if this_program.payouts.contains(&new_payout_id) {
                        return Err(Error::GenericError);
                    }
                    else {
                        // proceed to make the payouts
                        // calculate the payouts
                        let child_gets: Balance = this_claim.pay_in;
                        let parent_gets: Balance = this_program.first_level_reward;
                        let mut grand_gets: Balance = 0;
                        if this_claim.grandparent == ZERO_ADDRESS.into() {
                            // do nothing
                        }
                        else {
                            grand_gets = this_program.second_level_reward;
                        }
                        let totalpayout: Balance = child_gets + parent_gets + grand_gets;

                        // make sure the program balance can cover those payouts
                        if this_program.program_balance < totalpayout {
                            //error
                            return Err(Error::GenericError);
                        }
                        else {
                            // create the RewardPayout
                            let reward_payout =  RewardPayout {
                                payout_id: new_payout_id,
                                program_id: this_claim.program_id,
                                claim_id: this_claim.claim_id,
                                child_account: this_claim.child,
                                child_payout: this_claim.pay_in,
                                parent_account: this_claim.parent,
                                parent_payout: this_program.first_level_reward,
                                grandparent_account: this_claim.grandparent,
                                grandparent_payout: this_program.second_level_reward,
                                timestamp: rightnow,
                                total_payout: totalpayout,
                            };

                            // make the payout transactions (child, parent, grandparent)
                            self.env().transfer(this_claim.child, child_gets).expect("payout failed");
                            if self.env().transfer(this_claim.child, child_gets).is_err() {
                                return Err(Error::PayoutFailed);
                            }

                            self.env().transfer(this_claim.parent, parent_gets).expect("payout failed");
                            if self.env().transfer(this_claim.parent, parent_gets).is_err() {
                                return Err(Error::PayoutFailed);
                            }

                            if grand_gets > 0 {
                                self.env().transfer(this_claim.grandparent, grand_gets).expect("payout failed");
                                if self.env().transfer(this_claim.parent, grand_gets).is_err() {
                                    return Err(Error::PayoutFailed);
                                }
                            }

                            // update ProgramDetails (rewards_given #, program_balance, payouts,   
                            // and claims_endorsed_wiating/approved)
                            this_program.rewards_given += 1;
                            this_program.program_balance -= totalpayout;
                            this_program.payouts.push(new_payout_id);

                            // account_payouts: Mapping<AccountId, HashVector> for child, parent and grandparent
                            let mut child_vector = self.account_payouts.get(&this_claim.child).unwrap_or_default();
                            child_vector.hashvector.push(new_payout_id);
                            self.account_payouts.insert(&this_claim.child, &child_vector);

                            let mut parent_vector = self.account_payouts.get(&this_claim.parent).unwrap_or_default();
                            parent_vector.hashvector.push(new_payout_id);
                            self.account_payouts.insert(&this_claim.parent, &parent_vector);

                            if grand_gets > 0 {
                                let mut grand_vector = self.account_payouts.get(&this_claim.grandparent).unwrap_or_default();
                                grand_vector.hashvector.push(new_payout_id);
                                self.account_payouts.insert(&this_claim.grandparent, &grand_vector);
                            }

                            // payout_details: Mapping<Hash, RewardPayout>
                            self.payout_details.insert(&new_payout_id, &reward_payout);

                            // emit the reward payout event
                            Self::env().emit_event(RewardPayoutMade {
                                payout_id: new_payout_id,
                                program_id: programid,
                                claim_id: claim_id,
                                child_account: this_claim.child,
                                child_payout: child_gets,
                                parent_account: this_claim.parent,
                                parent_payout: parent_gets,
                                grandparent_account: this_claim.grandparent,
                                grandparent_payout: grand_gets,
                                timestamp: rightnow,
                                total_payout: totalpayout,
                            });

                            // update the payout id in the claim
                            this_claim.payout_id = new_payout_id;

                        }
                    }
                }

                // increment the program endorsement count from this ip address
                program_endorsement_count += 1;

                // UPDATE CONTRACT STORAGE MAPPINGS...

                // claim_details: Mapping<Hash, Claim>
                self.claim_details.insert(&claim_id, &this_claim);

                // program_details: Mapping<Hash, Program>
                self.program_details.insert(&programid, &this_program);

                // ip_program_endorsements: Mapping<(Vec<u8>, Hash), u8>
                self.ip_program_endorsements.insert(&(caller_ip, programid), &program_endorsement_count);

                // program_child_accounts: Mapping<Hash, AccountVector>
                program_children.accountvector.push(caller);
                self.program_child_accounts.insert(&programid, &program_children);

                Ok(()) 
            }
            else {
                return Err(Error::GenericError);
            }
        }


        // 3 🟢 New Program
        #[ink(message, payable)]
        #[openbrush::modifiers(non_reentrant)]
        pub fn new_program (&mut self, 
            title: Vec<u8>,
            description: Vec<u8>,
            more_info_link: Vec<u8>,
            photo: Vec<u8>,
            first_level_reward: Balance,
            second_level_reward: Balance,
            maximum_rewards: u128,
            owner_approval_required: bool,
            pay_in_minimum: Balance,
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();
            let rightnow = self.env().block_timestamp();

            // make the program id hash
            let encodable = (caller, rightnow, title.clone()); // Implements `scale::Encode`
            let mut new_id_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
            ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut new_id_u8);
            let new_program_id: Hash = Hash::from(new_id_u8);

            // make sure it is not a duplicate
            if self.all_programs.contains(&new_program_id) {
                return Err(Error::GenericError);
            }
            else {
                // proceed
                // collect the initial program funding from the caller
                let amount_paid: Balance = self.env().transferred_value();

                if amount_paid > pay_in_minimum {
                    // proceed...
                    // set up the ProgramDetails
                    let new_program = ProgramDetails {
                        program_id: new_program_id,
                        owner: caller,
                        title: title.clone(),
                        description: description,
                        more_info_link: more_info_link,
                        photo: photo,
                        first_level_reward: first_level_reward,
                        second_level_reward: second_level_reward,
                        maximum_rewards: maximum_rewards,
                        rewards_given: 0,
                        owner_approval_required: owner_approval_required,
                        pay_in_minimum: pay_in_minimum,
                        program_balance: amount_paid,
                        claims_endorsed_approved: <Vec<Hash>>::default(),
                        claims_endorsed_rejected: <Vec<Hash>>::default(),
                        claims_endorsed_waiting:  <Vec<Hash>>::default(),
                        claims_made:  <Vec<Hash>>::default(),
                        branches:  <Vec<Hash>>::default(),
                        payouts:  <Vec<Hash>>::default(),
                        active: true,
                    };

                    // UPDATE MAPPINGS...
                    // account_owned_programs: Mapping<AccountId, HashVector>
                    let mut my_programs = self.account_owned_programs.get(&caller).unwrap_or_default();
                    my_programs.hashvector.push(new_program_id);
                    self.account_owned_programs.insert(&caller, &my_programs);

                    // program_details: Mapping<Hash, ProgramDetails>
                    self.program_details.insert(&new_program_id, &new_program);

                    // all_programs: Vec<Hash>
                    self.all_programs.push(new_program_id);

                    // emit event
                    Self::env().emit_event(NewProgram {
                        program_id: new_program_id,
                        owner: caller,
                        title: title,
                        first_level_reward: first_level_reward,
                        second_level_reward: second_level_reward,
                        maximum_rewards: maximum_rewards,
                        owner_approval_required: owner_approval_required,
                        pay_in_minimum: pay_in_minimum
                    });

                    Ok(())
                }
                else {
                    // error
                    return Err(Error::GenericError);
                }
            }
        }


        // 4 🟢 Fund Your Program
        #[ink(message, payable)]
        #[openbrush::modifiers(non_reentrant)]
        pub fn fund_your_program (&mut self, 
            program_id: Hash
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();

            // get the program details
            let mut this_program = self.program_details.get(&program_id).unwrap_or_default();

            // make sure the caller owns the program
            if this_program.owner == caller {

                // collect the funding from the caller
                let funding: Balance = self.env().transferred_value();

                // update program_details: Mapping<Hash, ProgramDetails>
                this_program.program_balance += funding;
                self.program_details.insert(&program_id, &this_program);

                Ok(())
            }
            else {
                // error
                return Err(Error::GenericError);
            }
        }

        // 5 🟢 Update Your Program
        #[ink(message)]
        #[openbrush::modifiers(non_reentrant)]
        pub fn update_your_program (&mut self, 
            program_id: Hash,
            title: Vec<u8>,
            description: Vec<u8>,
            more_info_link: Vec<u8>,
            photo: Vec<u8>,
            first_level_reward: Balance,
            second_level_reward: Balance,
            maximum_rewards: u128,
            owner_approval_required: bool,
            pay_in_minimum: Balance,
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();

            // get the program details
            let this_program = self.program_details.get(&program_id).unwrap_or_default();

            // make sure the caller owns the program
            if this_program.owner == caller {

                // set up the ProgramDetails
                let update = ProgramDetails {
                    program_id: program_id,
                    owner: this_program.owner,
                    title: title,
                    description: description,
                    more_info_link: more_info_link,
                    photo: photo,
                    first_level_reward: first_level_reward,
                    second_level_reward: second_level_reward,
                    maximum_rewards: maximum_rewards,
                    rewards_given: this_program.rewards_given,
                    owner_approval_required: owner_approval_required,
                    pay_in_minimum: pay_in_minimum,
                    program_balance: this_program.program_balance,
                    claims_endorsed_approved: this_program.claims_endorsed_approved,
                    claims_endorsed_rejected: this_program.claims_endorsed_rejected,
                    claims_endorsed_waiting: this_program.claims_endorsed_waiting,
                    claims_made:  this_program.claims_made,
                    branches:  this_program.branches,
                    payouts:  this_program.payouts,
                    active: this_program.active,
                };

                // update program_details: Mapping<Hash, ProgramDetails>
                self.program_details.insert(&program_id, &update);

                Ok(())
            }
            else {
                // error
                return Err(Error::GenericError);
            }
        }


        // 6 🟢 Deactivate Your Program
        #[ink(message)]
        #[openbrush::modifiers(non_reentrant)]
        pub fn deactivate_your_program (&mut self, 
            program_id: Hash
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();

            // get the program details
            let mut this_program = self.program_details.get(&program_id).unwrap_or_default();

            // make sure the caller owns the program
            let my_programs = self.account_owned_programs.get(&caller).unwrap_or_default();
            if this_program.owner == caller && my_programs.hashvector.contains(&program_id) {

                // return the program funding to the caller (program  owner)
                let refund: Balance = this_program.program_balance;
                self.env().transfer(caller, refund).expect("payout failed");
                if self.env().transfer(caller, refund).is_err() {
                    return Err(Error::PayoutFailed);
                }

                // update program_details: Mapping<Hash, ProgramDetails>
                this_program.program_balance = 0;
                this_program.active = false;
                self.program_details.insert(&program_id, &this_program);

                Ok(())
            }
            else {
                // error
                return Err(Error::GenericError);
            }
        }


        // 7 🟢 Reactivate Your Program
        #[ink(message, payable)]
        #[openbrush::modifiers(non_reentrant)]
        pub fn reactivate_your_program (&mut self, 
            program_id: Hash
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();

            // get the program details
            let mut this_program = self.program_details.get(&program_id).unwrap_or_default();

            // make sure the caller owns the program
            if this_program.owner == caller {

                // collect the program funding from the caller
                let funding: Balance = self.env().transferred_value();

                // update program_details: Mapping<Hash, ProgramDetails>
                this_program.program_balance += funding;
                this_program.active = true;
                self.program_details.insert(&program_id, &this_program);

                Ok(())
            }
            else {
                // error
                return Err(Error::GenericError);
            }
        }


        // 8 🟢 Approve A Payout
        #[ink(message)]
        #[openbrush::modifiers(non_reentrant)]
        pub fn approve_a_payout (&mut self, 
            claim_id: Hash
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();
            let rightnow = self.env().block_timestamp();

            // get the claim details
            let mut this_claim = self.claim_details.get(claim_id).unwrap_or_default();
            // get the program details
            let mut this_program = self.program_details.get(&this_claim.program_id).unwrap_or_default();
            // get the caller's owned programs
            let my_programs = self.account_owned_programs.get(&caller).unwrap_or_default();

            // make sure the caller owns the program AND the claim is waiting to be approved
            if this_program.owner == caller && my_programs.hashvector.contains(&this_claim.program_id) 
            && this_program.claims_endorsed_waiting.contains(&claim_id) && this_claim.status == 1 {

                // remove the claim from claims_endorsed_waiting and move it to claims_endorsed_approved
                this_program.claims_endorsed_approved.push(claim_id);
                this_program.claims_endorsed_waiting.retain(|value| *value != claim_id);
                this_claim.status = 2;

                // make the payout id hash
                let encodable = (this_claim.program_id, claim_id); // Implements `scale::Encode`
                let mut new_id_u8 = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
                ink::env::hash_encoded::<Sha2x256, _>(&encodable, &mut new_id_u8);
                let new_payout_id: Hash = Hash::from(new_id_u8);

                // make sure this payout has not happened before
                if this_program.payouts.contains(&new_payout_id) {
                    return Err(Error::GenericError);
                }
                else {
                    // proceed to make the payouts
                    // calculate the payouts
                    let child_gets: Balance = this_claim.pay_in;
                    // note that the child was already paid when they endorsed
                    let parent_gets: Balance = this_program.first_level_reward;
                    let mut grand_gets: Balance = 0;
                    if this_claim.grandparent == ZERO_ADDRESS.into() {
                        // do nothing
                    }
                    else {
                        grand_gets = this_program.second_level_reward;
                    }
                    let totalpayout: Balance = child_gets + parent_gets + grand_gets;

                    // make sure the program balance can cover those payouts
                    if this_program.program_balance < parent_gets + grand_gets {
                        //error
                        return Err(Error::GenericError);
                    }
                    else {
                        // create the RewardPayout
                        let reward_payout =  RewardPayout {
                            payout_id: new_payout_id,
                            program_id: this_claim.program_id,
                            claim_id: this_claim.claim_id,
                            child_account: this_claim.child,
                            child_payout: this_claim.pay_in,
                            parent_account: this_claim.parent,
                            parent_payout: this_program.first_level_reward,
                            grandparent_account: this_claim.grandparent,
                            grandparent_payout: this_program.second_level_reward,
                            timestamp: rightnow,
                            total_payout: totalpayout,
                        };

                        // make the payout transactions (parent, grandparent)

                        self.env().transfer(this_claim.parent, parent_gets).expect("payout failed");
                        if self.env().transfer(this_claim.parent, parent_gets).is_err() {
                            return Err(Error::PayoutFailed);
                        }

                        if grand_gets > 0 {
                            self.env().transfer(this_claim.grandparent, grand_gets).expect("payout failed");
                            if self.env().transfer(this_claim.parent, grand_gets).is_err() {
                                return Err(Error::PayoutFailed);
                            }
                        }

                        // update ProgramDetails (rewards_given #, program_balance, payouts,   
                        // and claims_endorsed_wiating/approved)
                        this_program.rewards_given += 1;
                        this_program.program_balance -= parent_gets + grand_gets;
                        this_program.payouts.push(new_payout_id);

                        // account_payouts: Mapping<AccountId, HashVector> for child, parent and grandparent
                        let mut child_vector = self.account_payouts.get(&this_claim.child).unwrap_or_default();
                        child_vector.hashvector.push(new_payout_id);
                        self.account_payouts.insert(&this_claim.child, &child_vector);

                        let mut parent_vector = self.account_payouts.get(&this_claim.parent).unwrap_or_default();
                        parent_vector.hashvector.push(new_payout_id);
                        self.account_payouts.insert(&this_claim.parent, &parent_vector);

                        if grand_gets > 0 {
                            let mut grand_vector = self.account_payouts.get(&this_claim.grandparent).unwrap_or_default();
                            grand_vector.hashvector.push(new_payout_id);
                            self.account_payouts.insert(&this_claim.grandparent, &grand_vector);
                        }

                        // payout_details: Mapping<Hash, RewardPayout>
                        self.payout_details.insert(&new_payout_id, &reward_payout);

                        // emit the reward payout event
                        Self::env().emit_event(RewardPayoutMade {
                            payout_id: new_payout_id,
                            program_id: this_claim.program_id,
                            claim_id: claim_id,
                            child_account: this_claim.child,
                            child_payout: child_gets,
                            parent_account: this_claim.parent,
                            parent_payout: parent_gets,
                            grandparent_account: this_claim.grandparent,
                            grandparent_payout: grand_gets,
                            timestamp: rightnow,
                            total_payout: totalpayout,
                        });

                        // update the payout id in the claim details
                        this_claim.payout_id = new_payout_id;

                    }
                    
                    // UPDATE STORAGE MAPPINGS

                    // claim_details: Mapping<Hash, Claim>
                    self.claim_details.insert(&claim_id, &this_claim);

                    // program_details: Mapping<Hash, Program>
                    self.program_details.insert(&this_claim.program_id, &this_program);

                    Ok(())
                }  
            }
            else {
                // error
                return Err(Error::GenericError);
            }
        }


        // 9 🟢 Reject A Payout
        #[ink(message)]
        #[openbrush::modifiers(non_reentrant)]
        pub fn reject_a_payout (&mut self, 
            claim_id: Hash
        ) -> Result<(), Error> {
            // set up the caller
            let caller = Self::env().caller();

            // get the claim details
            let mut this_claim = self.claim_details.get(claim_id).unwrap_or_default();
            let programid = this_claim.program_id;
            // get the program details
            let mut this_program = self.program_details.get(&programid).unwrap_or_default();
            // get the caller's owned programs
            let my_programs = self.account_owned_programs.get(&caller).unwrap_or_default();

            // make sure the caller owns the program AND the claim is waiting to be approved/rejected
            if this_program.owner == caller && my_programs.hashvector.contains(&programid) 
            && this_program.claims_endorsed_waiting.contains(&claim_id) && this_claim.status == 1 {

                // remove the claim from claims_endorsed_waiting and move it to claims_endorsed_rejected
                this_program.claims_endorsed_rejected.push(claim_id);
                this_program.claims_endorsed_waiting.retain(|value| *value != claim_id);
                // update the claim status to 3 (rejected)
                this_claim.status = 3;

                // update mappings...

                // claim_details: Mapping<Hash, Claim>
                self.claim_details.insert(&claim_id, &this_claim);

                // program_details: Mapping<Hash, Program>
                self.program_details.insert(&programid, &this_program);

                Ok(())
            }
            else {
                // error
                return Err(Error::GenericError);
            }
        }
    

        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        // >>>>>>>>>>>>>>>>>>>>>>>>>> PRIMARY GET MESSAGES <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
 
        // 10 🟢 Browse All Programs
        #[ink(message)]
        pub fn browse_all_programs (&self) -> BrowseAllPrograms {

            // set up return structures
            let mut program_vec = <Vec<ProgramPublicDetails>>::default();

            // iterate over all program id hashes
            for id in self.all_programs.iter() {
                // get the program details
                let details = self.program_details.get(id).unwrap_or_default();

                // IF active, package those details into the public details struct
                if details.active == true {
                    let public_details = ProgramPublicDetails {
                        program_id: details.program_id,
                        owner: details.owner,
                        title: details.title,
                        description: details.description,
                        more_info_link: details.more_info_link,
                        photo: details.photo,
                        first_level_reward: details.first_level_reward,
                        second_level_reward: details.second_level_reward,
                        maximum_rewards: details.maximum_rewards,
                        rewards_given: details.rewards_given,
                        owner_approval_required: details.owner_approval_required,
                        pay_in_minimum: details.pay_in_minimum
                    };
                    // add the public details to the program_vec
                    program_vec.push(public_details);
                }
            }

            // package the results
            let results = BrowseAllPrograms {
                programs: program_vec
            };

            // return the results
            results
        }


        // 11 🟢 View My Programs
        #[ink(message)]
        pub fn view_my_programs (&self) -> ViewMyPrograms {
            // set up the caller
            let caller = Self::env().caller();
            // set up return structures
            let mut program_vec = <Vec<ViewProgramDetails>>::default();
            let my_programs = self.account_owned_programs.get(&caller).unwrap_or_default();
            
            // iterate over all program id hashes
            for id in my_programs.hashvector.iter() {
                // get the program details
                let details = self.program_details.get(id).unwrap_or_default();
                let mut approved = <Vec<Claim>>::new();
                let mut rejected = <Vec<Claim>>::new();
                let mut waiting = <Vec<Claim>>::new();
                let mut made = <Vec<Claim>>::new();
                let mut branch_vec = <Vec<Branch>>::new();
                let mut payout_vec = <Vec<RewardPayout>>::new();

                for claimid in details.claims_endorsed_approved.iter() {
                    // get the details and add it to the vector
                    let this = self.claim_details.get(claimid).unwrap_or_default();
                    approved.push(this);
                }

                for claimid in details.claims_endorsed_rejected.iter() {
                    // get the details and add it to the vector
                    let this = self.claim_details.get(claimid).unwrap_or_default();
                    rejected.push(this);
                }

                for claimid in details.claims_endorsed_waiting.iter() {
                    // get the details and add it to the vector
                    let this = self.claim_details.get(claimid).unwrap_or_default();
                    waiting.push(this);
                }

                for claimid in details.claims_made.iter() {
                    // get the details and add it to the vector
                    let this = self.claim_details.get(claimid).unwrap_or_default();
                    made.push(this);
                }

                for branchid in details.branches.iter() {
                    // get the details and add it to the vector
                    let this = self.branch_details.get(branchid).unwrap_or_default();
                    branch_vec.push(this);
                }

                for payoutid in details.payouts.iter() {
                    // get the details and add it to the vector
                    let this = self.payout_details.get(payoutid).unwrap_or_default();
                    payout_vec.push(this);
                }

                let view = ViewProgramDetails {
                    program_id: details.program_id,
                    owner: details.owner,
                    title: details.title,
                    description: details.description,
                    more_info_link: details.more_info_link,
                    photo: details.photo,
                    first_level_reward: details.first_level_reward,
                    second_level_reward: details.second_level_reward,
                    maximum_rewards: details.maximum_rewards,
                    rewards_given: details.rewards_given,
                    owner_approval_required: details.owner_approval_required,
                    pay_in_minimum: details.pay_in_minimum,
                    program_balance: details.program_balance,
                    claims_endorsed_approved: approved,
                    claims_endorsed_rejected: rejected,
                    claims_endorsed_waiting: waiting,
                    claims_made: made,
                    branches: branch_vec,
                    payouts: payout_vec,
                    active: details.active,
                };

                // add the packaged view to the program_vec
                program_vec.push(view);
            }

            // package the results
            let results = ViewMyPrograms {
                programs: program_vec
            };

            // return the results
            results
        }


        // 12 🟢 View My Activity
        #[ink(message)]
        pub fn view_my_activity (&self) -> ViewMyActivity {
            // set up the caller
            let caller = Self::env().caller();
            let caller_claims = self.account_claims.get(&caller).unwrap_or_default();
            let caller_branches = self.account_branches.get(&caller).unwrap_or_default();
            let caller_payouts = self.account_payouts.get(&caller).unwrap_or_default();

            // set up return structures
            let mut user_data = <Vec<UserDataByProgram>>::new();

            // iterate over all the programs this user is involved in
            let programs = self.account_participated_programs.get(&caller).unwrap_or_default();

            for id in programs.hashvector.iter() {

                let mut my_claims = <Vec<Claim>>::new();
                let mut my_payouts = <Vec<RewardPayout>>::new();
                let mut my_branches = <Vec<Branch>>::new();

                // get the program details
                let program_data = self.program_details.get(id).unwrap_or_default();

                for item in caller_claims.hashvector.iter() {
                    // get the details
                    let this = self.claim_details.get(item).unwrap_or_default();
                    // check the program id
                    if this.program_id == *id {
                        // add it to the vector
                        my_claims.push(this);
                    }
                }

                for item in caller_branches.hashvector.iter() {
                    // get the details
                    let this = self.branch_details.get(item).unwrap_or_default();
                    // check the program id
                    if this.program_id == *id {
                        // add it to the vector
                        my_branches.push(this);
                    }
                }

                for item in caller_payouts.hashvector.iter() {
                    // get the details
                    let this = self.payout_details.get(item).unwrap_or_default();
                    // check the program id
                    if this.program_id == *id {
                        // add it to the vector
                        my_payouts.push(this);
                    }
                }

                // package the user data for this program
                let program_user_data = UserDataByProgram {
                    program_id: program_data.program_id,
                    title: program_data.title,
                    description: program_data.description,
                    more_info_link: program_data.more_info_link,
                    photo: program_data.photo,
                    first_level_reward: program_data.first_level_reward,
                    second_level_reward: program_data.second_level_reward,
                    maximum_rewards: program_data.maximum_rewards,
                    rewards_given: program_data.rewards_given,
                    owner_approval_required: program_data.owner_approval_required,
                    pay_in_minimum: program_data.pay_in_minimum, 
                    claims: my_claims,
                    payouts: my_payouts,
                    branches: my_branches
                };

                // add this program data to the user data vector
                user_data.push(program_user_data);
            }
            
            // package the results
            let results: ViewMyActivity = ViewMyActivity {
                activity: user_data
            };

            // return the results
            results
        }


        // END OF MESSAGE LIST

    }
    // END OF CONTRACT STORAGE

}
