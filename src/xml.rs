use crate::dgrp;
use crate::objd;
use crate::slot;
use crate::spr;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct IffXml {
    #[serde(rename = "@objectfilename")]
    pub iff_file_path_relative: String,
    #[serde(rename = "@exportobjectdefinitions")]
    exportobjectdefinitions: i32,
    #[serde(rename = "@exportslots")]
    exportslots: i32,
    #[serde(rename = "@exportdrawgroups")]
    exportdrawgroups: i32,
    #[serde(rename = "@exportbitmaps")]
    exportbitmaps: i32,
    #[serde(rename = "@exportsprites")]
    exportsprites: i32,
    #[serde(rename = "@justchangecolors")]
    justchangecolors: i32,
    #[serde(rename = "@exportallzooms")]
    exportallzooms: i32,
    #[serde(rename = "@smoothsmallzoomcolors")]
    smoothsmallzoomcolors: i32,
    #[serde(rename = "@smoothsmallzoomedges")]
    smoothsmallzoomedges: i32,
    #[serde(rename = "@exportexpanded")]
    exportexpanded: i32,
    #[serde(rename = "@exportp")]
    exportp: i32,
    #[serde(rename = "@exportz")]
    exportz: i32,
    #[serde(rename = "@generatez")]
    generatez: i32,
    #[serde(rename = "@generatezfar")]
    generatezfar: i32,
    #[serde(rename = "@exporta")]
    exporta: i32,
    #[serde(rename = "@generatea")]
    generatea: i32,
    #[serde(rename = "@generateasoft")]
    generateasoft: i32,
    #[serde(rename = "@compressbitmaps")]
    compressbitmaps: i32,
    #[serde(rename = "@createsubdirectories")]
    createsubdirectories: i32,
    #[serde(rename = "@thingstodo")]
    thingstodo: i32,
    #[serde(rename = "objectdefinitions")]
    pub object_definitions: ObjectDefinitions,
    #[serde(rename = "slots")]
    pub slots: Slots,
    #[serde(rename = "drawgroups")]
    pub draw_groups: DrawGroups,
    #[serde(rename = "sprites")]
    pub sprites: Sprites,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectDefinitions {
    #[serde(default, rename = "objectdefinition")]
    pub object_definitions: Vec<objd::ObjectDefinition>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Slots {
    #[serde(default, rename = "slot")]
    pub slots: Vec<slot::Slot>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct DrawGroups {
    #[serde(default, rename = "drawgroup")]
    pub draw_groups: Vec<dgrp::DrawGroup>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sprites {
    #[serde(default, rename = "sprite")]
    pub sprites: Vec<spr::Sprite>,
}
