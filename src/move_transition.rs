use crate::obs;
use crate::obs_source;
use anyhow::Result;
use obws::Client as OBSClient;
use serde::{Deserialize, Serialize};
use std::fs;
use std::thread;
use std::time;
use std::time::Duration;

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

// THESE EXTRA VALUES ARE BULLSHIT!!!
// WE NEED TO ABSTRACT THEM AWAY
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

// ======================================================================
// == Defaults ==========================================================
// ======================================================================

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
// == Utilities ==========================================================
// =======================================================================

// This is a simple utility method
pub fn parse_json_into_struct(file_path: &str) -> MoveSourceFilterSettings {
    let contents = fs::read_to_string(file_path).expect("Can read file");

    let filter: MoveSourceFilterSettings =
        serde_json::from_str(&contents).unwrap();

    filter
}

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

// ===============================================================
// == TEXT
// ===============================================================

// So I need a version to update a text value
// start very unspecific
pub async fn update_and_trigger_text_move_filter(
    source: &str,
    filter_name: &str,
    new_text: &String,
    obs_client: &OBSClient,
) -> Result<()> {
    let mut new_settings: MoveTextFilter = Default::default();
    new_settings.move_value_type = Some(4);
    new_settings.setting_name = "text".to_string();
    new_settings.setting_text = new_text.to_string();
    new_settings.value_type = 4;
    new_settings.duration = Some(300);
    new_settings.custom_duration = true;
    new_settings.setting_decimals = Some(1);

    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: &filter_name,
        settings: new_settings,
        overlay: None,
    };

    obs_client.filters().set_settings(new_settings).await?;

    // This fixes the problem
    // TODO: this should be abstracted into a constant
    let ten_millis = time::Duration::from_millis(300);

    thread::sleep(ten_millis);

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;
    Ok(())
}

// ===================================================================================
// == Highest Level MOVE SOURCE
// ===================================================================================

// We update the filter with the settings passed in
// we then trigger than scene
//
// TODO: This needs to take in a scene
pub async fn move_with_move_source(
    filter_name: &str,
    new_settings: MoveSourceFilterSettings,
    obs_client: &obws::Client,
) -> Result<()> {
    update_move_source_filters(
        obs::DEFAULT_SCENE,
        filter_name,
        new_settings,
        &obs_client,
    )
    .await?;

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: obs::DEFAULT_SCENE,
        filter: &filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    Ok(())
}

// ===================================================================================
// == MOVE SOURCE ====================================================================
// ===================================================================================

pub async fn update_and_trigger_move_value_filter(
    source: &str,
    filter_name: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    value_type: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    // Fetch the current settings of the filter we are going to update and trigger
    let filter_details =
        match obs_client.filters().get(&source, &filter_name).await {
            Ok(val) => Ok(val),
            Err(err) => Err(err),
        }?;

    // Parse the settings into a MoveSingleValueSetting struct
    let mut new_settings = match serde_json::from_value::<MoveSingleValueSetting>(
        filter_details.settings,
    ) {
        Ok(val) => val,
        Err(e) => {
            println!("Error: {:?}", e);
            MoveSingleValueSetting {
                ..Default::default()
            }
        }
    };

    // Update the settings based on what is passed into the function
    new_settings.setting_name = String::from(filter_setting_name);
    new_settings.setting_float = filter_value;
    new_settings.duration = Some(duration);
    new_settings.value_type = value_type;

    // Create a SetSettings struct & use it to update the OBS settings
    // TODO: Should this moved into the update_move_source_filters function?
    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: &filter_name,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    // Pause so the settings can take effect before triggering the filter
    // TODO: Extract out into variable
    thread::sleep(Duration::from_millis(400));

    // Trigger the filter
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    // We always return Ok, because even if we fail to enable we want to continue our program
    // TODO: this might not be true since await? will bubble up an error?
    Ok(())
}

// ====================================================================
// == LOWER LEVEL???? =================================================
// ====================================================================

// This takes in settings and updates a filter
async fn update_move_source_filters(
    source: &str,
    filter_name: &str,
    new_settings: MoveSourceFilterSettings,
    obs_client: &OBSClient,
) -> Result<()> {
    let new_filter = obws::requests::filters::SetSettings {
        source,
        filter: filter_name,
        settings: Some(new_settings),
        overlay: Some(false),
    };
    obs_client.filters().set_settings(new_filter).await?;

    Ok(())
}

// ===============================================================================
// == FETCHING ===================================================================
// ===============================================================================

// This function is long!!!
pub async fn fetch_source_settings(
    scene: &str,
    source: &str,
    obs_client: &OBSClient,
) -> Result<MoveSourceFilterSettings> {
    let id = match obs_source::find_id(scene, source, &obs_client).await {
        Ok(val) => val,
        Err(_) => {
            return Ok(MoveSourceFilterSettings {
                ..Default::default()
            })
        }
    };

    let settings = match obs_client.scene_items().transform(scene, id).await {
        Ok(val) => val,
        Err(err) => {
            println!("Error Fetching Transform Settings: {:?}", err);
            let blank_transform =
                obws::responses::scene_items::SceneItemTransform {
                    ..Default::default()
                };
            blank_transform
        }
    };

    let transform_text = format!(
        "pos: x {} y {} rot: 0.0 bounds: x {} y {} crop: l {} t {} r {} b {}",
        settings.position_x,
        settings.position_y,
        settings.bounds_width,
        settings.bounds_height,
        settings.crop_left,
        settings.crop_top,
        settings.crop_right,
        settings.crop_bottom
    );

    let new_settings = MoveSourceFilterSettings {
        source: Some(source.to_string()),
        duration: Some(4444),
        bounds: Some(Coordinates {
            x: Some(settings.bounds_width),
            y: Some(settings.bounds_height),
        }),
        scale: Some(Coordinates {
            x: Some(settings.scale_x),
            y: Some(settings.scale_y),
        }),
        position: Some(Coordinates {
            x: Some(settings.position_x),
            y: Some(settings.position_y),
        }),
        crop: Some(MoveSourceCropSetting {
            left: Some(settings.crop_left as f32),
            right: Some(settings.crop_right as f32),
            bottom: Some(settings.crop_bottom as f32),
            top: Some(settings.crop_top as f32),
        }),
        transform_text: Some(transform_text.to_string()),
    };

    Ok(new_settings)
}
