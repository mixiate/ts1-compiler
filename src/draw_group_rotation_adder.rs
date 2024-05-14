use crate::iff_description;
use crate::spr;

use anyhow::Context;

pub fn add_rotations(xml_file_path: &std::path::Path) -> anyhow::Result<()> {
    let mut iff_description = iff_description::IffDescription::open(xml_file_path)
        .with_context(|| format!("Failed to open xml file {}", xml_file_path.display()))?;

    const FLIPPED_SPRITE_FLAG: u32 = 1;

    for draw_group in iff_description.draw_groups.draw_groups.iter_mut() {
        let first_draw_group_items_len = draw_group.draw_group_item_lists[0].draw_group_items.len();
        if draw_group
            .draw_group_item_lists
            .iter()
            .all(|x| x.draw_group_items.len() == first_draw_group_items_len)
        {
            for item_index in 0..first_draw_group_items_len {
                let flipped_sprite_id = draw_group
                    .draw_group_item_lists
                    .iter()
                    .find(|x| x.draw_group_items[item_index].flags & FLIPPED_SPRITE_FLAG != 0)
                    .map(|x| x.draw_group_items[item_index].sprite_chunk_id);
                let flipped_sprite_id_count = if let Some(flipped_sprite_id) = flipped_sprite_id {
                    draw_group.draw_group_item_lists.iter().fold(0, |acc, x| {
                        if x.draw_group_items[item_index].sprite_chunk_id == flipped_sprite_id
                            && x.draw_group_items[item_index].flags & FLIPPED_SPRITE_FLAG != 0
                        {
                            acc + 1
                        } else {
                            acc
                        }
                    })
                } else {
                    0
                };
                let unflipped_sprite_id = draw_group
                    .draw_group_item_lists
                    .iter()
                    .find(|x| x.draw_group_items[item_index].flags & FLIPPED_SPRITE_FLAG == 0)
                    .map(|x| x.draw_group_items[item_index].sprite_chunk_id);
                let unflipped_sprite_id_count = if let Some(unflipped_sprite_id) = unflipped_sprite_id {
                    draw_group.draw_group_item_lists.iter().fold(0, |acc, x| {
                        if x.draw_group_items[item_index].sprite_chunk_id == unflipped_sprite_id
                            && x.draw_group_items[item_index].flags & FLIPPED_SPRITE_FLAG == 0
                        {
                            acc + 1
                        } else {
                            acc
                        }
                    })
                } else {
                    0
                };

                if (flipped_sprite_id_count == draw_group.draw_group_item_lists.len() / 2
                    && unflipped_sprite_id_count == draw_group.draw_group_item_lists.len() / 2)
                    || unflipped_sprite_id_count == draw_group.draw_group_item_lists.len()
                {
                    if let Some(unflipped_sprite_id) = unflipped_sprite_id {
                        for (item_list, item_list_index) in draw_group.draw_group_item_lists.iter_mut().zip(0i32..) {
                            item_list.draw_group_items[item_index].sprite_chunk_id = unflipped_sprite_id;
                            item_list.draw_group_items[item_index].flags &= !FLIPPED_SPRITE_FLAG;
                            item_list.draw_group_items[item_index].sprite_index =
                                spr::SpriteIndex::new(((2 - (item_list_index / 4)) * 4) + (item_list_index % 4));
                        }
                    }
                } else if flipped_sprite_id_count > 0 {
                    println!(
                        "Flipped sprite detected in draw group {} {}, but could not convert to 4 rotations due to \
                        unequal flipped sprite usage",
                        draw_group.chunk_id.as_i16(),
                        draw_group.chunk_label
                    );
                    continue;
                }
            }
        } else {
            'outer: for item_list in &draw_group.draw_group_item_lists {
                for item in &item_list.draw_group_items {
                    if item.flags & FLIPPED_SPRITE_FLAG != 0 {
                        println!(
                            "Flipped sprite detected in draw group {} {}, but could not convert to 4 rotations due to \
                            unequal item list length",
                            draw_group.chunk_id.as_i16(),
                            draw_group.chunk_label
                        );
                        break 'outer;
                    }
                }
            }
        }
    }

    iff_description
        .save(xml_file_path)
        .with_context(|| format!("Failed to save xml file {}", xml_file_path.display()))
}
