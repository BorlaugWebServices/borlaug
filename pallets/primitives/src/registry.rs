use codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Registry {
    pub name: Vec<u8>,
}

// use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};

// impl<'de> Deserialize<'de> for Registry {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         #[derive(Deserialize)]
//         #[serde(field_identifier, rename_all = "lowercase")]
//         enum Field {
//             Name,
//         }

//         struct RegistryVisitor;

//         impl<'de> Visitor<'de> for RegistryVisitor {
//             type Value = Registry;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("struct Registry")
//             }

//             fn visit_map<V>(self, mut map: V) -> Result<Registry, V::Error>
//             where
//                 V: MapAccess<'de>,
//             {
//                 let mut name = Vec::new();
//                 while let Some(key) = map.next_key()? {
//                     match key {
//                         Field::Name => {
//                             name = Some(map.next_value()?);
//                         }
//                     }
//                 }

//                 Ok(Registry { name })
//             }
//         }

//         const FIELDS: &'static [&'static str] = &["name"];
//         deserializer.deserialize_struct("Registry", FIELDS, RegistryVisitor)
//     }
// }
