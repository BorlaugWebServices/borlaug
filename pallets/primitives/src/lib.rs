#![no_std]
pub mod asset;
pub mod asset_property;
pub mod attestation;
pub mod attribute;
pub mod audit;
pub mod bounded_vec;
pub mod catalog;
pub mod claim;
pub mod definition;
pub mod definition_step;
pub mod did;
pub mod did_document;
pub mod did_property;
pub mod evidence;
pub mod fact;
pub mod group;
pub mod lease_agreement;
pub mod observation;
pub mod process;
pub mod process_step;
pub mod registry;

pub use self::{
    asset::*, asset_property::*, attestation::*, attribute::*, audit::*, catalog::*, claim::*,
    definition::*, definition_step::*, did::*, did_document::*, did_property::*, evidence::*,
    fact::*, group::*, lease_agreement::*, observation::*, process::*, process_step::*,
    registry::*,
};
pub use codec::Encode;

#[macro_export]
macro_rules! enforce_limit_fact {
    ($id:expr) => {{
        let fact: Fact<BoundedVec<u8, <T as Config>::StringLimit>> = match $id {
            Fact::Bool(v) => Fact::Bool(v),
            Fact::Text(string) => {
                let bounded_string: BoundedVec<u8, <T as Config>::StringLimit> =
                    string.try_into().map_err(|_| Error::<T>::BadString)?;
                Fact::Text(bounded_string)
            }
            Fact::U8(v) => Fact::U8(v),
            Fact::U16(v) => Fact::U16(v),
            Fact::U32(v) => Fact::U32(v),
            Fact::U128(v) => Fact::U128(v),
            Fact::Date(a, b, c) => Fact::Date(a, b, c),
            Fact::Iso8601(a, b, c, d, e, f, g) => Fact::Iso8601(a, b, c, d, e, f, g),
        };
        fact
    }};
}

#[macro_export]
macro_rules! next_id {
    ($id:ty,$t:ty) => {{
        let current_id = <$id>::get();
        let next_id = current_id
            .checked_add(&One::one())
            .ok_or(Error::<$t>::NoIdAvailable)?;
        <$id>::put(next_id);
        current_id
    }};
}
#[macro_export]
macro_rules! enforce_limit {
    ($id:expr) => {{
        let bounded_string: BoundedVec<u8, <T as Config>::StringLimit> =
            $id.try_into().map_err(|_| Error::<T>::BadString)?;
        bounded_string
    }};
}
#[macro_export]
macro_rules! enforce_limit_option {
    ($id:expr) => {{
        let bounded_string = match $id {
            Some(id) => {
                let bounded_string: BoundedVec<u8, <T as Config>::StringLimit> =
                    id.try_into().map_err(|_| Error::<T>::BadString)?;
                Some(bounded_string)
            }
            None => None,
        };
        bounded_string
    }};
}

#[macro_export]
macro_rules! enforce_limit_did_properties_option {
    ($properties:expr) => {{
        $properties
            .map(|properties| {
                properties
                    .into_iter()
                    .map(|property| {
                        Ok(DidProperty {
                            name: enforce_limit!(property.name),
                            fact: enforce_limit_fact!(property.fact),
                        })
                    })
                    .collect::<Result<Vec<_>, Error<T>>>()
            })
            .map_or(Ok(None), |r| r.map(Some))?
    }};
}

#[macro_export]
macro_rules! enforce_limit_did_properties {
    ($properties:expr) => {{
        $properties
            .into_iter()
            .map(|property| {
                Ok(DidProperty {
                    name: enforce_limit!(property.name),
                    fact: enforce_limit_fact!(property.fact),
                })
            })
            .collect::<Result<Vec<_>, Error<T>>>()?
    }};
}

#[macro_export]
macro_rules! enforce_limit_definition_steps_option {
    ($definition_steps:expr) => {{
        $definition_steps
            .map(|definition_steps| {
                definition_steps
                    .into_iter()
                    .map(|definition_step| {
                        Ok(DefinitionStep {
                            name: enforce_limit!(definition_step.name),
                            group_id: definition_step.group_id,
                            threshold: definition_step.threshold,
                        })
                    })
                    .collect::<Result<Vec<_>, Error<T>>>()
            })
            .map_or(Ok(None), |r| r.map(Some))?
    }};
}

#[macro_export]
macro_rules! enforce_limit_definition_steps {
    ($definition_steps:expr) => {{
        $definition_steps
            .into_iter()
            .map(|definition_step| {
                Ok(DefinitionStep {
                    name: enforce_limit!(definition_step.name),
                    group_id: definition_step.group_id,
                    threshold: definition_step.threshold,
                })
            })
            .collect::<Result<Vec<_>, Error<T>>>()?
    }};
}

#[macro_export]
macro_rules! enforce_limit_attributes {
    ($attributes:expr) => {{
        $attributes
            .into_iter()
            .map(|attribute| {
                Ok(Attribute {
                    name: enforce_limit!(attribute.name),
                    fact: enforce_limit_fact!(attribute.fact),
                })
            })
            .collect::<Result<Vec<_>, Error<T>>>()?
    }};
}
