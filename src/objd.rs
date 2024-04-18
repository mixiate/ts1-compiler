use crate::iff;

pub const OBJD_CHUNK_DATA_SIZE: usize = 216;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectDefinition {
    #[serde(rename = "@name")]
    pub chunk_label: String,
    #[serde(rename = "@id")]
    pub chunk_id: iff::ChunkId,
    #[serde(rename = "@version")]
    pub version: i32,
    #[serde(rename = "@initialstacksize")]
    pub initialstacksize: i16,
    #[serde(rename = "@basegraphic")]
    pub basegraphic: i16,
    #[serde(rename = "@numgraphics")]
    pub numgraphics: i16,
    #[serde(rename = "@maintreeid")]
    pub maintreeid: i16,
    #[serde(rename = "@gardeningtreeid")]
    pub gardeningtreeid: i16,
    #[serde(rename = "@treetableid")]
    pub treetableid: i16,
    #[serde(rename = "@interactiongroup")]
    pub interactiongroup: i16,
    #[serde(rename = "@type")]
    pub object_type: i16,
    #[serde(rename = "@masterid")]
    pub masterid: i16,
    #[serde(rename = "@subindex")]
    pub subindex: i16,
    #[serde(rename = "@washhandstreeid")]
    pub washhandstreeid: i16,
    #[serde(rename = "@animtableid")]
    pub animtableid: i16,
    #[serde(rename = "@guid")]
    pub guid: i32,
    #[serde(rename = "@disabled")]
    pub disabled: i16,
    #[serde(rename = "@portaltreeid")]
    pub portaltreeid: i16,
    #[serde(rename = "@price")]
    pub price: i16,
    #[serde(rename = "@bodystringsid")]
    pub bodystringsid: i16,
    #[serde(rename = "@slotsid")]
    pub slotsid: i16,
    #[serde(rename = "@allowintersectiontreeid")]
    pub allowintersectiontreeid: i16,
    #[serde(rename = "@usesfntable")]
    pub usesfntable: i16,
    #[serde(rename = "@unused4")]
    pub unused4: i16,
    #[serde(rename = "@preptreeid")]
    pub preptreeid: i16,
    #[serde(rename = "@cooktreeid")]
    pub cooktreeid: i16,
    #[serde(rename = "@surfacetreeid")]
    pub surfacetreeid: i16,
    #[serde(rename = "@disposetreeid")]
    pub disposetreeid: i16,
    #[serde(rename = "@foodtreeid")]
    pub foodtreeid: i16,
    #[serde(rename = "@pickupfromslottreeid")]
    pub pickupfromslottreeid: i16,
    #[serde(rename = "@washdishtreeid")]
    pub washdishtreeid: i16,
    #[serde(rename = "@eatingsurfacetreeid")]
    pub eatingsurfacetreeid: i16,
    #[serde(rename = "@sittreeid")]
    pub sittreeid: i16,
    #[serde(rename = "@standtreeid")]
    pub standtreeid: i16,
    #[serde(rename = "@saleprice")]
    pub saleprice: i16,
    #[serde(rename = "@initialdepreciation")]
    pub initialdepreciation: i16,
    #[serde(rename = "@dailydepreciation")]
    pub dailydepreciation: i16,
    #[serde(rename = "@selfdepreciating")]
    pub selfdepreciating: i16,
    #[serde(rename = "@depreciationlimit")]
    pub depreciationlimit: i16,
    #[serde(rename = "@roomflags")]
    pub roomflags: i16,
    #[serde(rename = "@functionflags")]
    pub functionflags: i16,
    #[serde(rename = "@catalogid")]
    pub catalogid: i16,
    #[serde(rename = "@globalsimulationobject")]
    pub globalsimulationobject: i16,
    #[serde(rename = "@inittreeid")]
    pub inittreeid: i16,
    #[serde(rename = "@placementtreeid")]
    pub placementtreeid: i16,
    #[serde(rename = "@userpickuptreeid")]
    pub userpickuptreeid: i16,
    #[serde(rename = "@wallstyle")]
    pub wallstyle: i16,
    #[serde(rename = "@loadtreeid")]
    pub loadtreeid: i16,
    #[serde(rename = "@userplacementtreeid")]
    pub userplacementtreeid: i16,
    #[serde(rename = "@objectversion")]
    pub objectversion: i16,
    #[serde(rename = "@roomchangedtreeid")]
    pub roomchangedtreeid: i16,
    #[serde(rename = "@motiveeffectsid")]
    pub motiveeffectsid: i16,
    #[serde(rename = "@cleanuptreeid")]
    pub cleanuptreeid: i16,
    #[serde(rename = "@levelinforequesttreeid")]
    pub levelinforequesttreeid: i16,
    #[serde(rename = "@catalogpopupid")]
    pub catalogpopupid: i16,
    #[serde(rename = "@servingsurfacetreeid")]
    pub servingsurfacetreeid: i16,
    #[serde(rename = "@leveloffset")]
    pub leveloffset: i16,
    #[serde(rename = "@shadow")]
    pub shadow: i16,
    #[serde(rename = "@numattributes")]
    pub numattributes: i16,
    #[serde(rename = "@cleantreeid")]
    pub cleantreeid: i16,
    #[serde(rename = "@queueskippedtreeid")]
    pub queueskippedtreeid: i16,
    #[serde(rename = "@frontfacedirection")]
    pub frontfacedirection: i16,
    #[serde(rename = "@walladjacencychangedtreeid")]
    pub walladjacencychangedtreeid: i16,
    #[serde(rename = "@leadobject")]
    pub leadobject: i16,
    #[serde(rename = "@dynspritebaseid")]
    pub dynspritebaseid: i16,
    #[serde(rename = "@numdynsprites")]
    pub numdynsprites: i16,
    #[serde(rename = "@chairentryflags")]
    pub chairentryflags: i16,
    #[serde(rename = "@tilewidth")]
    pub tilewidth: i16,
    #[serde(rename = "@suitnotcopyable")]
    pub suitnotcopyable: i16,
    #[serde(rename = "@buildmodetype")]
    pub buildmodetype: i16,
    #[serde(rename = "@originalguid")]
    pub originalguid: i32,
    #[serde(rename = "@originalsuitguid")]
    pub originalsuitguid: i32,
    #[serde(rename = "@pickuptreeid")]
    pub pickuptreeid: i16,
    #[serde(rename = "@thumbnailgraphicindex")]
    pub thumbnailgraphicindex: i16,
    #[serde(rename = "@shadowflags")]
    pub shadowflags: i16,
    #[serde(rename = "@footprintinsetmask")]
    pub footprintinsetmask: i16,
    #[serde(rename = "@mtadjupdatetreeid")]
    pub mtadjupdatetreeid: i16,
    #[serde(rename = "@shadowbrightness")]
    pub shadowbrightness: i16,
    #[serde(rename = "@repairtreeid")]
    pub repairtreeid: i16,
    #[serde(rename = "@customwallstyleid")]
    pub customwallstyleid: i16,
    #[serde(rename = "@ratinghunger")]
    pub ratinghunger: i16,
    #[serde(rename = "@ratingcomfort")]
    pub ratingcomfort: i16,
    #[serde(rename = "@ratinghygiene")]
    pub ratinghygiene: i16,
    #[serde(rename = "@ratingbladder")]
    pub ratingbladder: i16,
    #[serde(rename = "@ratingenergy")]
    pub ratingenergy: i16,
    #[serde(rename = "@ratingfun")]
    pub ratingfun: i16,
    #[serde(rename = "@ratingroom")]
    pub ratingroom: i16,
    #[serde(rename = "@ratingskillflags")]
    pub ratingskillflags: i16,
    #[serde(rename = "@numtypeattributes")]
    pub numtypeattributes: i16,
    #[serde(rename = "@miscflags")]
    pub miscflags: i16,
    #[serde(rename = "@typeattrguid")]
    pub typeattrguid: i32,
    #[serde(rename = "@functionsubsort")]
    pub functionsubsort: i16,
    #[serde(rename = "@downtownsort")]
    pub downtownsort: i16,
    #[serde(rename = "@keepbuying")]
    pub keepbuying: i16,
    #[serde(rename = "@vacationsort")]
    pub vacationsort: i16,
    #[serde(rename = "@resetlotaction")]
    pub resetlotaction: i16,
    #[serde(rename = "@communitysort")]
    pub communitysort: i16,
    #[serde(rename = "@dreamflags")]
    pub dreamflags: i16,
    #[serde(rename = "@renderflags")]
    pub renderflags: i16,
    #[serde(rename = "@unused8")]
    pub unused8: i16,
    #[serde(rename = "@unused9")]
    pub unused9: i16,
    #[serde(rename = "@unused10")]
    pub unused10: i16,
    #[serde(rename = "@unused11")]
    pub unused11: i16,
    #[serde(rename = "@unused12")]
    pub unused12: i16,
    #[serde(rename = "@unused13")]
    pub unused13: i16,
}

