use crate::game::assets::{AssetType, asset_types};
use bevy::prelude::*;
use bevy_rich_text3d::TextAtlas;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub const BUILTIN_PREFIX: &str = "builtin:";

pub const BUILTIN_TEXT_ATLAS: &str = "text-atlas";

lazy_static! {
    pub static ref BUILTIN_HANDLES: HashMap<String, HashMap<String, UntypedHandle>> = {
        let mut builtins = HashMap::new();

        asset_types!(
            _Asset,
            builtins.insert(_Asset::TYPE_NAME.to_string(), HashMap::new())
        );

        builtins.get_mut(Image::TYPE_NAME).unwrap().insert(
            BUILTIN_TEXT_ATLAS.to_string(),
            TextAtlas::DEFAULT_IMAGE.into(),
        );

        builtins
    };
}
