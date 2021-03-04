#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_support::{
    codec::{Decode, Encode},
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    weights::{Weight},
    Parameter,
};
use frame_system::ensure_signed;
use pallet_ds_accounts as accounts;
use accounts::REGISTRAR_ROLE;

mod default_weight;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum ZoneType {
    /// Forbidden type zone
    Red,
    /// Available for safe flights
    Green,
    /// Owns zones
    Parent,
}

impl Default for ZoneType {
    fn default() -> Self {
        ZoneType::Green
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, Debug, PartialEq, Eq)]
pub struct Point3D<Coord> {
    x: Coord,
    y: Coord,
    z: Coord,
}
impl<Coord> Point3D<Coord> {
    pub fn new(x: Coord, y: Coord, z: Coord) -> Self {
        Point3D{x, y, z}
    }
}


#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, Debug, PartialEq, Eq)]
pub struct Box3D<Point> {
    point_1: Point,
    point_2: Point,
}

impl<Point> Box3D<Point> {
    pub fn new(point_1: Point, point_2: Point) -> Self {
        Box3D{point_1, point_2}
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default)]
pub struct Zone<Point> {
    pub bounding_box: Box3D<Point>,
    pub zone_type: ZoneType,
    pub zone_id: u32,
}

impl<Point> Zone<Point> {
    pub fn zone_is(&self, zone: ZoneType) -> bool {
        self.zone_type == zone
    }

    pub fn new( zone_id: u32, 
                zone_type: ZoneType, 
                bounding_box: Box3D<Point> ) -> Self {
            Zone {
                bounding_box,
                zone_type,
                zone_id,
            }
    }
}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: accounts::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    // Describe pallet constants.
    // Lean more https://substrate.dev/docs/en/knowledgebase/runtime/metadata
    type WeightInfo: WeightInfo;
    // new types, consider description
    /// representing a point in space
    type Point: Default + Parameter;
    /// guess use u32 for representing global coords, u16 for local
    type Coord: Default + Parameter;
}    
pub trait WeightInfo {
    fn zone_add() -> Weight;
}

decl_storage!{
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvv
    trait Store for Module<T: Trait> as DSMapsModule {
        // MAX is 4_294_967_295. Change if required more.
        TotalBoxes get(fn total_boxes): u32;    

        CityMap get(fn map_data): 
            map hasher(blake2_128_concat) u32 => ZoneOf<T>;
    }
}
pub type ZoneOf<T> = Zone<<T as Trait>::Point>;

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Coord = <T as Trait>::Coord,
    {
        // Event documentation should end with an array that provides descriptive names for event parameters.
        /// TODO add more meta/remove
        MapInitialized(Coord),
        /// New account has been created [zone number, its type], TODO later add printing coords
        ZoneCreated(u32, AccountId, ZoneType),
    }
);

// Errors inform users that something went wrong.
// learn more https://substrate.dev/docs/en/knowledgebase/runtime/errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Error names should be descriptive.
        NoneValue,
        /// Operation is not valid
        InvalidAction,
        /// Incorrect data provided
        InvalidData,
        /// Origin do not have sufficient privileges to perform the operation
        NotAuthorized,
        /// Account doesn't exist
        NotExists,
        // add additional errors below
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = <T as Trait>::WeightInfo::zone_add()]
        pub fn zone_add(origin, 
                        zone_type: ZoneType, 
                        bounding_box: Box3D<T::Point>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            // TODO implement inverted index, so we will not store same zones twice
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            
            let id = <TotalBoxes>::get();
            let zone = ZoneOf::<T>::new(id, zone_type.clone(), bounding_box);
            CityMap::<T>::insert(id, zone);
            Self::deposit_event(RawEvent::ZoneCreated(id, who, zone_type));
            <TotalBoxes>::put(id + 1);
            Ok(())
        }
    }
}

// Module allows  use  common functionality by dispatchables
impl<T: Trait> Module<T> {
    // Implement module function.
    // Public functions can be called from other runtime modules.
    /// Check if zone have required type
    pub fn zone_is(zone: u32, zone_type: ZoneType) -> bool {
        CityMap::<T>::get(zone).zone_is(zone_type)
    }
}