impl ObjectDefinition {
    pub fn write(&self, mut writer: impl std::io::Write, replacement_guid: Option<i32>) {
        let objd_chunk_header = iff::ChunkHeader::new("OBJD", OBJD_CHUNK_DATA_SIZE, self.chunk_id, &self.chunk_label);
        objd_chunk_header.write(&mut writer);

        writer.write_all(&self.version.to_le_bytes()).unwrap();
        writer.write_all(&self.initialstacksize.to_le_bytes()).unwrap();
        writer.write_all(&self.basegraphic.to_le_bytes()).unwrap();
        writer.write_all(&self.numgraphics.to_le_bytes()).unwrap();
        writer.write_all(&self.maintreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.gardeningtreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.treetableid.to_le_bytes()).unwrap();
        writer.write_all(&self.interactiongroup.to_le_bytes()).unwrap();
        writer.write_all(&self.object_type.to_le_bytes()).unwrap();
        writer.write_all(&self.masterid.to_le_bytes()).unwrap();
        writer.write_all(&self.subindex.to_le_bytes()).unwrap();
        writer.write_all(&self.washhandstreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.animtableid.to_le_bytes()).unwrap();
        writer.write_all(&replacement_guid.unwrap_or(self.guid).to_le_bytes()).unwrap();
        writer.write_all(&self.disabled.to_le_bytes()).unwrap();
        writer.write_all(&self.portaltreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.price.to_le_bytes()).unwrap();
        writer.write_all(&self.bodystringsid.to_le_bytes()).unwrap();
        writer.write_all(&self.slotsid.to_le_bytes()).unwrap();
        writer.write_all(&self.allowintersectiontreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.usesfntable.to_le_bytes()).unwrap();
        writer.write_all(&self.unused4.to_le_bytes()).unwrap();
        writer.write_all(&self.preptreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.cooktreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.surfacetreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.disposetreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.foodtreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.pickupfromslottreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.washdishtreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.eatingsurfacetreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.sittreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.standtreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.saleprice.to_le_bytes()).unwrap();
        writer.write_all(&self.initialdepreciation.to_le_bytes()).unwrap();
        writer.write_all(&self.dailydepreciation.to_le_bytes()).unwrap();
        writer.write_all(&self.selfdepreciating.to_le_bytes()).unwrap();
        writer.write_all(&self.depreciationlimit.to_le_bytes()).unwrap();
        writer.write_all(&self.roomflags.to_le_bytes()).unwrap();
        writer.write_all(&self.functionflags.to_le_bytes()).unwrap();
        writer.write_all(&self.catalogid.to_le_bytes()).unwrap();
        writer.write_all(&self.globalsimulationobject.to_le_bytes()).unwrap();
        writer.write_all(&self.inittreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.placementtreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.userpickuptreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.wallstyle.to_le_bytes()).unwrap();
        writer.write_all(&self.loadtreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.userplacementtreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.objectversion.to_le_bytes()).unwrap();
        writer.write_all(&self.roomchangedtreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.motiveeffectsid.to_le_bytes()).unwrap();
        writer.write_all(&self.cleanuptreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.levelinforequesttreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.catalogpopupid.to_le_bytes()).unwrap();
        writer.write_all(&self.servingsurfacetreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.leveloffset.to_le_bytes()).unwrap();
        writer.write_all(&self.shadow.to_le_bytes()).unwrap();
        writer.write_all(&self.numattributes.to_le_bytes()).unwrap();
        writer.write_all(&self.cleantreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.queueskippedtreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.frontfacedirection.to_le_bytes()).unwrap();
        writer.write_all(&self.walladjacencychangedtreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.leadobject.to_le_bytes()).unwrap();
        writer.write_all(&self.dynspritebaseid.to_le_bytes()).unwrap();
        writer.write_all(&self.numdynsprites.to_le_bytes()).unwrap();
        writer.write_all(&self.chairentryflags.to_le_bytes()).unwrap();
        writer.write_all(&self.tilewidth.to_le_bytes()).unwrap();
        writer.write_all(&self.suitnotcopyable.to_le_bytes()).unwrap();
        writer.write_all(&self.buildmodetype.to_le_bytes()).unwrap();
        writer.write_all(&self.originalguid.to_le_bytes()).unwrap();
        writer.write_all(&self.originalsuitguid.to_le_bytes()).unwrap();
        writer.write_all(&self.pickuptreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.thumbnailgraphicindex.to_le_bytes()).unwrap();
        writer.write_all(&self.shadowflags.to_le_bytes()).unwrap();
        writer.write_all(&self.footprintinsetmask.to_le_bytes()).unwrap();
        writer.write_all(&self.mtadjupdatetreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.shadowbrightness.to_le_bytes()).unwrap();
        writer.write_all(&self.repairtreeid.to_le_bytes()).unwrap();
        writer.write_all(&self.customwallstyleid.to_le_bytes()).unwrap();
        writer.write_all(&self.ratinghunger.to_le_bytes()).unwrap();
        writer.write_all(&self.ratingcomfort.to_le_bytes()).unwrap();
        writer.write_all(&self.ratinghygiene.to_le_bytes()).unwrap();
        writer.write_all(&self.ratingbladder.to_le_bytes()).unwrap();
        writer.write_all(&self.ratingenergy.to_le_bytes()).unwrap();
        writer.write_all(&self.ratingfun.to_le_bytes()).unwrap();
        writer.write_all(&self.ratingroom.to_le_bytes()).unwrap();
        writer.write_all(&self.ratingskillflags.to_le_bytes()).unwrap();
        writer.write_all(&self.numtypeattributes.to_le_bytes()).unwrap();
        writer.write_all(&self.miscflags.to_le_bytes()).unwrap();
        writer.write_all(&self.typeattrguid.to_le_bytes()).unwrap();
        writer.write_all(&self.functionsubsort.to_le_bytes()).unwrap();
        writer.write_all(&self.downtownsort.to_le_bytes()).unwrap();
        writer.write_all(&self.keepbuying.to_le_bytes()).unwrap();
        writer.write_all(&self.vacationsort.to_le_bytes()).unwrap();
        writer.write_all(&self.resetlotaction.to_le_bytes()).unwrap();
        writer.write_all(&self.communitysort.to_le_bytes()).unwrap();
        writer.write_all(&self.dreamflags.to_le_bytes()).unwrap();
        writer.write_all(&self.renderflags.to_le_bytes()).unwrap();
        writer.write_all(&self.unused8.to_le_bytes()).unwrap();
        writer.write_all(&self.unused9.to_le_bytes()).unwrap();
        writer.write_all(&self.unused10.to_le_bytes()).unwrap();
        writer.write_all(&self.unused11.to_le_bytes()).unwrap();
        writer.write_all(&self.unused12.to_le_bytes()).unwrap();
        writer.write_all(&self.unused13.to_le_bytes()).unwrap();
    }
}
