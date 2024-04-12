#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		type MaxClaimLength: Get<u32>;

		type WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated {
			who: T::AccountId,
			claim: BoundedVec<u8, T::MaxClaimLength>,
		},
		ClaimRevoked {
			who: T::AccountId,
			claim: BoundedVec<u8, T::MaxClaimLength>,
		},
		ClaimTransferred {
			who: T::AccountId,
			claim: BoundedVec<u8, T::MaxClaimLength>,
			to: T::AccountId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyClaimed,
		NoSuchClaim,
		NotClaimOwner,
	}
	#[pallet::storage]
	pub(super) type Claims<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, BlockNumberFor<T>),
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(Weight::default())]
		#[pallet::call_index(0)]
		pub fn create_claim(
			origin: OriginFor<T>,
			claim: BoundedVec<u8, T::MaxClaimLength>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);

			let current_block = <frame_system::Pallet<T>>::block_number();

			Claims::<T>::insert(&claim, (&sender, current_block));

			Self::deposit_event(Event::ClaimCreated { who: sender, claim });

			Ok(())
		}

		#[pallet::weight(Weight::default())]
		#[pallet::call_index(1)]
		pub fn revoke_claim(
			origin: OriginFor<T>,
			claim: BoundedVec<u8, T::MaxClaimLength>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let (owner, _) = Claims::<T>::get(&claim).ok_or(Error::<T>::NoSuchClaim)?;

			ensure!(sender == owner, Error::<T>::NotClaimOwner);

			Claims::<T>::remove(&claim);

			Self::deposit_event(Event::ClaimRevoked { who: sender, claim });
			Ok(())
		}

		#[pallet::weight(Weight::default())]
		#[pallet::call_index(2)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			claim: BoundedVec<u8, T::MaxClaimLength>,
			target: T::AccountId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let (owner, block) = Claims::<T>::get(&claim).ok_or(Error::<T>::NoSuchClaim)?;

			ensure!(sender == owner, Error::<T>::NotClaimOwner);

			Claims::<T>::insert(&claim, (&target, block));

			Self::deposit_event(Event::ClaimTransferred { who: sender, claim, to: target });

			Ok(())
		}
	}
}
