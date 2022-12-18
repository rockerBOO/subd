use anyhow::Result;
use obws::Client as OBSClient;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveSourceCropSetting {
    #[serde(rename = "bottom")]
    pub bottom: Option<f32>,

    #[serde(rename = "left")]
    pub left: Option<f32>,

    #[serde(rename = "top")]
    pub top: Option<f32>,

    #[serde(rename = "right")]
    pub right: Option<f32>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveSourceFilterSettings {
    pub crop: Option<MoveSourceCropSetting>,

    pub bounds: Option<Coordinates>,

    #[serde(rename = "pos")]
    pub position: Option<Coordinates>,

    pub scale: Option<Coordinates>,

    pub duration: Option<u64>,

    pub source: Option<String>,

    // This should be a method on this struct
    // How do we calculate the settings to this string
    //     "transform_text": "pos: x 83.0 y 763.0 rot: 0.0 bounds: x 251.000 y 234.000 crop: l 0 t 0 r 0 b 0",
    pub transform_text: Option<String>,
}

// This is kinda of internal only?

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Coordinates {
    #[serde(rename = "x")]
    pub x: Option<f32>,

    #[serde(rename = "y")]
    pub y: Option<f32>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveMultipleValuesSetting {
    pub filter: Option<String>,
    pub move_value_type: Option<u32>,
    pub value_type: Option<u32>,

    // This is ortho
    #[serde(rename = "Scale.X")]
    pub scale_x: Option<f32>,
    #[serde(rename = "Scale.Y")]
    pub scale_y: Option<f32>,
    #[serde(rename = "Shear.X")]
    pub shear_x: Option<f32>,
    #[serde(rename = "Shear.Y")]
    pub shear_y: Option<f32>,
    #[serde(rename = "Position.X")]
    pub position_x: Option<f32>,
    #[serde(rename = "Position.Y")]
    pub position_y: Option<f32>,
    #[serde(rename = "Rotation.X")]
    pub rotation_x: Option<f32>,
    #[serde(rename = "Rotation.Y")]
    pub rotation_y: Option<f32>,
    #[serde(rename = "Rotation.Z")]
    pub rotation_z: Option<f32>,
}

pub fn default_orthographic_settings() -> MoveMultipleValuesSetting {
    let filter = String::from("3D_Orthographic");
    MoveMultipleValuesSetting {
        filter: Some(filter),
        move_value_type: Some(1),
        value_type: Some(0),
        position_x: Some(0.0),
        position_y: Some(0.0),
        rotation_x: Some(0.0),
        rotation_y: Some(0.0),
        rotation_z: Some(0.0),
        scale_x: Some(100.0),
        scale_y: Some(100.0),
        shear_x: Some(0.0),
        shear_y: Some(0.0),
    }
}

pub fn default_perspective_settings() {}

pub fn default_corner_pin_settings() {}

// =======================================================================
// TODO: We need to organize this by:
//       - generic values
//       - values per filter-type
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveSingleValueSetting {
    #[serde(rename = "source")]
    pub source: Option<String>,

    #[serde(rename = "filter")]
    pub filter: String,
    #[serde(rename = "duration")]
    pub duration: Option<u32>,
    #[serde(rename = "move_value_type")]
    pub move_value_type: Option<u32>,

    #[serde(rename = "setting_float")]
    pub setting_float: f32,
    #[serde(rename = "setting_float_max")]
    pub setting_float_max: f32,
    #[serde(rename = "setting_float_min")]
    pub setting_float_min: f32,
    #[serde(rename = "setting_name")]
    pub setting_name: String,
    #[serde(rename = "value_type")]
    pub value_type: u32,

    // Just for the Blur Filter
    #[serde(rename = "Filter.Blur.Size")]
    pub filter_blur_size: Option<f32>,

    // Just for the SDF Effects Filter
    #[serde(rename = "Filter.SDFEffects.Glow.Inner")]
    pub glow_inner: Option<bool>,
    #[serde(rename = "Filter.SDFEffects.Glow.Outer")]
    pub glow_outer: Option<bool>,
    #[serde(rename = "Filter.SDFEffects.Shadow.Outer")]
    pub shadow_outer: Option<bool>,
    #[serde(rename = "Filter.SDFEffects.Shadow.Inner")]
    pub shadow_inner: Option<bool>,
    #[serde(rename = "Filter.SDFEffects.Outline")]
    pub outline: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveTextFilter {
    #[serde(rename = "setting_name")]
    pub setting_name: String,
    #[serde(rename = "value_type")]
    pub value_type: u32,

    #[serde(rename = "setting_text")]
    pub setting_text: String,
    // "setting_name": "text",
    // "setting_text": "Ok NOW",
    // "value_type": 4
    //
    #[serde(rename = "duration")]
    pub duration: Option<u32>,

    #[serde(rename = "custom_duration")]
    pub custom_duration: bool,

    #[serde(rename = "easing_match")]
    pub easing_match: Option<u32>,

    #[serde(rename = "setting_decimals")]
    pub setting_decimals: Option<u32>,

    // "move_value_type": 4,
    #[serde(rename = "move_value_type")]
    pub move_value_type: Option<u32>,
}

pub fn parse_json_into_struct(file_path: &str) -> MoveSourceFilterSettings {
    let contents = fs::read_to_string(file_path).expect("Can read file");

    let filter: MoveSourceFilterSettings =
        serde_json::from_str(&contents).unwrap();

    filter
}

// So we use this
pub fn create_move_source_filter_settings_from_file(
    file_path: &str,
) -> MoveSourceFilterSettings {
    let filter = parse_json_into_struct(file_path);
    filter
}

pub fn create_move_source_filter_settings(
    source: &str,
) -> MoveSourceFilterSettings {
    let settings = MoveSourceFilterSettings {
        source: Some(source.to_string()),
        duration: Some(300),
        bounds: Some(Coordinates {
            x: Some(251.0),
            y: Some(234.0),
        }),
        scale: Some(Coordinates {
            x: Some(1.0),
            y: Some(1.0),
        }),
        position: Some(Coordinates {
            x: Some(1662.0),
            y: Some(13.0),
        }),
        crop: Some(MoveSourceCropSetting {
            bottom: Some(0.0),
            left: Some(0.0),
            right: Some(0.0),
            top: Some(0.0),
        }),
        transform_text: Some("pos: x 1662.0 y 13.0 rot: 0.0 bounds: x 251.000 y 234.000 crop: l 0 t 0 r 0 b 0".to_string())
    };
    settings
}

// HMMM Why can't they see this???
// This needs to take in Custom Filters
pub fn custom_filter_settings(
    mut base_settings: MoveSourceFilterSettings,
    x: f32,
    y: f32,
) -> MoveSourceFilterSettings {
    base_settings.position = Some(Coordinates {
        x: Some(x),
        y: Some(y),
    });
    base_settings
}

pub async fn create_move_text_value_filter(
    source: &str,
    scene_item: &str,
    filter_name: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let base_settings = create_move_source_filter_settings(scene_item);
    let new_settings = custom_filter_settings(base_settings, 1662.0, 13.0);

    // "id": "move_value_filter",
    // "mixers": 0,
    // "monitoring_type": 0,
    // "muted": false,
    // "name": "OBS_Text",
    // "prev_ver": 469827586,
    // "private_settings": {},
    // "push-to-mute": false,
    // "push-to-mute-delay": 0,
    // "push-to-talk": false,
    // "push-to-talk-delay": 0,
    // "settings": {
    //     "custom_duration": true,
    //     "duration": 300,
    //     "filter": "",
    //     "move_value_type": 4,
    //     "setting_decimals": 1,
    //     "setting_name": "text",
    //     "setting_text": "we are working on getting\nfunctionality up",
    //     "value_type": 4
    // },
    // "sync": 0,
    let new_filter = obws::requests::filters::Create {
        source,
        filter: filter_name,
        kind: "move_source_filter",
        settings: Some(new_settings),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    Ok(())
}

pub async fn create_move_source_filters(
    source: &str,
    scene_item: &str,
    filter_name: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let base_settings = create_move_source_filter_settings(scene_item);
    let new_settings = custom_filter_settings(base_settings, 1662.0, 13.0);

    let new_filter = obws::requests::filters::Create {
        source,
        filter: filter_name,
        kind: "move_source_filter",
        settings: Some(new_settings),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    Ok(())
}
